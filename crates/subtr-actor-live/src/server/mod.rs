//! WebSocket broadcast server for the live game-state export stream.
//!
//! Designed to run inside a game-injected DLL, so it uses synchronous
//! `tungstenite` and plain `std` threads only — no async runtime — giving a
//! deterministic thread lifecycle:
//!
//! - one **accept** thread (non-blocking listener polled every ~100ms),
//! - one **broadcaster** thread that owns the [`LiveEventGenerator`] and turns
//!   raw ingested [`LiveFrame`]s into protocol messages, and
//! - one thread **per client** that performs the WebSocket handshake, the
//!   `Hello` negotiation, and then pumps its bounded outgoing queue.
//!
//! The host **must call [`ServerHandle::shutdown`] before the DLL unloads**;
//! it joins every thread (worst case bounded by one socket write timeout,
//! currently 5s). [`ServerHandle`] also shuts down on drop as a safety net.
//!
//! # Lock ordering
//!
//! Two lock levels exist: the single shared-state mutex, and each client's
//! queue mutex. The shared lock is always taken *before* a client queue lock,
//! never the reverse, and no lock is ever held across a blocking socket
//! operation (client threads pop from their queue, drop the lock, then write).
//!
//! # Ingest backpressure
//!
//! [`ServerHandle::push_frame`] never blocks: the ingest queue is bounded and
//! drops the *oldest* queued frame on overflow. A dropped raw frame loses its
//! physics samples, but its explicit hook-driven events
//! ([`LiveExplicitEvents`]) are first pinned to the dropped frame's timing and
//! then coalesced into the next queued frame, so events survive backpressure.
//!
//! # Per-client frame downsampling
//!
//! A client's `Hello { max_frame_hz }` caps how often *event-free* `Frame`
//! messages are sent to it. Frames whose derived events are non-empty are
//! always delivered (messages are pre-encoded and shared across clients, so
//! per-client event re-coalescing is not possible; delivering event-carrying
//! frames unconditionally is the documented simplification that guarantees no
//! event loss). Non-`Frame` messages are never dropped by downsampling.

use std::collections::VecDeque;
use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime};

use tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::{Message, WebSocket};

use subtr_actor::PlayerId;

use crate::generator::LiveEventGenerator;
use crate::meta::{LiveMatchContext, LiveMatchMeta};
use crate::model::{LiveEventTiming, LiveExplicitEvents, LiveFrame};
use crate::protocol::{
    ClientMessage, Encoding, FramePayload, PROTOCOL_MAJOR, PROTOCOL_MINOR, ServerMessage,
    protocol_versions_compatible,
};
use crate::wire::{WireEventHistory, WireFrameEventsState};

const ACCEPT_POLL_INTERVAL: Duration = Duration::from_millis(100);
const CLIENT_IDLE_WAIT: Duration = Duration::from_millis(100);
const READ_POLL_TIMEOUT: Duration = Duration::from_millis(1);
const CLOSE_DRAIN_TIMEOUT: Duration = Duration::from_millis(100);
const HELLO_TIMEOUT: Duration = Duration::from_secs(5);
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);
const MIN_HEARTBEAT_INTERVAL: Duration = Duration::from_millis(10);

/// Configuration for [`LiveExportServer::start`].
#[derive(Debug, Clone)]
pub struct LiveExportServerConfig {
    /// Address to bind, e.g. `"127.0.0.1"` or `"0.0.0.0"`.
    pub bind_addr: String,
    /// TCP port; `0` picks an ephemeral port (read it back via
    /// [`ServerHandle::local_addr`]).
    pub port: u16,
    /// Bound on raw frames waiting for the broadcaster; overflow drops the
    /// oldest frame (coalescing its explicit events forward).
    pub max_ingest_frames: usize,
    /// Bound on each client's outgoing message queue; a client that overflows
    /// is disconnected (it reconnects and gets a fresh snapshot).
    pub max_client_queue: usize,
    /// Server name reported in `ServerInfo`.
    pub server_name: String,
    /// Idle interval after which `Heartbeat` messages are broadcast when no
    /// frames are flowing.
    pub heartbeat_interval: Duration,
}

