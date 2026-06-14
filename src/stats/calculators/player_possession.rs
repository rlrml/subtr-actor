use super::*;

/// How long a player's possession may be interrupted (pending turnover,
/// contested challenge, brief neutral window) before the span is finalized
/// instead of resumed when the same player re-establishes control.
const PLAYER_POSSESSION_MERGE_GAP_SECONDS: f32 = 2.0;
/// Minimum spacing between touches for them to count as distinct touches
/// within a possession span (mirrors the controlled-play touch chaining).
const DISTINCT_TOUCH_GAP_SECONDS: f32 = 0.12;

/// A contiguous single-player possession span, merged across field-third
/// changes and brief contested interruptions, enriched with the touch, ball
/// movement, and sustained-control activity that happened while the player
/// owned the ball.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerPossessionEvent {
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub start_frame: usize,
    pub end_frame: usize,
    pub start_time: f32,
    pub end_time: f32,
    /// Seconds the player actually held possession. Excludes contested gap
    /// time inside a merged span, so it can be less than `end_time - start_time`.
    pub duration: f32,
    pub touch_count: u32,
    pub aerial_touch_count: u32,
    pub wall_touch_count: u32,
    /// Ball travel toward the opponent goal while the player had possession (uu).
    pub advance_distance: f32,
    /// Ball travel toward the player's own goal while the player had possession (uu).
    pub retreat_distance: f32,
    /// Seconds of the span spent in a grounded ball-carry sample.
    pub carry_time: f32,
    /// Seconds of the span spent in an air-dribble sample.
    pub air_dribble_time: f32,
    pub carry_count: u32,
    pub air_dribble_count: u32,
    /// Seconds of the span the owner spent within close range of the ball
    /// (same proximity signal as controlled play).
    pub close_time: f32,
    /// True when the span meets the controlled-play qualifying criteria
    /// (touch count, possessed duration, first-to-last touch span, and close
    /// time). Controlled play is conceptually a labeled subset of player
    /// possession; this label lets consumers treat it that way.
    pub sustained_control: bool,
    pub start_field_third: Option<String>,
    pub end_field_third: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct ActivePlayerPossession {
    player_id: PlayerId,
    is_team_0: bool,
    start_frame: usize,
    end_frame: usize,
    start_time: f32,
    end_time: f32,
    duration: f32,
    touch_count: u32,
    aerial_touch_count: u32,
    wall_touch_count: u32,
    first_touch_time: Option<f32>,
    last_touch_time: Option<f32>,
    close_time: f32,
    advance_distance: f32,
    retreat_distance: f32,
    carry_time: f32,
    air_dribble_time: f32,
    carry_count: u32,
    air_dribble_count: u32,
    last_carry_kind: Option<BallCarryKind>,
    start_field_third: Option<String>,
    end_field_third: Option<String>,
}

impl ActivePlayerPossession {
    fn open(
        frame: &FrameInfo,
        player_id: PlayerId,
        is_team_0: bool,
        field_third: Option<String>,
    ) -> Self {
        Self {
            player_id,
            is_team_0,
            // The opening frame's dt is credited to the span, so the span
            // window starts one frame earlier (mirrors continuous ball
            // control sequences).
            start_frame: frame.frame_number.saturating_sub(1),
            end_frame: frame.frame_number,
            start_time: (frame.time - frame.dt).max(0.0),
            end_time: frame.time,
            duration: 0.0,
            touch_count: 0,
            aerial_touch_count: 0,
            wall_touch_count: 0,
            first_touch_time: None,
            last_touch_time: None,
            close_time: 0.0,
            advance_distance: 0.0,
            retreat_distance: 0.0,
            carry_time: 0.0,
            air_dribble_time: 0.0,
            carry_count: 0,
            air_dribble_count: 0,
            last_carry_kind: None,
            start_field_third: field_third.clone(),
            end_field_third: field_third,
        }
    }

    fn record_frame(&mut self, frame: &FrameInfo, field_third: Option<String>) {
        self.duration += frame.dt.max(0.0);
        self.end_frame = frame.frame_number;
        self.end_time = frame.time;
        if field_third.is_some() {
            self.end_field_third = field_third;
        }
    }

