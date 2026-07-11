//! End-to-end loopback test: a real `LiveExportServer` fed a scripted
//! synthetic match, consumed by two independent client/store/driver stacks —
//! one postcard client connected from the start, one JSON client joining
//! mid-stream via the snapshot prologue.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};

use subtr_actor::{Event, EventLifecycle, EventTiming, EventTransaction};
use subtr_actor_live::{
    Encoding, LiveDodgeRefreshedEvent, LiveEventTiming, LiveExportServer, LiveExportServerConfig,
    LiveFrame, LiveGoalEvent, LivePlayerFrame, LiveTouchEvent, ServerMessage, WireEventHistory,
};
use subtr_actor_live_consumer::{DriverOutput, LiveClient, LiveGraphDriver, LiveStateStore};

const FPS: f32 = 30.0;

fn vec3(x: f32, y: f32, z: f32) -> Vector3f {
    Vector3f { x, y, z }
}

fn rigid_body(location: Vector3f, linear_velocity: Vector3f) -> RigidBody {
    RigidBody {
        sleeping: false,
        location,
        rotation: Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(linear_velocity),
        angular_velocity: Some(vec3(0.0, 0.0, 0.0)),
    }
}

fn player(index: u32, is_team_0: bool, location: Vector3f) -> LivePlayerFrame {
    LivePlayerFrame {
        player_index: index,
        name: Some(format!("Player {index}")),
        is_team_0,
        rigid_body: Some(rigid_body(location, vec3(0.0, 100.0, 0.0))),
        boost_amount: 85.0,
        last_boost_amount: 85.0,
        ..LivePlayerFrame::default()
    }
}

/// Base frame: two players slowly advancing, ball far from both so no
/// geometric touches are synthesized unless the script adds explicit events.
fn frame(frame_number: u64) -> LiveFrame {
    let progress = frame_number as f32 * 10.0;
    LiveFrame {
        frame_number,
        time: frame_number as f32 / FPS,
        dt: 1.0 / FPS,
        seconds_remaining: Some(280),
        ball_has_been_hit: Some(true),
        live_play: Some(true),
        team_zero_score: Some(0),
        team_one_score: Some(0),
        ball: Some(rigid_body(vec3(0.0, 4000.0, 92.75), vec3(0.0, 0.0, 0.0))),
        players: vec![
            player(0, true, vec3(-1000.0, -4000.0 + progress, 17.0)),
            player(1, false, vec3(1000.0, 4000.0 - progress, 17.0)),
        ],
        ..LiveFrame::default()
    }
}

const TOUCH_FRAME: u64 = 45;
const GOAL_FRAME: u64 = 75;
/// Long enough past the discrete events that the driver's ~1s event
/// projections publish them well before `MatchEnd`.
const LAST_FRAME: u64 = 149;
/// The late joiner connects after this frame has been observed.
const JOIN_FRAME: u64 = 29;

/// The scripted second half of the match, containing every discrete event:
/// a dodge-refresh + touch, then a goal. All of it happens after
/// `JOIN_FRAME`, so both consumers' graphs see identical discrete inputs.
fn scripted_frame(frame_number: u64) -> LiveFrame {
    let mut frame = frame(frame_number);
    if frame_number >= GOAL_FRAME {
        frame.team_zero_score = Some(1);
    }
    if frame_number == TOUCH_FRAME {
        frame.events.touches.push(LiveTouchEvent {
            timing: LiveEventTiming::default(),
            player: Some(RemoteId::SplitScreen(0)),
            is_team_0: true,
            closest_approach_distance: Some(60.0),
        });
        frame.events.dodge_refreshes.push(LiveDodgeRefreshedEvent {
            timing: LiveEventTiming::default(),
            player: RemoteId::SplitScreen(0),
            is_team_0: true,
            counter_value: 1,
        });
    }
    if frame_number == GOAL_FRAME {
        frame.events.goals.push(LiveGoalEvent {
            timing: LiveEventTiming::default(),
            scoring_team_is_team_0: true,
            player: Some(RemoteId::SplitScreen(0)),
            team_zero_score: Some(1),
            team_one_score: Some(0),
        });
    }
    frame
}

/// One consumer stack plus everything it observed.
struct Consumer {
    client: LiveClient,
    store: LiveStateStore,
    driver: LiveGraphDriver,
    outputs: Vec<DriverOutput>,
    /// History snapshot taken just before each `MatchEnd` clears the store.
    pre_match_end_history: Option<WireEventHistory>,
    /// Output count when the first `MatchEnd` message arrived, separating
    /// events drained live (from interim captures) from the finish drain.
    outputs_before_first_match_end: Option<usize>,
    match_end_count: usize,
}

