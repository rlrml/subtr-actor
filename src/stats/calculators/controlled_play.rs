use super::*;

const DISTINCT_TOUCH_GAP_SECONDS: f32 = 0.12;
const MAX_TOUCH_CHAIN_GAP_SECONDS: f32 = 2.50;
// Shared with the player_possession span stream's `sustained_control` label so
// the two notions of "deliberate on-ball play" stay in lockstep; the plan is
// for controlled_play to eventually become a projection of labeled
// player_possession spans.
pub(crate) const CLOSE_DISTANCE_3D: f32 = 700.0;
pub(crate) const MIN_CLOSE_DURATION_SECONDS: f32 = 0.75;
pub(crate) const MIN_EPISODE_DURATION_SECONDS: f32 = 1.00;
pub(crate) const MIN_FIRST_TO_LAST_TOUCH_DURATION_SECONDS: f32 = 1.00;
pub(crate) const MIN_TOUCHES: u32 = 2;

/// A span of sustained controlled play with ball-progress metrics.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ControlledPlayEvent {
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub start_frame: usize,
    pub end_frame: usize,
    pub start_time: f32,
    pub end_time: f32,
    pub duration: f32,
    pub first_touch_frame: usize,
    pub last_touch_frame: usize,
    pub first_touch_time: f32,
    pub last_touch_time: f32,
    pub touch_count: u32,
    pub close_duration: f32,
    pub total_advance_distance: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveControlledPlay {
    player_id: PlayerId,
    is_team_0: bool,
    start_frame: usize,
    end_frame: usize,
    start_time: f32,
    end_time: f32,
    first_touch_frame: usize,
    last_touch_frame: usize,
    first_touch_time: f32,
    last_touch_time: f32,
    touch_count: u32,
    close_duration: f32,
    total_advance_distance: f32,
}

impl ActiveControlledPlay {
    fn from_touch(touch: &TouchEvent, player_id: PlayerId) -> Self {
        Self {
            player_id,
            is_team_0: touch.team_is_team_0,
            start_frame: touch.frame,
            end_frame: touch.frame,
            start_time: touch.time,
            end_time: touch.time,
            first_touch_frame: touch.frame,
            last_touch_frame: touch.frame,
            first_touch_time: touch.time,
            last_touch_time: touch.time,
            touch_count: 1,
            close_duration: 0.0,
            total_advance_distance: 0.0,
        }
    }

    fn record_touch(&mut self, touch: &TouchEvent) {
        if touch.time - self.last_touch_time < DISTINCT_TOUCH_GAP_SECONDS {
            return;
        }

        self.touch_count += 1;
        self.last_touch_frame = touch.frame;
        self.last_touch_time = touch.time;
        self.extend_to(touch.frame, touch.time);
    }

    fn extend_to(&mut self, frame: usize, time: f32) {
        if time < self.end_time || (time == self.end_time && frame < self.end_frame) {
            return;
        }
        self.end_frame = frame;
        self.end_time = time;
    }

    fn duration(&self) -> f32 {
        (self.end_time - self.start_time).max(0.0)
    }

    fn touch_span(&self) -> f32 {
        (self.last_touch_time - self.first_touch_time).max(0.0)
    }

    fn is_valid(&self) -> bool {
        self.touch_count >= MIN_TOUCHES
            && self.duration() >= MIN_EPISODE_DURATION_SECONDS
            && self.touch_span() >= MIN_FIRST_TO_LAST_TOUCH_DURATION_SECONDS
            && self.close_duration >= MIN_CLOSE_DURATION_SECONDS
    }

    fn into_event(self) -> ControlledPlayEvent {
        let duration = self.duration();
        ControlledPlayEvent {
            player_id: self.player_id,
            is_team_0: self.is_team_0,
            start_frame: self.start_frame,
            end_frame: self.end_frame,
            start_time: self.start_time,
            end_time: self.end_time,
            duration,
            first_touch_frame: self.first_touch_frame,
            last_touch_frame: self.last_touch_frame,
            first_touch_time: self.first_touch_time,
            last_touch_time: self.last_touch_time,
            touch_count: self.touch_count,
            close_duration: self.close_duration,
            total_advance_distance: self.total_advance_distance,
        }
    }
}

