use super::*;

/// A span of game time during which a single player held one categorical state of
/// one positioning facet (field third, ball-relative depth, rotation role, ...).
///
/// Every player-facet event stream shares this envelope; only the `state` payload
/// differs per facet. Spans tile the player's tracked time contiguously: a span
/// covers `(time, end_time]` and `duration` is the exact f32 sum of the per-frame
/// (or sub-frame) contributions, so summing `duration` per state reproduces the
/// exported per-state time totals. Frames whose motion crosses a state boundary
/// are split at the crossing point into sub-frame spans instead of being
/// annotated with fraction fields.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerStateSpan<S> {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub duration: f32,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub state: S,
}

fn player_sort_key(player: &PlayerId) -> String {
    format!("{player:?}")
}

/// Builds coalesced [`PlayerStateSpan`] streams for one facet: consecutive
/// same-state contributions per player extend the open span, a state change
/// closes it, and an explicit `close` (the facet stopped applying to the
/// player) ends the span so a later resumption starts fresh.
#[derive(Debug, Clone)]
pub struct PlayerSpanTracker<S> {
    open: HashMap<PlayerId, PlayerStateSpan<S>>,
    closed: EventStream<PlayerStateSpan<S>>,
}

impl<S> Default for PlayerSpanTracker<S> {
    fn default() -> Self {
        Self {
            open: HashMap::new(),
            closed: EventStream::default(),
        }
    }
}

impl<S: Clone + PartialEq> PlayerSpanTracker<S> {
    pub fn begin_update(&mut self) {
        self.closed.begin_update();
    }

    /// Record that `player` held `state` over `(start_time, end_time]` within
    /// `frame_number`, contributing `duration` seconds.
    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &mut self,
        frame_number: usize,
        start_time: f32,
        end_time: f32,
        duration: f32,
        player: &PlayerId,
        player_position: Option<[f32; 3]>,
        is_team_0: bool,
        state: S,
    ) {
        if let Some(open) = self.open.get_mut(player) {
            if open.state == state {
                open.end_time = end_time;
                open.end_frame = frame_number;
                open.duration += duration;
                open.player_position = player_position;
                return;
            }
        }
        let span = PlayerStateSpan {
            time: start_time,
            frame: frame_number,
            end_time,
            end_frame: frame_number,
            duration,
            player: player.clone(),
            player_position,
            is_team_0,
            state,
        };
        if let Some(previous) = self.open.insert(player.clone(), span) {
            self.closed.push(previous);
        }
    }

    pub fn close(&mut self, player: &PlayerId) {
        if let Some(span) = self.open.remove(player) {
            self.closed.push(span);
        }
    }

    pub fn close_all(&mut self) {
        let mut spans: Vec<_> = self.open.drain().map(|(_, span)| span).collect();
        spans.sort_by_key(|span| player_sort_key(&span.player));
        self.closed.extend(spans);
    }

    /// Spans already closed by a state change or facet gap.
    pub fn events(&self) -> &[PlayerStateSpan<S>] {
        self.closed.all()
    }

    /// Spans closed during the current update (since the last `begin_update`).
    pub fn new_events(&self) -> &[PlayerStateSpan<S>] {
        self.closed.new_events()
    }

    /// All spans including still-open ones with their duration so far.
    pub fn projected_events(&self) -> Vec<PlayerStateSpan<S>> {
        let mut events = self.closed.all().to_vec();
        let mut open: Vec<_> = self.open.values().cloned().collect();
        open.sort_by_key(|span| player_sort_key(&span.player));
        events.extend(open);
        events
    }

    /// Like [`projected_events`](Self::projected_events), but each span is
    /// paired with whether it is closed (an open span's end/duration/position
    /// still advance every frame), and the merged list is stable-ordered by
    /// player.
    ///
    /// This ordering is a pure function of the tracker's state and is
    /// invariant across projection cadences: within one player, closed spans
    /// keep their per-player commit order (a player's spans close in the order
    /// they opened) and the at-most-one open span comes last, so the stable
    /// per-player sort yields (player, per-player commit order) whether a span
    /// is observed while still open or only after it closed. The raw merged
    /// order (all closed spans in cross-player close order, then open spans)
    /// does *not* have that property, which is why identity-sensitive
    /// consumers use this accessor.
    pub fn projected_events_by_player(&self) -> Vec<(PlayerStateSpan<S>, bool)> {
        let mut events: Vec<(PlayerStateSpan<S>, bool)> = self
            .closed
            .all()
            .iter()
            .map(|span| (span.clone(), true))
            .collect();
        let mut open: Vec<_> = self
            .open
            .values()
            .map(|span| (span.clone(), false))
            .collect();
        open.sort_by_key(|(span, _)| player_sort_key(&span.player));
        events.extend(open);
        events.sort_by_key(|(span, _)| player_sort_key(&span.player));
        events
    }
}

/// Ordered `(state, fraction)` segments of a frame whose scalar moves linearly
/// from `start` to `end`, classified against the half-open regions delimited by
/// `thresholds` (ascending): region `i` is `[thresholds[i-1], thresholds[i])`.
/// Fractions sum to 1 so `fraction * dt` per segment tiles the frame exactly.
pub(crate) fn scalar_state_segments<S: Copy>(
    start: f32,
    end: f32,
    thresholds: &[f32],
    states: &[S],
) -> Vec<(S, f32)> {
    debug_assert_eq!(states.len(), thresholds.len() + 1);
    let region = |value: f32| -> usize { thresholds.iter().take_while(|&&t| value >= t).count() };
    let start_region = region(start);
    let end_region = region(end);
    if (end - start).abs() <= f32::EPSILON || start_region == end_region {
        return vec![(states[start_region], 1.0)];
    }
    let direction: isize = if end > start { 1 } else { -1 };
    let mut segments = Vec::new();
    let mut current_region = start_region;
    let mut previous_t = 0.0f32;
    while current_region != end_region {
        let crossing = if direction > 0 {
            thresholds[current_region]
        } else {
            thresholds[current_region - 1]
        };
        let t = ((crossing - start) / (end - start)).clamp(0.0, 1.0);
        segments.push((states[current_region], (t - previous_t).max(0.0)));
        previous_t = t;
        current_region = (current_region as isize + direction) as usize;
    }
    segments.push((states[end_region], (1.0 - previous_t).max(0.0)));
    segments
}

#[cfg(test)]
#[path = "player_state_span_tests.rs"]
mod tests;
