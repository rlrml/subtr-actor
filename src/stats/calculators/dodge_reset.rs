use super::*;

// Bounds on the delay between an on-ball reset and the *dodge* that converts it.
// The window is measured from the dodge's onset (its rising edge), not from the
// conversion touch: a flip-into-ball contact is routinely sampled a frame or two
// before the dodge component's active byte replicates, so anchoring on the touch
// would push very fast reset-then-flip conversions under the minimum.
const FLIP_RESET_MIN_DODGE_TOUCH_DELAY_SECONDS: f32 = 0.05;
const FLIP_RESET_MAX_DODGE_TOUCH_DELAY_SECONDS: f32 = 2.0;
const FLIP_RESET_GROUNDED_Z: f32 = 80.0;
const FALLBACK_RESET_MIN_PLAYER_HEIGHT: f32 = 95.0;
const FALLBACK_RESET_MAX_LOCAL_VERTICAL_OFFSET: f32 = 10.0;
const FALLBACK_RESET_MAX_LOCAL_FORWARD_OFFSET: f32 = 240.0;
const FALLBACK_RESET_MAX_LOCAL_LATERAL_OFFSET: f32 = 240.0;
/// A dodge can start while the car is already dragging through the ball, then
/// the replay's active byte can drop before the sampled touch that carries the
/// resulting impulse. Keep a short post-onset continuation window for matching
/// that touch to the pending reset without broadening the global touch-classifier
/// dodge-lag tolerance.
const FLIP_RESET_DODGE_CONTACT_CONTINUATION_SECONDS: f32 = 0.35;
/// How long after a conversion touch the dodge component's active byte may take
/// to replicate. The ball-hit and the dodge activation routinely land on adjacent
/// frames, so a flip-reset conversion touch can be sampled on the frame *before*
/// the dodge flag flips on. We keep the touch around for this brief window so the
/// dodge, once it appears, still confirms the reset retroactively. Shared with the
/// touch classifier and flick detector via
/// [`DODGE_ACTIVE_BYTE_LAG_TOLERANCE_SECONDS`].
const FLIP_RESET_DODGE_TOUCH_LAG_TOLERANCE_SECONDS: f32 = DODGE_ACTIVE_BYTE_LAG_TOLERANCE_SECONDS;

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

