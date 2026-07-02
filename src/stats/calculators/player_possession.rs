use super::*;

/// How long a player's possession may be interrupted (pending turnover,
/// contested challenge, brief neutral window) before the span is finalized
/// instead of resumed when the same player re-establishes control.
const PLAYER_POSSESSION_MERGE_GAP_SECONDS: f32 = 2.0;
/// Minimum spacing between touches for them to count as distinct touches
/// within a possession span (mirrors the controlled-play touch chaining).
const DISTINCT_TOUCH_GAP_SECONDS: f32 = 0.12;
/// Distinct touches a player must make for a span to count as possession at
/// all. A single glancing contact — a kickoff poke the player never follows up
/// on, say — is not possession, even though that player stayed the last to
/// touch until an opponent took over. Consecutive touches are the primary
/// possession signal; proximity is only a loose sanity bound (see
/// `MAX_POSSESSION_BALL_DISTANCE`).
const MIN_POSSESSION_TOUCHES: u32 = 2;
/// A generous ceiling on how far the ball may drift from the holder before the
/// span is treated as loose. Deliberately far looser than the controlled-play
/// close radius (`controlled_play::CLOSE_DISTANCE_3D`, 700uu): proximity is not
/// a primary signal, but once the ball is clearly gone the holder no longer
/// has it, so the span suspends (and eventually expires) instead of riding the
/// last touch until an opponent finally intervenes.
const MAX_POSSESSION_BALL_DISTANCE: f32 = 2500.0;

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
    /// time inside a merged span and the loose tail after the player's final
    /// touch (once the ball is hit away, the remaining flight time is nobody's
    /// possession — mirroring how team possession backdates a loss to the last
    /// touch), so it can be less than `end_time - start_time`.
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

/// The per-frame accumulators that only count while the ball is possessed.
/// The running copy accrues every active frame; a snapshot is taken at each
/// ball contact, and the snapshot is what the emitted event reports, so the
/// provisional loose tail after the final touch is never credited.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct PossessedTotals {
    duration: f32,
    close_time: f32,
    advance_distance: f32,
    retreat_distance: f32,
    carry_time: f32,
    air_dribble_time: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct ActivePlayerPossession {
    player_id: PlayerId,
    is_team_0: bool,
    start_frame: usize,
    end_frame: usize,
    start_time: f32,
    end_time: f32,
    running: PossessedTotals,
    /// [`Self::running`] as of the player's most recent ball contact.
    at_last_touch: PossessedTotals,
    touch_count: u32,
    aerial_touch_count: u32,
    wall_touch_count: u32,
    first_touch_time: Option<f32>,
    last_touch_time: Option<f32>,
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
            running: PossessedTotals::default(),
            at_last_touch: PossessedTotals::default(),
            touch_count: 0,
            aerial_touch_count: 0,
            wall_touch_count: 0,
            first_touch_time: None,
            last_touch_time: None,
            carry_count: 0,
            air_dribble_count: 0,
            last_carry_kind: None,
            start_field_third: field_third.clone(),
            end_field_third: field_third,
        }
    }

    fn record_frame(&mut self, frame: &FrameInfo, field_third: Option<String>) {
        self.running.duration += frame.dt.max(0.0);
        self.end_frame = frame.frame_number;
        self.end_time = frame.time;
        if field_third.is_some() {
            self.end_field_third = field_third;
        }
    }

    fn record_touch(&mut self, touch: &TouchEvent) {
        // Any contact — even one too close to the previous touch to count as
        // distinct — extends the possessed totals to this frame.
        self.at_last_touch = self.running;
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
            self.running.advance_distance += advance;
        } else {
            self.running.retreat_distance -= advance;
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
            self.running.close_time += frame.dt.max(0.0);
        }
    }

    fn touch_span(&self) -> f32 {
        match (self.first_touch_time, self.last_touch_time) {
            (Some(first), Some(last)) => (last - first).max(0.0),
            _ => 0.0,
        }
    }

    /// Controlled play's qualifying criteria, applied to this span. Kept in
    /// lockstep via the shared constants in `controlled_play`, and judged on
    /// the same possessed totals the emitted event reports.
    fn is_sustained_control(&self) -> bool {
        self.touch_count >= controlled_play::MIN_TOUCHES
            && self.at_last_touch.duration >= controlled_play::MIN_EPISODE_DURATION_SECONDS
            && self.touch_span() >= controlled_play::MIN_FIRST_TO_LAST_TOUCH_DURATION_SECONDS
            && self.at_last_touch.close_time >= controlled_play::MIN_CLOSE_DURATION_SECONDS
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
                BallCarryKind::Carry => self.running.carry_time += frame.dt.max(0.0),
                BallCarryKind::AirDribble => self.running.air_dribble_time += frame.dt.max(0.0),
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
            duration: self.at_last_touch.duration,
            touch_count: self.touch_count,
            aerial_touch_count: self.aerial_touch_count,
            wall_touch_count: self.wall_touch_count,
            advance_distance: self.at_last_touch.advance_distance,
            retreat_distance: self.at_last_touch.retreat_distance,
            carry_time: self.at_last_touch.carry_time,
            air_dribble_time: self.at_last_touch.air_dribble_time,
            carry_count: self.carry_count,
            air_dribble_count: self.air_dribble_count,
            close_time: self.at_last_touch.close_time,
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
        // Possession requires the holder to have actually played the ball more
        // than once. A span finalizes only when an opponent takes over or the
        // ball is lost, so this is the retroactive "there was never really any
        // possession here" check: a lone touch is dropped, never emitted.
        if span.touch_count < MIN_POSSESSION_TOUCHES {
            return;
        }
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
            BallThirdLabel::from_ball(sample)
                .as_label_value()
                .to_owned()
        })
    }

    /// Demotes the touch-based holder to "no holder" when the ball has drifted
    /// far from them. Possession is touch-led, so proximity is deliberately
    /// loose — but once the ball is well out of reach the holder no longer has
    /// it. Returning `None` makes the open span suspend (resumable if the same
    /// player gets back to it within the merge gap) rather than ride the last
    /// touch until an opponent finally intervenes.
    fn holder_within_reach(
        current_player: Option<PlayerId>,
        ball: &BallFrameState,
        players: &PlayerFrameState,
    ) -> Option<PlayerId> {
        let player_id = current_player?;
        let within = match (
            ball.position(),
            players.player(&player_id).and_then(PlayerSample::position),
        ) {
            (Some(ball_position), Some(player_position)) => {
                player_position.distance(ball_position) <= MAX_POSSESSION_BALL_DISTANCE
            }
            // Missing geometry: trust the touch-based holder rather than guess.
            _ => true,
        };
        within.then_some(player_id)
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

        let current_player =
            Self::holder_within_reach(possession_state.current_player.clone(), ball, players);
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
                    // The loose tail since the last touch was provisional; the
                    // hold lapsed, so it is not possession even if the same
                    // player re-establishes control and the span resumes.
                    active.running = active.at_last_touch;
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
        let carry_kind = Self::carry_sample_kind(&active.player_id, ball, players);
        active.record_carry_sample(frame, carry_kind);
        // Touches last: a touch snapshots the running possessed totals, which
        // must already include this frame's samples.
        for touch in touch_state.touch_events.iter() {
            if touch.player.as_ref() == Some(&active.player_id) {
                active.record_touch(touch);
            }
        }
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