impl Default for LiveExportServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1".to_owned(),
            port: 0,
            max_ingest_frames: 256,
            max_client_queue: 512,
            server_name: "subtr-actor-live".to_owned(),
            heartbeat_interval: Duration::from_secs(2),
        }
    }
}

/// Counters exposed by [`ServerHandle::stats`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerStats {
    /// `Frame` messages broadcast (counted once per frame, not per client).
    pub frames_sent: u64,
    /// Raw frames dropped by ingest backpressure.
    pub frames_dropped: u64,
    /// Currently subscribed clients.
    pub clients: usize,
}

/// Namespace for starting the export server.
pub struct LiveExportServer;

impl LiveExportServer {
    /// Binds the listener eagerly (so port errors surface immediately) and
    /// spawns the accept and broadcaster threads.
    pub fn start(config: LiveExportServerConfig) -> io::Result<ServerHandle> {
        let listener = TcpListener::bind((config.bind_addr.as_str(), config.port))?;
        let local_addr = listener.local_addr()?;
        listener.set_nonblocking(true)?;
        let core = Arc::new(ServerCore::new(config));

        let accept_core = core.clone();
        let accept_thread = std::thread::Builder::new()
            .name("subtr-actor-live-accept".to_owned())
            .spawn(move || accept_loop(&accept_core, &listener))?;

        let broadcast_core = core.clone();
        let broadcaster_thread = match std::thread::Builder::new()
            .name("subtr-actor-live-broadcast".to_owned())
            .spawn(move || broadcaster_loop(&broadcast_core))
        {
            Ok(handle) => handle,
            Err(error) => {
                core.shutdown.store(true, Ordering::SeqCst);
                let _ = accept_thread.join();
                return Err(error);
            }
        };

        Ok(ServerHandle {
            core,
            local_addr,
            accept_thread: Mutex::new(Some(accept_thread)),
            broadcaster_thread: Mutex::new(Some(broadcaster_thread)),
        })
    }
}

/// Handle owned by the host (game plugin); all methods are cheap and safe to
/// call from the game thread.
pub struct ServerHandle {
    core: Arc<ServerCore>,
    local_addr: SocketAddr,
    accept_thread: Mutex<Option<JoinHandle<()>>>,
    broadcaster_thread: Mutex<Option<JoinHandle<()>>>,
}

impl ServerHandle {
    /// The actually-bound address (resolves port `0`).
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Enqueues one sampled frame for processing and broadcast.
    ///
    /// Never blocks beyond a short mutex hold: the ingest queue is bounded
    /// with drop-oldest semantics (see the module docs for how dropped frames'
    /// explicit events are coalesced forward).
    pub fn push_frame(&self, frame: LiveFrame) {
        if self.core.shutdown.load(Ordering::SeqCst) {
            return;
        }
        let mut frame = frame;
        {
            let mut ingest = lock(&self.core.ingest);
            if ingest.items.len() >= self.core.config.max_ingest_frames {
                drop_oldest_frame(&mut ingest, &mut frame);
                self.core.frames_dropped.fetch_add(1, Ordering::Relaxed);
            }
            ingest.items.push_back(IngestItem::Frame(Box::new(frame)));
        }
        self.core.ingest_signal.notify_all();
    }

    /// Sets (or replaces) the match-level context merged into the broadcast
    /// [`LiveMatchMeta`].
    ///
    /// The context flows through the ingest queue so it stays ordered with
    /// frames. When a match is live, the updated meta is re-broadcast as a
    /// `RosterChange`; otherwise the context is held and attached to the
    /// eventual `MatchStart`. [`ServerHandle::match_end`] clears it.
    pub fn set_match_context(&self, context: LiveMatchContext) {
        if self.core.shutdown.load(Ordering::SeqCst) {
            return;
        }
        lock(&self.core.ingest)
            .items
            .push_back(IngestItem::MatchContext(context));
        self.core.ingest_signal.notify_all();
    }

