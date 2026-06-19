use super::*;

const FLIP_RESET_MIN_DODGE_TOUCH_DELAY_SECONDS: f32 = 0.05;
const FLIP_RESET_MAX_DODGE_TOUCH_DELAY_SECONDS: f32 = 2.0;
const FLIP_RESET_GROUNDED_Z: f32 = 80.0;

/// How a flip reset (an on-ball dodge reset) was ultimately resolved.
///
/// Every on-ball reset resolves into exactly one outcome: it was either used
/// (converted by a dodge-powered touch) or it went unused. A reset that is
/// replaced by a newer reset for the same player before being used counts as
/// unused with the [`Superseded`](Self::Superseded) outcome (no latency is
/// recorded for it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum FlipResetOutcome {
    /// Converted by a dodge-powered touch within the reset-to-touch window.
    Used,
    /// The player landed before dodging into the ball.
    Landed,
    /// A confirming dodge touch arrived only after the reset-to-touch window
    /// had already elapsed.
    Expired,
    /// Replaced by a newer flip reset for the same player before being used.
    Superseded,
    /// Live play ended before the reset was used.
    PlayEnded,
    /// A goal was scored before the reset was used.
    GoalScored,
    /// The replay ended before the reset was used.
    ReplayEnded,
}

impl FlipResetOutcome {
    pub fn is_used(self) -> bool {
        matches!(self, Self::Used)
    }
}

/// A frame-level dodge refresh marked as occurring on the ball (a flip reset).
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DodgeResetEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub counter_value: i32,
    /// Whether the dodge refresh happened on the ball (i.e. this reset is a flip reset).
    pub on_ball: bool,
    /// Whether an on-ball reset (flip reset) was later converted by a dodge-powered
    /// touch. Always `false` for non-`on_ball` resets. Set retroactively once the
    /// confirming touch is observed, so it is meaningful at finish time.
    #[serde(default)]
    pub used: bool,
    /// Final outcome of an on-ball reset (flip reset). Set retroactively when
    /// the reset resolves (used, landed, superseded, or cut off by a game-flow
    /// boundary), so it is meaningful at finish time. Always `None` for
    /// non-`on_ball` resets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<FlipResetOutcome>,
    /// Seconds between the on-ball reset and the dodge-powered touch that used
    /// it. Set retroactively together with `used`; `None` for unused resets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_to_use: Option<f32>,
}

/// Resolution of a flip reset (an on-ball dodge reset), emitted once per reset
/// at the moment its outcome becomes known: either it was used by a
/// dodge-powered touch (with the reset-to-use latency) or it went unused.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FlipResetOutcomeEvent {
    /// Time at which the outcome was resolved (touch time, landing time, or
    /// boundary time).
    pub time: f32,
    pub frame: usize,
    pub reset_time: f32,
    pub reset_frame: usize,
    pub player: PlayerId,
    pub is_team_0: bool,
    pub counter_value: i32,
    pub outcome: FlipResetOutcome,
    /// Seconds between the reset and the confirming dodge touch; `Some` iff
    /// `outcome` is [`FlipResetOutcome::Used`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_to_use: Option<f32>,
}

/// Internal bookkeeping for an on-ball dodge reset awaiting confirmation, including
/// the index of the emitted [`DodgeResetEvent`] so it can be marked `used` later.
#[derive(Debug, Clone, PartialEq)]
struct PendingOnBallReset {
    reset: DodgeRefreshedEvent,
    event_index: usize,
}

impl InFlightItem for PendingOnBallReset {
    fn recognition(&self) -> Recognition {
        // The reset itself has definitely happened; only its outcome (used vs
        // unused) is still pending.
        Recognition::committed(self.reset.time, self.reset.frame)
    }

    fn on_boundary(&mut self, boundary: Boundary) -> Disposition {
        // A reset still pending at a boundary resolves as unused at that
        // boundary rather than being silently dropped.
        Disposition::Finalize(FinalizeReason::Boundary(boundary))
    }
}

/// A flip reset confirmed once later converted by a dodge-powered touch before landing.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FlipResetEvent {
    pub time: f32,
    pub frame: usize,
    pub reset_time: f32,
    pub reset_frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub counter_value: i32,
    pub time_since_reset: f32,
}

/// Detects flip/dodge resets and resolves their outcomes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DodgeResetCalculator {
    events: EventStream<DodgeResetEvent>,
    confirmed_flip_reset_events: EventStream<FlipResetEvent>,
    flip_reset_outcome_events: EventStream<FlipResetOutcomeEvent>,
    pending_on_ball_resets: KeyedInFlightLedger<PlayerId, PendingOnBallReset>,
    pending_reset_dodge_started: HashSet<PlayerId>,
    previous_dodge_active: HashMap<PlayerId, bool>,
    previous_live_play: Option<bool>,
    last_frame: Option<(f32, usize)>,
}

