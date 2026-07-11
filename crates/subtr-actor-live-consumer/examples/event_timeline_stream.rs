//! Real-time event timeline for a live game exported by the state-export
//! plugin: one line per event transaction, updating as spans grow and events
//! finalize.
//!
//! ```text
//! cargo run -p subtr-actor-live-consumer --example event_timeline_stream -- \
//!     --url ws://127.0.0.1:49109 --format postcard
//! ```
//!
//! Line markers: `●` a newly confirmed event (may still be revised), `✓` a
//! finalized event (first sight or the finalizing revision), `~` a content
//! revision of a previously printed id (a span's end growing, enrichment
//! landing), `✗` a retraction. Span events print `start–end` times; moment
//! events a single time. Pass `--json` to emit raw transactions as JSON
//! lines for piping instead of the human-readable stream.
//!
//! Reconnects with backoff on any error; the server's snapshot-on-connect
//! makes reconnecting cheap.

use std::collections::HashMap;
use std::time::Duration;

use clap::{Parser, ValueEnum};

use subtr_actor::{
    Event, EventLifecycle, EventPropertyValue, EventTiming, EventTransaction, PlayerId,
};
use subtr_actor_live::Encoding;
use subtr_actor_live_consumer::{DriverOutput, LiveClient, LiveGraphDriver, LiveStateStore};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Format {
    Postcard,
    Json,
}

impl From<Format> for Encoding {
    fn from(format: Format) -> Self {
        match format {
            Format::Postcard => Encoding::Postcard,
            Format::Json => Encoding::Json,
        }
    }
}

fn default_url() -> String {
    format!(
        "ws://127.0.0.1:{}",
        subtr_actor_live::DEFAULT_STATE_EXPORT_PORT
    )
}

/// Stream a live game's event timeline from a subtr-actor live export server.
#[derive(Parser, Debug)]
struct Args {
    /// WebSocket URL of the live export server.
    #[arg(long, default_value_t = default_url())]
    url: String,
    /// Wire encoding to negotiate.
    #[arg(long, value_enum, default_value_t = Format::Postcard)]
    format: Format,
    /// Cap on event-free frame delivery, in Hz.
    #[arg(long)]
    max_frame_hz: Option<f32>,
    /// Emit raw event transactions as JSON lines instead of the
    /// human-readable timeline.
    #[arg(long)]
    json: bool,
}

const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(30);

fn main() {
    let args = Args::parse();
    let encoding: Encoding = args.format.into();

    let mut store = LiveStateStore::new();
    let mut driver = LiveGraphDriver::new();
    let mut printer = TimelinePrinter::new(args.json);
    let mut backoff = INITIAL_BACKOFF;

    loop {
        let mut client = match LiveClient::connect(&args.url, encoding, args.max_frame_hz) {
            Ok(client) => client,
            Err(error) => {
                eprintln!(
                    "connect to {} failed: {error}; retrying in {backoff:?}",
                    args.url
                );
                std::thread::sleep(backoff);
                backoff = (backoff * 2).min(MAX_BACKOFF);
                continue;
            }
        };
        let (major, minor) = client.server_protocol();
        eprintln!(
            "connected to {} (server \"{}\", protocol {major}.{minor}, {:?})",
            args.url,
            client.server_name(),
            client.encoding(),
        );
        backoff = INITIAL_BACKOFF;

        loop {
            let message = match client.next_message() {
                Ok(Some(message)) => message,
                Ok(None) => {
                    eprintln!("server closed the connection; reconnecting");
                    break;
                }
                Err(error) => {
                    eprintln!("read failed: {error}; reconnecting");
                    break;
                }
            };
            let mut outputs = Vec::new();
            let result = driver.on_message(&mut store, message, &mut |output| {
                outputs.push(output);
            });
            for output in outputs {
                printer.print_output(&store, output);
            }
            if let Err(error) = result {
                // Graph failures are typically persistent for the current
                // match; start over with fresh state.
                eprintln!("graph evaluation failed: {error}; resetting consumer state");
                store = LiveStateStore::new();
                driver = LiveGraphDriver::new();
                printer.reset();
            }
        }
    }
}

/// Renders driver outputs as a line-per-transaction timeline (or JSON lines),
/// tracking which ids have been printed so revisions are marked distinctly
/// and the match-end summary can count the surviving events per stream.
struct TimelinePrinter {
    json: bool,
    /// Stream of every currently-live printed id, for revision detection and
    /// the per-stream match-end summary.
    stream_by_id: HashMap<String, String>,
}

impl TimelinePrinter {
    fn new(json: bool) -> Self {
        Self {
            json,
            stream_by_id: HashMap::new(),
        }
    }