    fn record_touch(&mut self, touch: &TouchEvent) {
        if self
            .last_touch_time
            .is_some_and(|last| touch.time - last < DISTINCT_TOUCH_GAP_SECONDS)
        {
            return;
        }
        if self.first_touch_time.is_none() {
            self.first_touch_time = Some(touch.time);
        }
        self.last_touch_time = Some(touch.time);
        self.touch_count += 1;
        let Some(position) = touch.player_position.as_ref().map(vec_to_glam) else {
            return;
        };
        if player_is_on_wall(position) {
            self.wall_touch_count += 1;
        } else if AirDribblePolicy::is_air_touch_position(position) {
            self.aerial_touch_count += 1;
        }
    }

    fn record_ball_movement(&mut self, previous_ball_y: f32, ball_y: f32) {
        let team_forward_sign = if self.is_team_0 { 1.0 } else { -1.0 };
        let advance = (ball_y - previous_ball_y) * team_forward_sign;
        if advance >= 0.0 {
            self.advance_distance += advance;
        } else {
            self.retreat_distance -= advance;
        }
    }

    fn record_proximity_sample(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
    ) {
        let Some(ball_position) = ball.position() else {
            return;
        };
        let close = players
            .player(&self.player_id)
            .and_then(PlayerSample::position)
            .is_some_and(|player_position| {
                player_position.distance(ball_position) <= controlled_play::CLOSE_DISTANCE_3D
            });
        if close {
            self.close_time += frame.dt.max(0.0);
        }
    }

    fn touch_span(&self) -> f32 {
        match (self.first_touch_time, self.last_touch_time) {
            (Some(first), Some(last)) => (last - first).max(0.0),
            _ => 0.0,
        }
    }

    /// Controlled play's qualifying criteria, applied to this span. Kept in
    /// lockstep via the shared constants in `controlled_play`.
    fn is_sustained_control(&self) -> bool {
        self.touch_count >= controlled_play::MIN_TOUCHES
            && self.duration >= controlled_play::MIN_EPISODE_DURATION_SECONDS
            && self.touch_span() >= controlled_play::MIN_FIRST_TO_LAST_TOUCH_DURATION_SECONDS
            && self.close_time >= controlled_play::MIN_CLOSE_DURATION_SECONDS
    }

    fn record_carry_sample(&mut self, frame: &FrameInfo, kind: Option<BallCarryKind>) {
        if let Some(kind) = kind {
            if self.last_carry_kind != Some(kind) {
                match kind {
                    BallCarryKind::Carry => self.carry_count += 1,
                    BallCarryKind::AirDribble => self.air_dribble_count += 1,
                }
            }
            match kind {
                BallCarryKind::Carry => self.carry_time += frame.dt.max(0.0),
                BallCarryKind::AirDribble => self.air_dribble_time += frame.dt.max(0.0),
            }
        }
        self.last_carry_kind = kind;
    }

    fn into_event(self) -> PlayerPossessionEvent {
        let sustained_control = self.is_sustained_control();
        PlayerPossessionEvent {
            player_id: self.player_id,
            is_team_0: self.is_team_0,
            start_frame: self.start_frame,
            end_frame: self.end_frame,
            start_time: self.start_time,
            end_time: self.end_time,
            duration: self.duration,
            touch_count: self.touch_count,
            aerial_touch_count: self.aerial_touch_count,
            wall_touch_count: self.wall_touch_count,
            advance_distance: self.advance_distance,
            retreat_distance: self.retreat_distance,
            carry_time: self.carry_time,
            air_dribble_time: self.air_dribble_time,
            carry_count: self.carry_count,
            air_dribble_count: self.air_dribble_count,
            close_time: self.close_time,
            sustained_control,
            start_field_third: self.start_field_third,
            end_field_third: self.end_field_third,
        }
    }
}

/// Builds per-player possession spans from the shared possession tracker state.
///
/// The raw `possession` stream slices spans whenever the labeled state changes
/// (including field-third moves) and drops the player during pending-turnover
/// windows. This calculator instead emits one event per continuous stretch of
/// player control, bridging contested interruptions shorter than
/// `PLAYER_POSSESSION_MERGE_GAP_SECONDS`, so consumers get a stable
/// "possession" unit for duration, touch, and ball-progress stats.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlayerPossessionCalculator {
    events: EventStream<PlayerPossessionEvent>,
    active: Option<ActivePlayerPossession>,
    suspended: Option<(ActivePlayerPossession, f32)>,
    previous_ball_y: Option<f32>,
}