    /// Broadcasts `MatchEnd` (if a match was live) and resets the event
    /// generator, history, and roster for the next match.
    ///
    /// The message sequence counter deliberately keeps increasing across
    /// matches so connected clients always observe a strictly increasing
    /// stream.
    pub fn match_end(&self) {
        if self.core.shutdown.load(Ordering::SeqCst) {
            return;
        }
        lock(&self.core.ingest)
            .items
            .push_back(IngestItem::MatchEnd);
        self.core.ingest_signal.notify_all();
    }

    /// Currently subscribed client count (relaxed atomic read).
    pub fn client_count(&self) -> usize {
        self.core.client_count.load(Ordering::Relaxed)
    }

    pub fn stats(&self) -> ServerStats {
        ServerStats {
            frames_sent: self.core.frames_sent.load(Ordering::Relaxed),
            frames_dropped: self.core.frames_dropped.load(Ordering::Relaxed),
            clients: self.client_count(),
        }
    }

    /// Stops accepting, disconnects all clients, and joins every server
    /// thread. Idempotent. **Must** be called before the hosting DLL unloads.
    pub fn shutdown(&self) {
        self.core.shutdown.store(true, Ordering::SeqCst);
        self.core.ingest_signal.notify_all();
        let clients: Vec<Arc<ClientSlot>> = lock(&self.core.shared).clients.clone();
        for client in clients {
            client.disconnect.store(true, Ordering::SeqCst);
            client.signal.notify_all();
        }
        if let Some(handle) = lock(&self.accept_thread).take() {
            let _ = handle.join();
        }
        if let Some(handle) = lock(&self.broadcaster_thread).take() {
            let _ = handle.join();
        }
        // The accept thread is joined, so no new client threads can appear;
        // the loop is belt-and-braces against races with it.
        loop {
            let handles = std::mem::take(&mut *lock(&self.core.client_threads));
            if handles.is_empty() {
                break;
            }
            for handle in handles {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

type RosterSignature = Vec<(PlayerId, bool, Option<String>)>;

struct ServerCore {
    config: LiveExportServerConfig,
    shutdown: AtomicBool,
    ingest: Mutex<IngestQueue>,
    ingest_signal: Condvar,
    shared: Mutex<SharedState>,
    client_count: AtomicUsize,
    frames_sent: AtomicU64,
    frames_dropped: AtomicU64,
    next_client_id: AtomicU64,
    client_threads: Mutex<Vec<JoinHandle<()>>>,
}

impl ServerCore {
    fn new(config: LiveExportServerConfig) -> Self {
        Self {
            config,
            shutdown: AtomicBool::new(false),
            ingest: Mutex::new(IngestQueue::default()),
            ingest_signal: Condvar::new(),
            shared: Mutex::new(SharedState::default()),
            client_count: AtomicUsize::new(0),
            frames_sent: AtomicU64::new(0),
            frames_dropped: AtomicU64::new(0),
            next_client_id: AtomicU64::new(0),
            client_threads: Mutex::new(Vec::new()),
        }
    }
}

#[derive(Default)]
struct IngestQueue {
    items: VecDeque<IngestItem>,
}

enum IngestItem {
    Frame(Box<LiveFrame>),
    MatchContext(LiveMatchContext),
    MatchEnd,
}

#[derive(Default)]
struct SharedState {
    seq: u64,
    match_live: bool,
    roster: Option<LiveMatchMeta>,
    roster_signature: Option<RosterSignature>,
    /// Host-supplied match context, attached to every broadcast meta. Cleared
    /// on `MatchEnd`.
    context: LiveMatchContext,
    history: WireEventHistory,
    latest_frame: Option<FramePayload>,
    clients: Vec<Arc<ClientSlot>>,
}

struct ClientSlot {
    id: u64,
    encoding: Encoding,
    /// Minimum spacing between event-free `Frame` messages, from
    /// `Hello { max_frame_hz }`.
    min_frame_interval: Option<Duration>,
    queue: Mutex<ClientQueue>,
    signal: Condvar,
    disconnect: AtomicBool,
}

#[derive(Default)]
struct ClientQueue {
    messages: VecDeque<Message>,
    last_frame_enqueued: Option<Instant>,
}

struct Subscription {
    encoding: Encoding,
    max_frame_hz: Option<f32>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn wait_timeout<'a, T>(
    condvar: &Condvar,
    guard: MutexGuard<'a, T>,
    timeout: Duration,
) -> MutexGuard<'a, T> {
    match condvar.wait_timeout(guard, timeout) {
        Ok((guard, _)) => guard,
        Err(poisoned) => poisoned.into_inner().0,
    }
}

fn is_timeout_io_error(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    )
}

fn unix_time_seconds() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs_f64())
        .unwrap_or(0.0)
}