    fn print_output(&mut self, store: &LiveStateStore, output: DriverOutput) {
        match output {
            DriverOutput::MatchStarted { meta } => {
                self.reset();
                eprintln!("match started with {} players:", meta.players.len());
                print_roster(&meta);
            }
            DriverOutput::RosterChanged { meta } => {
                eprintln!("roster changed:");
                print_roster(&meta);
            }
            DriverOutput::EventTransactions(transactions) => {
                for transaction in &transactions {
                    self.print_transaction(store, transaction);
                }
            }
            DriverOutput::MatchEnded => {
                self.print_match_summary();
                self.reset();
            }
            DriverOutput::ServerRestarted => {
                eprintln!("server restarted; state reset");
                self.reset();
            }
        }
    }

    fn print_transaction(&mut self, store: &LiveStateStore, transaction: &EventTransaction) {
        if self.json {
            match serde_json::to_string(transaction) {
                Ok(line) => println!("{line}"),
                Err(error) => eprintln!("could not serialize transaction: {error}"),
            }
        }
        match transaction {
            EventTransaction::Upsert { event, .. } => {
                let seen_before = self.stream_by_id.contains_key(&event.meta.id);
                self.stream_by_id
                    .insert(event.meta.id.clone(), event.meta.stream.clone());
                if !self.json {
                    let marker = match (seen_before, event.meta.lifecycle) {
                        // The finalizing revision and a born-final event both
                        // read as "this will not change again".
                        (_, EventLifecycle::Finalized) => "✓",
                        (false, EventLifecycle::Confirmed) => "●",
                        (true, EventLifecycle::Confirmed) => "~",
                    };
                    print_event_line(store, event, marker);
                }
            }
            EventTransaction::Retract { id, .. } => {
                self.stream_by_id.remove(id);
                if !self.json {
                    println!("          ✗ {id}");
                }
            }
        }
    }

    fn print_match_summary(&self) {
        let mut counts_by_stream: HashMap<&str, usize> = HashMap::new();
        for stream in self.stream_by_id.values() {
            *counts_by_stream.entry(stream.as_str()).or_insert(0) += 1;
        }
        let mut counts: Vec<(&str, usize)> = counts_by_stream.into_iter().collect();
        counts.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
        eprintln!("match ended ({} timeline events)", self.stream_by_id.len());
        for (stream, count) in counts {
            eprintln!("  {count:>5}  {stream}");
        }
    }

    fn reset(&mut self) {
        self.stream_by_id.clear();
    }
}

fn print_roster(meta: &subtr_actor_live::LiveMatchMeta) {
    for player in &meta.players {
        eprintln!(
            "  [team {}] {}",
            if player.is_team_0 { 0 } else { 1 },
            player.name.as_deref().unwrap_or("<unnamed>"),
        );
    }
}

fn player_name(store: &LiveStateStore, player_id: &PlayerId) -> String {
    store
        .player_name(player_id)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{player_id:?}"))
}

fn print_event_line(store: &LiveStateStore, event: &Event, marker: &str) {
    // Span events show `start–end`; the end keeps updating on `~` revision
    // lines while the span is open.
    let clock = match event.meta.timing {
        EventTiming::Moment { time, .. } => format!("{time:8.2}s"),
        EventTiming::Span {
            start_time,
            end_time,
            ..
        } => format!("{start_time:8.2}–{end_time:.2}s"),
    };
    let mut players = event
        .meta
        .primary_player
        .as_ref()
        .map(|player| player_name(store, player))
        .unwrap_or_default();
    if let Some(secondary) = &event.meta.secondary_player {
        players = format!("{players} → {}", player_name(store, secondary));
    }
    let team = event
        .meta
        .team_is_team_0
        .map(|is_team_0| format!(" [team {}]", if is_team_0 { 0 } else { 1 }))
        .unwrap_or_default();
    let confidence = event
        .meta
        .confidence
        .map(|confidence| format!(" ({confidence:.2})"))
        .unwrap_or_default();
    let properties = event
        .meta
        .properties
        .iter()
        .map(|property| {
            let value = match &property.value {
                EventPropertyValue::Text(text) => text.clone(),
                EventPropertyValue::Unsigned(value) => value.to_string(),
                EventPropertyValue::Float(value) => format!("{value:.2}"),
                EventPropertyValue::Boolean(value) => value.to_string(),
            };
            format!(" {}={value}", property.key)
        })
        .collect::<String>();
    println!(
        "{clock:>18} {marker} {stream:<22} {players}{team}{confidence}{properties}",
        stream = event.meta.stream,
    );
}