impl PlayerPossessionCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[PlayerPossessionEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[PlayerPossessionEvent] {
        self.events.new_events()
    }

    fn finalize(&mut self, span: ActivePlayerPossession) {
        self.events.push(span.into_event());
    }

    fn finalize_all(&mut self) {
        if let Some(active) = self.active.take() {
            self.finalize(active);
        }
        if let Some((suspended, _)) = self.suspended.take() {
            self.finalize(suspended);
        }
    }

    fn expire_suspended(&mut self, time: f32) {
        let expired = self.suspended.as_ref().is_some_and(|(_, suspended_at)| {
            time - suspended_at > PLAYER_POSSESSION_MERGE_GAP_SECONDS
        });
        if expired {
            if let Some((suspended, _)) = self.suspended.take() {
                self.finalize(suspended);
            }
        }
    }

    fn field_third(ball: &BallFrameState) -> Option<String> {
        ball.sample().map(|sample| {
            possession::FieldThirdLabel::from_ball(sample)
                .as_label_value()
                .to_owned()
        })
    }

    fn carry_sample_kind(
        player_id: &PlayerId,
        ball: &BallFrameState,
        players: &PlayerFrameState,
    ) -> Option<BallCarryKind> {
        let ball = ball.sample()?;
        let player = players.player(player_id)?;
        BallCarryCalculator::carry_frame_sample(player, ball).map(|sample| sample.kind)
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        possession_state: &PossessionState,
        touch_state: &TouchState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        let ball_y = ball.position().map(|position| position.y);
        if !live_play_state.is_live_play {
            self.finalize_all();
            self.previous_ball_y = ball_y;
            return Ok(());
        }

        self.expire_suspended(frame.time);

        let current_player = possession_state.current_player.clone();
        let field_third = Self::field_third(ball);

        if let Some(active) = self.active.as_ref() {
            if current_player.as_ref() != Some(&active.player_id) {
                let mut active = self.active.take().expect("active span checked above");
                // A different player taking over ends the span outright; a
                // neutral window only suspends it for possible resumption.
                if current_player.is_some() {
                    self.finalize(active);
                } else {
                    active.last_carry_kind = None;
                    self.suspended = Some((active, frame.time));
                }
            }
        }

        if self.active.is_none() {
            if let Some(player_id) = current_player.clone() {
                let resumes_suspended = self
                    .suspended
                    .as_ref()
                    .is_some_and(|(suspended, _)| suspended.player_id == player_id);
                if resumes_suspended {
                    self.active = self.suspended.take().map(|(suspended, _)| suspended);
                } else {
                    if let Some((suspended, _)) = self.suspended.take() {
                        self.finalize(suspended);
                    }
                    let is_team_0 = possession_state
                        .current_team_is_team_0
                        .or_else(|| players.player(&player_id).map(|player| player.is_team_0))
                        .unwrap_or(true);
                    self.active = Some(ActivePlayerPossession::open(
                        frame,
                        player_id,
                        is_team_0,
                        field_third.clone(),
                    ));
                }
            }
        }

        let Some(active) = self.active.as_mut() else {
            self.previous_ball_y = ball_y;
            return Ok(());
        };

        active.record_frame(frame, field_third);
        active.record_proximity_sample(frame, ball, players);
        if let (Some(previous_ball_y), Some(ball_y)) = (self.previous_ball_y, ball_y) {
            active.record_ball_movement(previous_ball_y, ball_y);
        }
        for touch in touch_state.touch_events.iter() {
            if touch.player.as_ref() == Some(&active.player_id) {
                active.record_touch(touch);
            }
        }
        let carry_kind = Self::carry_sample_kind(&active.player_id, ball, players);
        active.record_carry_sample(frame, carry_kind);
        self.previous_ball_y = ball_y;

        Ok(())
    }

    pub fn finish(&mut self) {
        self.finalize_all();
    }
}

#[cfg(test)]
#[path = "player_possession_tests.rs"]
mod tests;