impl Consumer {
    fn connect(url: &str, encoding: Encoding) -> Self {
        Self {
            client: LiveClient::connect(url, encoding, None).expect("connect"),
            store: LiveStateStore::new(),
            driver: LiveGraphDriver::new(),
            outputs: Vec::new(),
            pre_match_end_history: None,
            outputs_before_first_match_end: None,
            match_end_count: 0,
        }
    }

    /// Reads and applies messages until `stop` matches one (inclusive).
    fn pump_until(&mut self, stop: impl Fn(&ServerMessage) -> bool) {
        loop {
            let message = self
                .client
                .next_message()
                .expect("read message")
                .expect("stream should not close during the test");
            let is_stop = stop(&message);
            if matches!(message, ServerMessage::MatchEnd { .. }) {
                self.pre_match_end_history = Some(self.store.history().clone().into());
                self.outputs_before_first_match_end
                    .get_or_insert(self.outputs.len());
                self.match_end_count += 1;
            }
            let outputs = &mut self.outputs;
            self.driver
                .on_message(&mut self.store, message, &mut |output| outputs.push(output))
                .expect("driver should process every message");
            if is_stop {
                return;
            }
        }
    }

    /// All drained transactions in log order, optionally truncated to those
    /// emitted before the first `MatchEnd` message (i.e. from interim
    /// captures only, excluding the finish drain).
    fn transactions(&self, before_match_end_only: bool) -> Vec<&EventTransaction> {
        let output_count = if before_match_end_only {
            self.outputs_before_first_match_end
                .expect("a MatchEnd message was consumed")
        } else {
            self.outputs.len()
        };
        self.outputs[..output_count]
            .iter()
            .filter_map(|output| match output {
                DriverOutput::EventTransactions(transactions) => Some(transactions.iter()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// The latest upserted version of every event id, replaying every drained
    /// transaction (there are no retracts in a normal match; asserted by the
    /// test body).
    fn reduced_events(&self) -> HashMap<&str, &Event> {
        let mut events: HashMap<&str, &Event> = HashMap::new();
        for transaction in self.transactions(false) {
            match transaction {
                EventTransaction::Upsert { event, .. } => {
                    events.insert(&event.meta.id, event);
                }
                EventTransaction::Retract { id, .. } => {
                    events.remove(id.as_str());
                }
            }
        }
        events
    }
}

fn is_frame(message: &ServerMessage, frame_number: u64) -> bool {
    matches!(
        message,
        ServerMessage::Frame { payload, .. } if payload.frame.frame_number == frame_number
    )
}

#[test]
fn two_consumers_converge_over_a_scripted_match() {
    let handle = LiveExportServer::start(LiveExportServerConfig {
        server_name: "loopback-test".to_owned(),
        // Keep heartbeats flowing so client reads never stall between the
        // test's push batches.
        heartbeat_interval: Duration::from_millis(500),
        ..LiveExportServerConfig::default()
    })
    .expect("server start");
    let url = format!("ws://{}", handle.local_addr());

    // Early consumer: postcard, subscribed before the first frame.
    let mut early = Consumer::connect(&url, Encoding::Postcard);

    for frame_number in 0..=JOIN_FRAME {
        handle.push_frame(scripted_frame(frame_number));
    }
    // Make the join point deterministic: the server has broadcast (and the
    // early consumer has consumed) everything up to JOIN_FRAME before the
    // late consumer subscribes and snapshots.
    early.pump_until(|message| is_frame(message, JOIN_FRAME));

    // Late consumer: JSON, joining mid-stream via the snapshot prologue.
    let mut late = Consumer::connect(&url, Encoding::Json);
    late.pump_until(|message| matches!(message, ServerMessage::EventHistorySnapshot { .. }));
    assert!(
        late.store.latest().is_some(),
        "snapshot should carry the latest frame for a mid-match joiner"
    );
    assert_eq!(
        WireEventHistory::from(late.store.history().clone()),
        WireEventHistory::from(early.store.history().clone()),
        "snapshot history should equal the early consumer's accumulated history"
    );

    for frame_number in (JOIN_FRAME + 1)..=LAST_FRAME {
        handle.push_frame(scripted_frame(frame_number));
    }
    handle.match_end();

    early.pump_until(|message| matches!(message, ServerMessage::MatchEnd { .. }));
    late.pump_until(|message| matches!(message, ServerMessage::MatchEnd { .. }));

    // (a) Both consumers accumulated identical event histories (compared via
    // the PartialEq wire mirror, snapshotted just before MatchEnd reset).
    let early_history = early.pre_match_end_history.clone().expect("match ended");
    let late_history = late.pre_match_end_history.clone().expect("match ended");
    assert_eq!(early_history, late_history);
    assert!(
        !early_history.touch_events.is_empty(),
        "scripted touch should be in the shared history"
    );
    assert_eq!(early_history.goal_events.len(), 1);

    // (b) The early consumer's driver streamed discrete events LIVE — drained
    // from interim projections before the MatchEnd message arrived — including
    // the scripted touch, which arrives as a Confirmed upsert (its outcome
    // enrichment may still upgrade it until finish).
    let early_live_transactions = early.transactions(true);
    assert!(
        !early_live_transactions.is_empty(),
        "early driver should drain timeline transactions before match end"
    );
    assert!(
        early_live_transactions.iter().any(|transaction| matches!(
            transaction,
            EventTransaction::Upsert { event, .. }
                if event.meta.stream == "touch"
                    && event.meta.lifecycle == EventLifecycle::Confirmed
        )),
        "the scripted touch should stream as a Confirmed upsert before match end"
    );

    // No retracts anywhere: nothing legitimately vanishes during a match.
    let early_transactions = early.transactions(false);
    assert!(
        early_transactions
            .iter()
            .all(|transaction| matches!(transaction, EventTransaction::Upsert { .. })),
        "a normal match should never retract an event"
    );

    // Drained transactions carry strictly increasing seqs, in drain order.
    let mut last_seq = None;
    for transaction in &early_transactions {
        assert!(
            last_seq.is_none_or(|last| transaction.seq() > last),
            "transaction seq must be strictly increasing, got {} after {last_seq:?}",
            transaction.seq()
        );
        last_seq = Some(transaction.seq());
    }

    // The finish drain completes the set with goal-derived events and
    // finalizes everything: replaying every transaction leaves only
    // Finalized events.
    let early_events = early.reduced_events();
    let early_streams: HashSet<&str> = early_events
        .values()
        .map(|event| event.meta.stream.as_str())
        .collect();
    assert!(
        early_streams.contains("timeline") || early_streams.contains("goal_context"),
        "goal-derived events should be present, got streams {early_streams:?}"
    );
    assert!(
        early_events
            .values()
            .all(|event| event.meta.lifecycle == EventLifecycle::Finalized),
        "after MatchEnd every received event must be Finalized"
    );

    // (c) The late joiner's drained Moment-timed events (after its first
    // evaluated frame) are a subset of the early consumer's — ids are a
    // deterministic function of calculator state, so both graphs mint the
    // same ids for the same discrete inputs. Two id-stable exclusions apply:
    // span events, whose start frame — and therefore id — legitimately
    // differs when a graph first sees the world mid-match, and initial-state
    // moment events (e.g. boost_respawn), which anchor to whichever frame a
    // graph evaluates first. The script schedules every discrete event well
    // after JOIN_FRAME, so the filtered set is non-empty.
    let late_first_frame = (JOIN_FRAME + 1) as usize;
    let late_events = late.reduced_events();
    assert!(
        !late_events.is_empty(),
        "late driver should drain timeline transactions"
    );
    let late_moment_ids: Vec<&str> = late_events
        .values()
        .filter(|event| {
            matches!(event.meta.timing, EventTiming::Moment { frame, .. } if frame > late_first_frame)
        })
        .map(|event| event.meta.id.as_str())
        .collect();
    assert!(
        !late_moment_ids.is_empty(),
        "the scripted touch/goal should yield moment events for the late joiner"
    );
    for id in &late_moment_ids {
        assert!(
            early_events.contains_key(id),
            "late joiner drained {id} that the early consumer never saw; early ids: {:?}",
            early_events.keys().collect::<Vec<_>>()
        );
    }

    // (d) MatchEnd finished + reset both drivers without error, and the
    // early consumer can process a fresh match afterwards.
    assert_eq!(early.match_end_count, 1);
    assert_eq!(late.match_end_count, 1);
    assert!(
        early
            .outputs
            .iter()
            .any(|output| matches!(output, DriverOutput::MatchEnded)),
        "driver should report the match end"
    );
    assert!(early.store.latest().is_none(), "store resets at match end");

    handle.set_match_context(subtr_actor_live::LiveMatchContext {
        match_guid: Some("second-scripted-match".to_owned()),
        playlist_id: None,
        map_name: Some("second-scripted-map".to_owned()),
    });
    for frame_number in 0..5 {
        handle.push_frame(scripted_frame(frame_number));
    }
    handle.match_end();
    early.pump_until(|message| matches!(message, ServerMessage::MatchEnd { .. }));
    assert_eq!(early.match_end_count, 2);

    handle.shutdown();
}