/// Fills each explicit event's missing timing fields from its enclosing frame
/// so the events stay accurate after being coalesced into a later frame.
fn pin_explicit_event_timing(frame: &mut LiveFrame) {
    let frame_and_time = (frame.frame_number, frame.time);
    let seconds_remaining = frame.seconds_remaining;
    let pin = |timing: &mut LiveEventTiming| {
        if timing.frame_and_time.is_none() {
            timing.frame_and_time = Some(frame_and_time);
        }
        if timing.seconds_remaining.is_none() {
            timing.seconds_remaining = seconds_remaining;
        }
    };
    let events = &mut frame.events;
    events.touches.iter_mut().for_each(|e| pin(&mut e.timing));
    events
        .dodge_refreshes
        .iter_mut()
        .for_each(|e| pin(&mut e.timing));
    events
        .boost_pad_events
        .iter_mut()
        .for_each(|e| pin(&mut e.timing));
    events.goals.iter_mut().for_each(|e| pin(&mut e.timing));
    events
        .player_stat_events
        .iter_mut()
        .for_each(|e| pin(&mut e.timing));
    events
        .demolishes
        .iter_mut()
        .for_each(|e| pin(&mut e.timing));
}

/// Prepends `older`'s events onto `newer`, preserving chronological order.
fn merge_explicit_events(older: LiveExplicitEvents, newer: &mut LiveExplicitEvents) {
    fn prepend<T>(mut older: Vec<T>, newer: &mut Vec<T>) {
        older.append(newer);
        *newer = older;
    }
    prepend(older.touches, &mut newer.touches);
    prepend(older.dodge_refreshes, &mut newer.dodge_refreshes);
    prepend(older.boost_pad_events, &mut newer.boost_pad_events);
    prepend(older.goals, &mut newer.goals);
    prepend(older.player_stat_events, &mut newer.player_stat_events);
    prepend(older.demolishes, &mut newer.demolishes);
}

/// Drops the oldest queued frame, coalescing its explicit events into the next
/// queued frame (or into `incoming` when none is queued). Events are discarded
/// only when a `MatchEnd` boundary separates the dropped frame from every
/// later frame — they belonged to the ended match.
fn drop_oldest_frame(ingest: &mut IngestQueue, incoming: &mut LiveFrame) {
    let Some(dropped_index) = ingest
        .items
        .iter()
        .position(|item| matches!(item, IngestItem::Frame(_)))
    else {
        return;
    };
    let Some(IngestItem::Frame(mut dropped)) = ingest.items.remove(dropped_index) else {
        return;
    };
    pin_explicit_event_timing(&mut dropped);
    let dropped_events = std::mem::take(&mut dropped.events);

    // The next `Frame` receives the coalesced events (`MatchContext` items are
    // transparent to event coalescing), while a `MatchEnd` is a boundary — the
    // dropped frame belonged to the ended match, so its events are discarded
    // rather than leaked into the next match.
    let mut successor_index = dropped_index;
    loop {
        match ingest.items.get_mut(successor_index) {
            Some(IngestItem::Frame(next)) => {
                merge_explicit_events(dropped_events, &mut next.events);
                return;
            }
            Some(IngestItem::MatchContext(_)) => successor_index += 1,
            Some(IngestItem::MatchEnd) => return,
            None => {
                merge_explicit_events(dropped_events, &mut incoming.events);
                return;
            }
        }
    }
}