impl DodgeResetCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[DodgeResetEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[DodgeResetEvent] {
        self.events.new_events()
    }

    pub fn confirmed_flip_reset_events(&self) -> &[FlipResetEvent] {
        self.confirmed_flip_reset_events.all()
    }

    pub fn new_confirmed_flip_reset_events(&self) -> &[FlipResetEvent] {
        self.confirmed_flip_reset_events.new_events()
    }

    pub fn flip_reset_outcome_events(&self) -> &[FlipResetOutcomeEvent] {
        self.flip_reset_outcome_events.all()
    }

    pub fn new_flip_reset_outcome_events(&self) -> &[FlipResetOutcomeEvent] {
        self.flip_reset_outcome_events.new_events()
    }

    fn player<'a>(players: &'a PlayerFrameState, player_id: &PlayerId) -> Option<&'a PlayerSample> {
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
    }

    fn player_is_grounded(players: &PlayerFrameState, player_id: &PlayerId) -> bool {
        Self::player(players, player_id)
            .and_then(PlayerSample::position)
            .is_some_and(|position| position.z <= FLIP_RESET_GROUNDED_Z)
    }

    fn player_dodge_active(players: &PlayerFrameState, player_id: &PlayerId) -> bool {
        Self::player(players, player_id).is_some_and(|player| player.dodge_active)
    }

    fn on_ball_dodge_reset(
        ball: &BallFrameState,
        players: &PlayerFrameState,
        player_id: &PlayerId,
    ) -> bool {
        const MIN_PLAYER_HEIGHT: f32 = 95.0;
        const MIN_BALL_HEIGHT: f32 = 80.0;
        const MAX_CENTER_DISTANCE: f32 = 180.0;
        const MAX_LOCAL_VERTICAL_OFFSET: f32 = 140.0;

        let Some(ball) = ball.sample() else {
            return false;
        };
        let Some(player) = Self::player(players, player_id) else {
            return false;
        };
        let Some(player_rigid_body) = &player.rigid_body else {
            return false;
        };

        let ball_position = vec_to_glam(&ball.rigid_body.location);
        let player_position = vec_to_glam(&player_rigid_body.location);
        if player_position.z < MIN_PLAYER_HEIGHT || ball_position.z < MIN_BALL_HEIGHT {
            return false;
        }

        let relative_ball_position = ball_position - player_position;
        let center_distance = relative_ball_position.length();
        if !center_distance.is_finite() || center_distance > MAX_CENTER_DISTANCE {
            return false;
        }

        let player_rotation = quat_to_glam(&player_rigid_body.rotation);
        let local_ball_position = player_rotation.inverse() * relative_ball_position;
        local_ball_position.z <= MAX_LOCAL_VERTICAL_OFFSET
    }

    fn boundary_outcome(boundary: Boundary) -> FlipResetOutcome {
        match boundary {
            Boundary::LivePlayEnded => FlipResetOutcome::PlayEnded,
            Boundary::GoalScored => FlipResetOutcome::GoalScored,
            Boundary::ReplayEnded => FlipResetOutcome::ReplayEnded,
        }
    }

    /// Convert a resolved pending reset into a [`FlipResetOutcomeEvent`] and
    /// patch the originating [`DodgeResetEvent`] with the outcome (and `used`
    /// plus latency when the reset was converted).
    fn record_outcome(
        &mut self,
        pending: PendingOnBallReset,
        outcome: FlipResetOutcome,
        time: f32,
        frame: usize,
        time_to_use: Option<f32>,
    ) {
        if let Some(reset) = self.events.get_mut(pending.event_index) {
            reset.outcome = Some(outcome);
            reset.time_to_use = time_to_use;
            if outcome.is_used() {
                reset.used = true;
            }
        }
        self.flip_reset_outcome_events.push(FlipResetOutcomeEvent {
            time,
            frame,
            reset_time: pending.reset.time,
            reset_frame: pending.reset.frame,
            player: pending.reset.player.clone(),
            is_team_0: pending.reset.is_team_0,
            counter_value: pending.reset.counter_value,
            outcome,
            time_to_use,
        });
    }

    fn resolve_pending(
        &mut self,
        player_id: &PlayerId,
        reason: FinalizeReason,
        outcome: FlipResetOutcome,
        time: f32,
        frame: usize,
        time_to_use: Option<f32>,
    ) {
        let Some(pending) = self.pending_on_ball_resets.finalize(player_id, reason) else {
            return;
        };
        self.pending_reset_dodge_started.remove(player_id);
        self.record_outcome(pending, outcome, time, frame, time_to_use);
    }

    fn apply_ledger_boundary(&mut self, boundary: Boundary, time: f32, frame: usize) {
        for (player_id, pending, _reason) in self.pending_on_ball_resets.apply_boundary(boundary) {
            self.pending_reset_dodge_started.remove(&player_id);
            self.record_outcome(pending, Self::boundary_outcome(boundary), time, frame, None);
        }
    }

    fn prune_pending_resets(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        let grounded_players = self
            .pending_on_ball_resets
            .keys()
            .filter(|player_id| Self::player_is_grounded(players, player_id))
            .cloned()
            .collect::<Vec<_>>();
        for player_id in grounded_players {
            self.resolve_pending(
                &player_id,
                FinalizeReason::Completed,
                FlipResetOutcome::Landed,
                frame.time,
                frame.frame_number,
                None,
            );
        }
    }

    fn update_pending_reset_dodges(&mut self, players: &PlayerFrameState) {
        for player in &players.players {
            let was_dodge_active = self
                .previous_dodge_active
                .insert(player.player_id.clone(), player.dodge_active)
                .unwrap_or(false);
            if player.dodge_active
                && !was_dodge_active
                && self.pending_on_ball_resets.contains(&player.player_id)
            {
                self.pending_reset_dodge_started
                    .insert(player.player_id.clone());
            }
        }
    }

    fn apply_confirmed_flip_reset_touch(
        &mut self,
        players: &PlayerFrameState,
        touch_event: &TouchEvent,
    ) {
        let Some(player_id) = touch_event.player.as_ref() else {
            return;
        };
        if !self.pending_reset_dodge_started.contains(player_id)
            || !Self::player_dodge_active(players, player_id)
        {
            return;
        }

        let Some(pending) = self.pending_on_ball_resets.get(player_id) else {
            return;
        };
        let reset_event = pending.reset.clone();
        let time_since_reset = touch_event.time - reset_event.time;
        if !(FLIP_RESET_MIN_DODGE_TOUCH_DELAY_SECONDS..=FLIP_RESET_MAX_DODGE_TOUCH_DELAY_SECONDS)
            .contains(&time_since_reset)
        {
            if time_since_reset > FLIP_RESET_MAX_DODGE_TOUCH_DELAY_SECONDS {
                self.resolve_pending(
                    player_id,
                    FinalizeReason::Completed,
                    FlipResetOutcome::Expired,
                    touch_event.time,
                    touch_event.frame,
                    None,
                );
            }
            return;
        }

        self.confirmed_flip_reset_events.push(FlipResetEvent {
            time: touch_event.time,
            frame: touch_event.frame,
            reset_time: reset_event.time,
            reset_frame: reset_event.frame,
            player: player_id.clone(),
            player_position: touch_event
                .player_position
                .map(|position| vec_to_glam(&position).to_array())
                .or_else(|| players.player_position(player_id)),
            is_team_0: touch_event.team_is_team_0,
            counter_value: reset_event.counter_value,
            time_since_reset,
        });
        self.resolve_pending(
            player_id,
            FinalizeReason::Completed,
            FlipResetOutcome::Used,
            touch_event.time,
            touch_event.frame,
            Some(time_since_reset),
        );
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        touch_state: &TouchState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        self.confirmed_flip_reset_events.begin_update();
        self.flip_reset_outcome_events.begin_update();
        self.last_frame = Some((frame.time, frame.frame_number));

        if !events.goal_events.is_empty() {
            self.apply_ledger_boundary(Boundary::GoalScored, frame.time, frame.frame_number);
        }
        let live_play_just_ended =
            !live_play_state.is_live_play && self.previous_live_play.unwrap_or(true);
        if live_play_just_ended {
            self.apply_ledger_boundary(Boundary::LivePlayEnded, frame.time, frame.frame_number);
        }
        self.previous_live_play = Some(live_play_state.is_live_play);

        self.prune_pending_resets(frame, players);
        for event in &events.dodge_refreshed_events {
            let on_ball = Self::on_ball_dodge_reset(ball, players, &event.player);
            let reset_event = event.clone();
            let event = DodgeResetEvent {
                time: event.time,
                frame: event.frame,
                player: event.player.clone(),
                player_position: players.player_position(&event.player),
                is_team_0: event.is_team_0,
                counter_value: event.counter_value,
                on_ball,
                used: false,
                outcome: None,
                time_to_use: None,
            };
            if on_ball {
                // A still-pending earlier reset for this player is superseded
                // by the new one and counts as unused (no latency recorded).
                self.resolve_pending(
                    &event.player,
                    FinalizeReason::Superseded,
                    FlipResetOutcome::Superseded,
                    reset_event.time,
                    reset_event.frame,
                    None,
                );
                // Index this event will occupy after the push below, so a later
                // confirming touch can mark it `used`.
                let event_index = self.events.all().len();
                self.pending_on_ball_resets.arm(
                    event.player.clone(),
                    PendingOnBallReset {
                        reset: reset_event,
                        event_index,
                    },
                );
                self.pending_reset_dodge_started.remove(&event.player);
            }
            self.events.push(event);
        }
        self.update_pending_reset_dodges(players);
        for touch_event in chronological_touch_events(&touch_state.touch_events) {
            self.apply_confirmed_flip_reset_touch(players, touch_event);
        }
        Ok(())
    }

    /// Resolve any flip resets still pending at end of stream as unused
    /// (handled uniformly via the ledger so none are silently dropped).
    pub fn finish(&mut self) {
        let (time, frame) = self.last_frame.unwrap_or((0.0, 0));
        for (player_id, pending, _reason) in self.pending_on_ball_resets.finish() {
            self.pending_reset_dodge_started.remove(&player_id);
            self.record_outcome(pending, FlipResetOutcome::ReplayEnded, time, frame, None);
        }
    }
}

#[cfg(test)]
#[path = "dodge_reset_tests.rs"]
mod tests;