impl InFlightItem for ActiveControlledPlay {
    fn recognition(&self) -> Recognition {
        // Speculative until the chain accumulates enough to be a real
        // controlled play; only then does it count as having "happened".
        Recognition::new(self.start_time, self.start_frame, self.is_valid())
    }

    fn on_boundary(&mut self, boundary: Boundary) -> Disposition {
        if self.is_valid() {
            Disposition::Finalize(FinalizeReason::Boundary(boundary))
        } else {
            Disposition::Discard
        }
    }
}

/// Detects stretches of controlled play from ball/player positions and touches.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ControlledPlayCalculator {
    events: EventStream<ControlledPlayEvent>,
    active: InFlightLedger<ActiveControlledPlay>,
    previous_ball_position: Option<glam::Vec3>,
}

impl ControlledPlayCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[ControlledPlayEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[ControlledPlayEvent] {
        self.events.new_events()
    }

    /// Natural finalization (touch-chain gap or a superseding player): emit the
    /// run if it qualifies, otherwise discard it.
    fn finish_active(&mut self) {
        let valid = self
            .active
            .in_flight()
            .first()
            .is_some_and(ActiveControlledPlay::is_valid);
        if valid {
            for (active, _reason) in self.active.finalize_all(FinalizeReason::Completed) {
                self.events.push(active.into_event());
            }
        } else {
            self.active.clear();
        }
    }

    /// Resolve any in-flight run against a game-flow boundary, emitting it only
    /// if it qualifies (handled uniformly via the ledger).
    fn finish_active_at_boundary(&mut self, boundary: Boundary) {
        for (active, _reason) in self.active.apply_boundary(boundary) {
            self.events.push(active.into_event());
        }
    }

    fn player_is_close(
        players: &PlayerFrameState,
        ball_position: glam::Vec3,
        player_id: &PlayerId,
    ) -> bool {
        players
            .player(player_id)
            .and_then(PlayerSample::position)
            .is_some_and(|player_position| {
                player_position.distance(ball_position) <= CLOSE_DISTANCE_3D
            })
    }

    fn apply_frame_sample(
        &mut self,
        frame: &FrameInfo,
        ball_position: Option<glam::Vec3>,
        players: &PlayerFrameState,
    ) {
        let Some(active) = self.active.in_flight_mut().first_mut() else {
            self.previous_ball_position = ball_position;
            return;
        };
        let Some(ball_position) = ball_position else {
            self.previous_ball_position = None;
            return;
        };

        if Self::player_is_close(players, ball_position, &active.player_id) {
            active.close_duration += frame.dt.max(0.0);
        }

        if let Some(previous_ball_position) = self.previous_ball_position {
            let team_forward_sign = if active.is_team_0 { 1.0 } else { -1.0 };
            let advance_distance = (ball_position.y - previous_ball_position.y) * team_forward_sign;
            active.total_advance_distance += advance_distance.max(0.0);
        }
        active.extend_to(frame.frame_number, frame.time);
        self.previous_ball_position = Some(ball_position);
    }

    fn expire_stale_candidate(&mut self, frame: &FrameInfo) {
        let Some(active) = self.active.in_flight().first() else {
            return;
        };
        if frame.time - active.last_touch_time > MAX_TOUCH_CHAIN_GAP_SECONDS {
            self.finish_active();
        }
    }

    fn apply_touch(&mut self, touch: &TouchEvent) {
        let Some(player_id) = touch.player.clone() else {
            return;
        };

        let same_player = self
            .active
            .in_flight()
            .first()
            .is_some_and(|active| active.player_id == player_id);
        if same_player {
            if let Some(active) = self.active.in_flight_mut().first_mut() {
                active.record_touch(touch);
            }
            return;
        }

        self.finish_active();
        self.active
            .arm(ActiveControlledPlay::from_touch(touch, player_id));
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if !live_play_state.is_live_play {
            self.finish_active_at_boundary(Boundary::LivePlayEnded);
            self.previous_ball_position = ball.position();
            return Ok(());
        }

        self.expire_stale_candidate(frame);
        self.apply_frame_sample(frame, ball.position(), players);
        for touch in chronological_touch_events(&touch_state.touch_events) {
            self.apply_touch(touch);
        }

        Ok(())
    }

    pub fn finish(&mut self) {
        self.finish_active_at_boundary(Boundary::ReplayEnded);
    }
}

#[cfg(test)]
#[path = "controlled_play_tests.rs"]
mod tests;