enum BroadcasterWake {
    Item(IngestItem),
    HeartbeatDue,
    Shutdown,
}

fn wait_for_ingest(core: &ServerCore, last_activity: Instant) -> BroadcasterWake {
    let mut ingest = lock(&core.ingest);
    loop {
        if core.shutdown.load(Ordering::SeqCst) {
            return BroadcasterWake::Shutdown;
        }
        if let Some(item) = ingest.items.pop_front() {
            return BroadcasterWake::Item(item);
        }
        let elapsed = last_activity.elapsed();
        // Clamp so a zero/tiny configured interval cannot busy-loop the
        // broadcaster.
        let interval = core.config.heartbeat_interval.max(MIN_HEARTBEAT_INTERVAL);
        if elapsed >= interval {
            return BroadcasterWake::HeartbeatDue;
        }
        ingest = wait_timeout(&core.ingest_signal, ingest, interval - elapsed);
    }
}

fn broadcaster_loop(core: &Arc<ServerCore>) {
    // The generator is owned by this thread: it must see every frame exactly
    // once, in order, which is why frames flow through the ingest queue rather
    // than being processed on the caller's (game) thread.
    let mut generator = LiveEventGenerator::default();
    let mut last_activity = Instant::now();
    loop {
        match wait_for_ingest(core, last_activity) {
            BroadcasterWake::Shutdown => return,
            BroadcasterWake::HeartbeatDue => {
                last_activity = Instant::now();
                let mut state = lock(&core.shared);
                state.seq += 1;
                let message = ServerMessage::Heartbeat {
                    seq: state.seq,
                    time: unix_time_seconds(),
                };
                broadcast(core, &state.clients, &message, None);
            }
            BroadcasterWake::Item(IngestItem::Frame(frame)) => {
                last_activity = Instant::now();
                process_frame(core, &mut generator, *frame);
            }
            BroadcasterWake::Item(IngestItem::MatchContext(context)) => {
                apply_match_context(core, context);
            }
            BroadcasterWake::Item(IngestItem::MatchEnd) => {
                generator = LiveEventGenerator::default();
                let mut state = lock(&core.shared);
                if state.match_live {
                    state.seq += 1;
                    let message = ServerMessage::MatchEnd { seq: state.seq };
                    broadcast(core, &state.clients, &message, None);
                }
                state.match_live = false;
                state.roster = None;
                state.roster_signature = None;
                state.context = LiveMatchContext::default();
                state.history = WireEventHistory::default();
                state.latest_frame = None;
            }
        }
    }
}

/// Stores host-supplied match context and, when a match is live, re-broadcasts
/// the roster meta (now carrying the new context) as a `RosterChange`.
fn apply_match_context(core: &Arc<ServerCore>, context: LiveMatchContext) {
    let mut state = lock(&core.shared);
    state.context = context;
    if !state.match_live {
        return;
    }
    let context = state.context.clone();
    let Some(roster) = state.roster.as_mut() else {
        return;
    };
    if roster.context == context {
        return;
    }
    roster.context = context;
    let meta = roster.clone();
    state.seq += 1;
    let message = ServerMessage::RosterChange {
        seq: state.seq,
        meta,
    };
    broadcast(core, &state.clients, &message, None);
}

