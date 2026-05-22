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
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub passer: PlayerId,
    pub is_team_0: bool,
    pub pass_start_time: f32,
    pub pass_start_frame: usize,
    pub pass_duration: f32,
    pub pass_travel_distance: f32,
    pub pass_advance_distance: f32,
    pub ball_speed: f32,
    pub goal_alignment: f32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct OneTimerPlayerStats {
    pub count: u32,
    pub total_ball_speed: f32,
    pub fastest_ball_speed: f32,
    pub total_pass_distance: f32,
    pub is_last_one_timer: bool,
    pub last_one_timer_time: Option<f32>,
    pub last_one_timer_frame: Option<usize>,
    pub time_since_last_one_timer: Option<f32>,
    pub frames_since_last_one_timer: Option<usize>,
}

impl OneTimerPlayerStats {
    pub fn average_ball_speed(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_speed / self.count as f32
        }
    }

    pub fn average_pass_distance(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_pass_distance / self.count as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct OneTimerTeamStats {
    pub count: u32,
    pub total_ball_speed: f32,
    pub fastest_ball_speed: f32,
}

impl OneTimerTeamStats {
    pub fn average_ball_speed(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_speed / self.count as f32
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OneTimerCalculator {
    player_stats: HashMap<PlayerId, OneTimerPlayerStats>,
    team_zero_stats: OneTimerTeamStats,
    team_one_stats: OneTimerTeamStats,
    events: Vec<OneTimerEvent>,
    processed_pass_events: usize,
    current_last_one_timer_player: Option<PlayerId>,
}

impl OneTimerCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, OneTimerPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &OneTimerTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &OneTimerTeamStats {
        &self.team_one_stats
    }

    pub fn events(&self) -> &[OneTimerEvent] {
        &self.events
    }

    fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_one_timer = false;
            stats.time_since_last_one_timer = stats
                .last_one_timer_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_one_timer = stats
                .last_one_timer_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
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
            passer: pass.passer.clone(),
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
        let player_stats = self.player_stats.entry(event.player.clone()).or_default();
        player_stats.count += 1;
        player_stats.total_ball_speed += event.ball_speed;
        player_stats.fastest_ball_speed = player_stats.fastest_ball_speed.max(event.ball_speed);
        player_stats.total_pass_distance += event.pass_travel_distance;
        player_stats.last_one_timer_time = Some(event.time);
        player_stats.last_one_timer_frame = Some(event.frame);
        player_stats.time_since_last_one_timer = Some((frame.time - event.time).max(0.0));
        player_stats.frames_since_last_one_timer =
            Some(frame.frame_number.saturating_sub(event.frame));

        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        team_stats.count += 1;
        team_stats.total_ball_speed += event.ball_speed;
        team_stats.fastest_ball_speed = team_stats.fastest_ball_speed.max(event.ball_speed);

        self.current_last_one_timer_player = Some(event.player.clone());
        self.events.push(event);
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        pass_calculator: &PassCalculator,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.begin_sample(frame);
        if !live_play {
            self.current_last_one_timer_player = None;
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

        if let Some(player_id) = self.current_last_one_timer_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_one_timer = true;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "one_timer_tests.rs"]
mod tests;
