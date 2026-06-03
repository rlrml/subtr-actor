use super::*;

const ONE_TIMER_MIN_BALL_SPEED: f32 = 1000.0;
const ONE_TIMER_MIN_GOAL_ALIGNMENT_COSINE: f32 = 0.65;
const GOAL_CENTER_Y: f32 = 5120.0;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct OneTimerEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub passer: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passer_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub pass_start_time: f32,
    pub pass_start_frame: usize,
    pub pass_duration: f32,
    pub pass_travel_distance: f32,
    pub pass_advance_distance: f32,
    pub ball_speed: f32,
    pub goal_alignment: f32,
}

#[derive(Debug, Clone, Default)]
pub struct OneTimerCalculator {
    stats: OneTimerStatsAccumulator,
    events: EventStream<OneTimerEvent>,
    processed_pass_events: usize,
}

impl OneTimerCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, OneTimerPlayerStats> {
        self.stats.player_stats()
    }

    pub fn team_zero_stats(&self) -> &OneTimerTeamStats {
        self.stats.team_zero_stats()
    }

    pub fn team_one_stats(&self) -> &OneTimerTeamStats {
        self.stats.team_one_stats()
    }

    pub fn events(&self) -> &[OneTimerEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[OneTimerEvent] {
        self.events.new_events()
    }

    fn one_timer_event_for_pass(pass: &PassEvent, ball: &BallFrameState) -> Option<OneTimerEvent> {
        let ball = ball.sample()?;
        let ball_position = ball.position();
        let ball_velocity = ball.velocity();
        let ball_speed = ball_velocity.length();
        if ball_speed < ONE_TIMER_MIN_BALL_SPEED {
            return None;
        }

        let target_y = if pass.is_team_0 {
            GOAL_CENTER_Y
        } else {
            -GOAL_CENTER_Y
        };
        let goal_direction = glam::Vec3::new(0.0, target_y, ball_position.z) - ball_position;
        let goal_alignment = goal_direction
            .normalize_or_zero()
            .dot(ball_velocity.normalize_or_zero());
        if goal_alignment < ONE_TIMER_MIN_GOAL_ALIGNMENT_COSINE {
            return None;
        }

        Some(OneTimerEvent {
            time: pass.time,
            frame: pass.frame,
            player: pass.receiver.clone(),
            player_position: pass.receiver_position,
            passer: pass.passer.clone(),
            passer_position: pass.passer_position,
            is_team_0: pass.is_team_0,
            pass_start_time: pass.start_time,
            pass_start_frame: pass.start_frame,
            pass_duration: pass.duration,
            pass_travel_distance: pass.ball_travel_distance,
            pass_advance_distance: pass.ball_advance_distance,
            ball_speed,
            goal_alignment,
        })
    }

    fn record_one_timer(&mut self, frame: &FrameInfo, event: OneTimerEvent) {
        self.stats.apply_event(frame, &event);
        self.events.push(event);
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        pass_calculator: &PassCalculator,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        self.stats.begin_sample(frame);
        if !live_play {
            self.stats.clear_current_last();
            self.processed_pass_events = pass_calculator.events().len();
            return Ok(());
        }

        for pass in &pass_calculator.events()[self.processed_pass_events..] {
            if pass.frame != frame.frame_number {
                continue;
            }
            if let Some(event) = Self::one_timer_event_for_pass(pass, ball) {
                self.record_one_timer(frame, event);
            }
        }
        self.processed_pass_events = pass_calculator.events().len();
        self.stats.finish_sample();

        Ok(())
    }
}

#[cfg(test)]
#[path = "one_timer_tests.rs"]
mod tests;