fn process_frame(core: &Arc<ServerCore>, generator: &mut LiveEventGenerator, frame: LiveFrame) {
    // Derive events outside the shared lock; only state updates and fan-out
    // happen under it.
    let (frame_events, live_play) = generator.frame_events(&frame);
    let mut meta = LiveMatchMeta::from_player_frames(&frame.players);
    let derived_events: WireFrameEventsState = frame_events.into();
    let has_events = derived_events.has_discrete_events();

    let mut state = lock(&core.shared);
    meta.context = state.context.clone();
    if !meta.players.is_empty() {
        let signature = meta.signature();
        if !state.match_live {
            state.match_live = true;
            state.roster_signature = Some(signature);
            state.roster = Some(meta.clone());
            state.seq += 1;
            let message = ServerMessage::MatchStart {
                seq: state.seq,
                meta,
            };
            broadcast(core, &state.clients, &message, None);
        } else if state.roster_signature.as_ref() != Some(&signature) {
            state.roster_signature = Some(signature);
            state.roster = Some(meta.clone());
            state.seq += 1;
            let message = ServerMessage::RosterChange {
                seq: state.seq,
                meta,
            };
            broadcast(core, &state.clients, &message, None);
        }
    }
    state.history.append_frame_events(&derived_events);
    let payload = FramePayload {
        frame,
        derived_events,
        live_play,
    };
    state.latest_frame = Some(payload.clone());
    state.seq += 1;
    let message = ServerMessage::Frame {
        seq: state.seq,
        payload: Box::new(payload),
    };
    broadcast(core, &state.clients, &message, Some(has_events));
    core.frames_sent.fetch_add(1, Ordering::Relaxed);
}

fn encode_ws_message(message: &ServerMessage, encoding: Encoding) -> Option<Message> {
    let bytes = message.encode(encoding).ok()?;
    Some(match encoding {
        Encoding::Postcard => Message::binary(bytes),
        Encoding::Json => Message::text(String::from_utf8(bytes).ok()?),
    })
}

/// Fans a message out to all clients, encoding at most once per encoding in
/// use. `frame_has_events` is `Some` only for `Frame` messages and drives the
/// per-client downsampling policy.
fn broadcast(
    core: &ServerCore,
    clients: &[Arc<ClientSlot>],
    message: &ServerMessage,
    frame_has_events: Option<bool>,
) {
    let mut cache: [Option<Option<Message>>; 2] = [None, None];
    for slot in clients {
        if slot.disconnect.load(Ordering::Relaxed) {
            continue;
        }
        let cache_index = match slot.encoding {
            Encoding::Postcard => 0,
            Encoding::Json => 1,
        };
        let encoded = cache[cache_index]
            .get_or_insert_with(|| encode_ws_message(message, slot.encoding))
            .clone();
        let Some(ws_message) = encoded else {
            continue;
        };
        enqueue(core, slot, ws_message, frame_has_events);
    }
}

fn enqueue(core: &ServerCore, slot: &ClientSlot, message: Message, frame_has_events: Option<bool>) {
    let mut queue = lock(&slot.queue);
    if let Some(has_events) = frame_has_events
        && let Some(interval) = slot.min_frame_interval
    {
        let now = Instant::now();
        if !has_events
            && queue
                .last_frame_enqueued
                .is_some_and(|last| now.duration_since(last) < interval)
        {
            return;
        }
        queue.last_frame_enqueued = Some(now);
    }
    if queue.messages.len() >= core.config.max_client_queue {
        // Overflowing (slow) client: disconnect it; on reconnect it receives a
        // fresh snapshot, which is cheaper and more correct than silently
        // dropping arbitrary mid-stream messages.
        drop(queue);
        slot.disconnect.store(true, Ordering::SeqCst);
        slot.signal.notify_all();
        return;
    }
    queue.messages.push_back(message);
    drop(queue);
    slot.signal.notify_all();
}