/// The most recent attributed touch by a player who has a pending on-ball reset,
/// kept so a dodge whose active byte replicates a frame or two *after* the
/// conversion touch can still confirm the reset retroactively. See
/// [`FLIP_RESET_DODGE_TOUCH_LAG_TOLERANCE_SECONDS`].
#[derive(Debug, Clone, PartialEq)]
struct RecentResetTouch {
    time: f32,
    frame: usize,
    team_is_team_0: bool,
    player_position: Option<[f32; 3]>,
    dodge_contact: bool,
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
    /// Onset time of the post-reset dodge, per player. Presence means a distinct
    /// dodge has started while the reset was pending (so the reset's own
    /// establishing contact cannot masquerade as the conversion); the value
    /// anchors the reset-to-dodge delay window.
    pending_reset_dodge_onset: HashMap<PlayerId, f32>,
    /// Latest attributed touch per player with a pending reset, used to confirm a
    /// reset when the dodge byte replicates after the conversion touch.
    recent_confirmable_touch: HashMap<PlayerId, RecentResetTouch>,
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
        self.clear_pending_reset_tracking(player_id);
        self.record_outcome(pending, outcome, time, frame, time_to_use);
    }

    fn apply_ledger_boundary(&mut self, boundary: Boundary, time: f32, frame: usize) {
        for (player_id, pending, _reason) in self.pending_on_ball_resets.apply_boundary(boundary) {
            self.clear_pending_reset_tracking(&player_id);
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

    fn clear_pending_reset_tracking(&mut self, player_id: &PlayerId) {
        self.pending_reset_dodge_onset.remove(player_id);
        self.recent_confirmable_touch.remove(player_id);
    }

    fn fallback_on_ball_reset(touch_event: &TouchEvent) -> bool {
        if touch_event.player.is_none() || touch_event.dodge_contact {
            return false;
        }
        if touch_event
            .closest_approach_distance
            .is_none_or(|gap| gap > TouchCandidateScoring::DEFAULT.relaxed_contact_gap_threshold)
        {
            return false;
        }
        if touch_event
            .player_position
            .is_none_or(|position| position.z < FALLBACK_RESET_MIN_PLAYER_HEIGHT)
        {
            return false;
        }
        let Some(local_ball_position) = touch_event.contact_local_ball_position else {
            return false;
        };

        local_ball_position[0].abs() <= FALLBACK_RESET_MAX_LOCAL_FORWARD_OFFSET
            && local_ball_position[1].abs() <= FALLBACK_RESET_MAX_LOCAL_LATERAL_OFFSET
            && local_ball_position[2] <= FALLBACK_RESET_MAX_LOCAL_VERTICAL_OFFSET
    }

    fn arm_fallback_on_ball_reset(&mut self, touch_event: &TouchEvent) {
        let Some(player_id) = touch_event.player.as_ref() else {
            return;
        };
        if self.pending_on_ball_resets.contains(player_id) {
            return;
        }
        if !Self::fallback_on_ball_reset(touch_event) {
            return;
        }

        let reset_event = DodgeRefreshedEvent {
            time: touch_event.time,
            frame: touch_event.frame,
            player: player_id.clone(),
            player_position: touch_event
                .player_position
                .map(|position| vec_to_glam(&position).to_array()),
            is_team_0: touch_event.team_is_team_0,
            counter_value: 0,
        };
        let event_index = self.events.all().len();
        self.pending_on_ball_resets.arm(
            player_id.clone(),
            PendingOnBallReset {
                reset: reset_event.clone(),
                event_index,
            },
        );
        self.clear_pending_reset_tracking(player_id);
        self.events.push(DodgeResetEvent {
            time: reset_event.time,
            frame: reset_event.frame,
            player: reset_event.player,
            player_position: reset_event.player_position,
            is_team_0: reset_event.is_team_0,
            counter_value: reset_event.counter_value,
            on_ball: true,
            used: false,
            outcome: None,
            time_to_use: None,
        });
    }

    /// Track dodge rising edges for players with a pending reset. When a dodge
    /// starts we record its onset, then try to confirm against a conversion
    /// touch that arrived a frame or two earlier (the dodge byte lagging the
    /// ball-hit it produced).
    fn update_pending_reset_dodges(&mut self, players: &PlayerFrameState, frame_time: f32) {
        let mut newly_started = Vec::new();
        for player in &players.players {
            let was_dodge_active = self
                .previous_dodge_active
                .insert(player.player_id.clone(), player.dodge_active)
                .unwrap_or(false);
            if player.dodge_active
                && !was_dodge_active
                && self.pending_on_ball_resets.contains(&player.player_id)
            {
                self.pending_reset_dodge_onset
                    .insert(player.player_id.clone(), frame_time);
                newly_started.push(player.player_id.clone());
            }
        }

        for player_id in newly_started {
            let Some(touch) = self.recent_confirmable_touch.get(&player_id).cloned() else {
                continue;
            };
            if frame_time - touch.time > FLIP_RESET_DODGE_TOUCH_LAG_TOLERANCE_SECONDS {
                continue;
            }
            self.confirm_flip_reset(&player_id, &touch, frame_time);
        }
    }

    /// Record an attributed touch as a flip-reset conversion candidate and, when
    /// the toucher is already dodging, confirm the pending reset immediately.
    fn reset_and_touch_are_same_dodge_contact(
        reset_event: &DodgeRefreshedEvent,
        touch: &RecentResetTouch,
    ) -> bool {
        touch.dodge_contact
            && touch.frame == reset_event.frame
            && (touch.time - reset_event.time).abs() <= f32::EPSILON
    }

    fn touch_matches_player_frame(
        touch_event: &TouchEvent,
        player_id: &PlayerId,
        time: f32,
        frame: usize,
    ) -> bool {
        touch_event.player.as_ref() == Some(player_id)
            && touch_event.frame == frame
            && (touch_event.time - time).abs() <= f32::EPSILON
    }

    fn recent_reset_touch(
        players: &PlayerFrameState,
        touch_event: &TouchEvent,
    ) -> Option<RecentResetTouch> {
        let player_id = touch_event.player.as_ref()?;
        let dodge_contact =
            touch_event.dodge_contact || Self::player_dodge_active(players, player_id);
        Some(RecentResetTouch {
            time: touch_event.time,
            frame: touch_event.frame,
            team_is_team_0: touch_event.team_is_team_0,
            player_position: touch_event
                .player_position
                .map(|position| vec_to_glam(&position).to_array())
                .or_else(|| players.player_position(player_id)),
            dodge_contact,
        })
    }

    fn touch_within_dodge_continuation(touch_time: f32, dodge_onset_time: f32) -> bool {
        touch_time >= dodge_onset_time
            && touch_time - dodge_onset_time <= FLIP_RESET_DODGE_CONTACT_CONTINUATION_SECONDS
    }

    fn process_touch_for_flip_reset(
        &mut self,
        players: &PlayerFrameState,
        touch_event: &TouchEvent,
    ) -> bool {
        let Some(player_id) = touch_event.player.as_ref() else {
            return false;
        };
        if !self.pending_on_ball_resets.contains(player_id) {
            return false;
        }

        let Some(touch) = Self::recent_reset_touch(players, touch_event) else {
            return false;
        };
        self.recent_confirmable_touch
            .insert(player_id.clone(), touch.clone());

        // Dodge-then-touch (or same-frame): the dodge byte is already up, so the
        // recorded onset anchors the timing window.
        if let Some(&dodge_onset_time) = self.pending_reset_dodge_onset.get(player_id) {
            if touch.dodge_contact
                || Self::touch_within_dodge_continuation(touch.time, dodge_onset_time)
            {
                let player_id = player_id.clone();
                return self.confirm_flip_reset(&player_id, &touch, dodge_onset_time);
            }
        }

        // Same-frame dodge-on-ball resets have no positive reset-to-dodge delay:
        // the contact both refreshes the dodge and hits the ball during the
        // dodge. Treat that as a zero-latency used reset only when the touch
        // itself carries dodge evidence, so ordinary reset contacts still remain
        // pending.
        if touch.dodge_contact && Self::player_dodge_active(players, player_id) {
            let Some(pending) = self.pending_on_ball_resets.get(player_id) else {
                return false;
            };
            if Self::reset_and_touch_are_same_dodge_contact(&pending.reset, &touch) {
                let player_id = player_id.clone();
                return self.confirm_flip_reset(&player_id, &touch, touch.time);
            }
        }

        false
    }

    /// Confirm a pending on-ball reset as a used flip reset, gating on the
    /// reset-to-dodge-onset delay. The reported latency stays measured to the
    /// conversion touch for continuity with `time_to_use`.
    fn confirm_flip_reset(
        &mut self,
        player_id: &PlayerId,
        touch: &RecentResetTouch,
        dodge_onset_time: f32,
    ) -> bool {
        let Some(pending) = self.pending_on_ball_resets.get(player_id) else {
            return false;
        };
        let reset_event = pending.reset.clone();
        let dodge_delay = dodge_onset_time - reset_event.time;
        let same_frame_dodge_reset =
            Self::reset_and_touch_are_same_dodge_contact(&reset_event, touch);
        let time_since_reset = touch.time - reset_event.time;
        if dodge_delay < FLIP_RESET_MIN_DODGE_TOUCH_DELAY_SECONDS
            && time_since_reset < FLIP_RESET_MIN_DODGE_TOUCH_DELAY_SECONDS
            && !same_frame_dodge_reset
        {
            // The dodge is too close to the reset to be a distinct conversion.
            return false;
        }
        if dodge_delay > FLIP_RESET_MAX_DODGE_TOUCH_DELAY_SECONDS {
            self.resolve_pending(
                player_id,
                FinalizeReason::Completed,
                FlipResetOutcome::Expired,
                touch.time,
                touch.frame,
                None,
            );
            return true;
        }
        if time_since_reset < 0.0 {
            return false;
        }

        self.confirmed_flip_reset_events.push(FlipResetEvent {
            time: touch.time,
            frame: touch.frame,
            reset_time: reset_event.time,
            reset_frame: reset_event.frame,
            player: player_id.clone(),
            player_position: touch.player_position,
            is_team_0: touch.team_is_team_0,
            counter_value: reset_event.counter_value,
            time_since_reset,
        });
        self.resolve_pending(
            player_id,
            FinalizeReason::Completed,
            FlipResetOutcome::Used,
            touch.time,
            touch.frame,
            Some(time_since_reset),
        );
        true
    }

    fn confirm_pending_with_same_frame_touch(
        &mut self,
        players: &PlayerFrameState,
        touches: &[&TouchEvent],
        player_id: &PlayerId,
        time: f32,
        frame: usize,
    ) -> Option<usize> {
        let (touch_index, touch_event) = touches.iter().enumerate().find(|(_, touch)| {
            let dodge_onset_time = self.pending_reset_dodge_onset.get(player_id).copied();
            Self::touch_matches_player_frame(touch, player_id, time, frame)
                && (touch.dodge_contact
                    || Self::player_dodge_active(players, player_id)
                    || dodge_onset_time.is_some_and(|onset| {
                        Self::touch_within_dodge_continuation(touch.time, onset)
                    }))
        })?;
        let touch = Self::recent_reset_touch(players, touch_event)?;
        self.recent_confirmable_touch
            .insert(player_id.clone(), touch.clone());
        let dodge_onset_time = self
            .pending_reset_dodge_onset
            .get(player_id)
            .copied()
            .unwrap_or(touch.time);
        self.confirm_flip_reset(player_id, &touch, dodge_onset_time)
            .then_some(touch_index)
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

        self.update_pending_reset_dodges(players, frame.time);
        let ordered_touches = chronological_touch_events(&touch_state.touch_events);
        let mut consumed_touch_indices = vec![false; ordered_touches.len()];
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
                if let Some(touch_index) = self.confirm_pending_with_same_frame_touch(
                    players,
                    &ordered_touches,
                    &event.player,
                    event.time,
                    event.frame,
                ) {
                    consumed_touch_indices[touch_index] = true;
                }
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
                self.clear_pending_reset_tracking(&event.player);
            }
            self.events.push(event);
        }
        for (touch_index, touch_event) in ordered_touches.iter().enumerate() {
            if consumed_touch_indices[touch_index] {
                continue;
            }
            if !events.dodge_refreshed_counter_available {
                self.arm_fallback_on_ball_reset(touch_event);
            }
            self.process_touch_for_flip_reset(players, touch_event);
        }
        self.prune_pending_resets(frame, players);
        Ok(())
    }

    /// Resolve any flip resets still pending at end of stream as unused
    /// (handled uniformly via the ledger so none are silently dropped).
    pub fn finish(&mut self) {
        let (time, frame) = self.last_frame.unwrap_or((0.0, 0));
        for (player_id, pending, _reason) in self.pending_on_ball_resets.finish() {
            self.clear_pending_reset_tracking(&player_id);
            self.record_outcome(pending, FlipResetOutcome::ReplayEnded, time, frame, None);
        }
    }
}

#[cfg(test)]
#[path = "dodge_reset_tests.rs"]
mod tests;