fn accept_loop(core: &Arc<ServerCore>, listener: &TcpListener) {
    while !core.shutdown.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _peer)) => {
                // Accepted sockets should be blocking regardless of the
                // listener's non-blocking mode.
                if stream.set_nonblocking(false).is_err() {
                    continue;
                }
                let client_core = core.clone();
                let spawned = std::thread::Builder::new()
                    .name("subtr-actor-live-client".to_owned())
                    .spawn(move || client_thread(&client_core, stream));
                if let Ok(handle) = spawned {
                    lock(&core.client_threads).push(handle);
                }
            }
            Err(error) if is_timeout_io_error(&error) => {
                std::thread::sleep(ACCEPT_POLL_INTERVAL);
            }
            Err(_) => {
                std::thread::sleep(ACCEPT_POLL_INTERVAL);
            }
        }
    }
}

fn client_thread(core: &Arc<ServerCore>, stream: TcpStream) {
    if stream.set_read_timeout(Some(HELLO_TIMEOUT)).is_err()
        || stream.set_write_timeout(Some(WRITE_TIMEOUT)).is_err()
    {
        return;
    }
    let mut format_json_query = false;
    let accepted = tungstenite::accept_hdr(stream, |request: &Request, response: Response| {
        format_json_query = request
            .uri()
            .query()
            .is_some_and(|query| query.split('&').any(|pair| pair == "format=json"));
        Ok::<_, ErrorResponse>(response)
    });
    let Ok(mut ws) = accepted else {
        return;
    };

    let subscription = if format_json_query {
        // `GET /?format=json` subscribes without a Hello, for zero-code
        // consumers (browsers, curl-adjacent tools).
        Some(Subscription {
            encoding: Encoding::Json,
            max_frame_hz: None,
        })
    } else {
        negotiate_hello(&mut ws)
    };
    let Some(subscription) = subscription else {
        return;
    };
    if ws
        .get_ref()
        .set_read_timeout(Some(READ_POLL_TIMEOUT))
        .is_err()
    {
        return;
    }

    let min_frame_interval = subscription
        .max_frame_hz
        .filter(|hz| hz.is_finite() && *hz > 0.0)
        .map(|hz| Duration::from_secs_f32(1.0 / hz));
    let slot = Arc::new(ClientSlot {
        id: core.next_client_id.fetch_add(1, Ordering::Relaxed),
        encoding: subscription.encoding,
        min_frame_interval,
        queue: Mutex::new(ClientQueue::default()),
        signal: Condvar::new(),
        disconnect: AtomicBool::new(false),
    });
    subscribe(core, &slot);
    core.client_count.fetch_add(1, Ordering::Relaxed);

    pump_client(core, &slot, &mut ws);

    {
        let mut state = lock(&core.shared);
        state.clients.retain(|client| client.id != slot.id);
    }
    core.client_count.fetch_sub(1, Ordering::Relaxed);
    let _ = ws.close(None);
    drain_close(&mut ws);
}

/// Registers a client and enqueues its subscription prologue atomically under
/// the shared-state lock, so there is no gap or overlap with the live stream:
/// `ServerInfo`, then `MatchStart` (if a match is live), then
/// `EventHistorySnapshot`, then registration for fan-out.
fn subscribe(core: &Arc<ServerCore>, slot: &Arc<ClientSlot>) {
    let mut state = lock(&core.shared);
    state.seq += 1;
    let info = ServerMessage::ServerInfo {
        protocol_major: PROTOCOL_MAJOR,
        protocol_minor: PROTOCOL_MINOR,
        server: core.config.server_name.clone(),
        seq: state.seq,
    };
    enqueue_for(core, slot, &info);
    if state.match_live
        && let Some(roster) = state.roster.clone()
    {
        state.seq += 1;
        let message = ServerMessage::MatchStart {
            seq: state.seq,
            meta: roster,
        };
        enqueue_for(core, slot, &message);
    }
    state.seq += 1;
    let snapshot = ServerMessage::EventHistorySnapshot {
        seq: state.seq,
        history: state.history.clone(),
        latest_frame: state.latest_frame.clone().map(Box::new),
    };
    enqueue_for(core, slot, &snapshot);
    state.clients.push(slot.clone());
}

fn enqueue_for(core: &ServerCore, slot: &ClientSlot, message: &ServerMessage) {
    if let Some(ws_message) = encode_ws_message(message, slot.encoding) {
        enqueue(core, slot, ws_message, None);
    }
}

/// Reads the initial `Hello` (always JSON text) and validates the protocol
/// version, closing the socket with a descriptive reason on failure.
fn negotiate_hello(ws: &mut WebSocket<TcpStream>) -> Option<Subscription> {
    loop {
        let message = match ws.read() {
            Ok(message) => message,
            Err(_) => return None,
        };
        let text = match message {
            Message::Text(text) => text,
            Message::Ping(_) | Message::Pong(_) => continue,
            Message::Close(_) => return None,
            _ => {
                close_with_reason(ws, "expected a JSON text Hello message");
                return None;
            }
        };
        let hello = match ClientMessage::decode_json(text.as_str()) {
            Ok(hello) => hello,
            Err(_) => {
                close_with_reason(ws, "malformed Hello message");
                return None;
            }
        };
        let ClientMessage::Hello {
            protocol_major,
            protocol_minor,
            encoding,
            max_frame_hz,
        } = hello;
        if !protocol_versions_compatible(encoding, protocol_major, protocol_minor) {
            close_with_reason(
                ws,
                &format!(
                    "unsupported protocol version {protocol_major}.{protocol_minor}; \
                     server speaks {PROTOCOL_MAJOR}.{PROTOCOL_MINOR} \
                     (postcard requires an exact minor match)"
                ),
            );
            return None;
        }
        return Some(Subscription {
            encoding,
            max_frame_hz,
        });
    }
}

fn close_with_reason(ws: &mut WebSocket<TcpStream>, reason: &str) {
    let _ = ws.close(Some(CloseFrame {
        code: CloseCode::Protocol,
        reason: reason.to_owned().into(),
    }));
    drain_close(ws);
}

/// Best-effort drive of the close handshake so the close frame flushes before
/// the socket drops; bounded to a handful of short reads.
fn drain_close(ws: &mut WebSocket<TcpStream>) {
    let _ = ws.get_ref().set_read_timeout(Some(CLOSE_DRAIN_TIMEOUT));
    for _ in 0..10 {
        match ws.read() {
            Ok(_) => {}
            Err(tungstenite::Error::Io(error)) if is_timeout_io_error(&error) => return,
            Err(_) => return,
        }
    }
}

fn pump_client(core: &Arc<ServerCore>, slot: &Arc<ClientSlot>, ws: &mut WebSocket<TcpStream>) {
    loop {
        if core.shutdown.load(Ordering::SeqCst) || slot.disconnect.load(Ordering::SeqCst) {
            return;
        }
        let batch: Vec<Message> = {
            let mut queue = lock(&slot.queue);
            if queue.messages.is_empty() {
                queue = wait_timeout(&slot.signal, queue, CLIENT_IDLE_WAIT);
            }
            queue.messages.drain(..).collect()
        };
        // The queue lock is dropped before any socket write.
        for message in batch {
            if ws.send(message).is_err() {
                return;
            }
        }
        // Poll the read side (1ms socket timeout) to consume pings and detect
        // client-initiated close.
        match ws.read() {
            Ok(Message::Close(_)) => return,
            Ok(_) => {
                // Ping replies are queued internally by tungstenite; push them
                // out now.
                if ws.flush().is_err() {
                    return;
                }
            }
            Err(tungstenite::Error::Io(error)) if is_timeout_io_error(&error) => {}
            Err(_) => return,
        }
    }
}

#[cfg(test)]
#[path = "server_tests.rs"]
mod tests;
