use super::*;

const PASS_MAX_DURATION_SECONDS: f32 = 3.0;
const PASS_MIN_BALL_TRAVEL_DISTANCE: f32 = 500.0;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PassEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub passer: PlayerId,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub receiver: PlayerId,
    pub is_team_0: bool,
    pub start_time: f32,
    pub start_frame: usize,
    pub duration: f32,
    pub ball_travel_distance: f32,
    pub ball_advance_distance: f32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PassPlayerStats {
    pub completed_pass_count: u32,
    pub received_pass_count: u32,
    pub total_pass_distance: f32,
    pub total_pass_advance: f32,
    pub longest_pass_distance: f32,
    pub is_last_completed_pass: bool,
    pub last_completed_pass_time: Option<f32>,
    pub last_completed_pass_frame: Option<usize>,
    pub time_since_last_completed_pass: Option<f32>,
    pub frames_since_last_completed_pass: Option<usize>,
}

impl PassPlayerStats {
    pub fn average_pass_distance(&self) -> f32 {
        if self.completed_pass_count == 0 {
            0.0
        } else {
            self.total_pass_distance / self.completed_pass_count as f32
        }
    }

    pub fn average_pass_advance(&self) -> f32 {
        if self.completed_pass_count == 0 {
            0.0
        } else {
            self.total_pass_advance / self.completed_pass_count as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PassTeamStats {
    pub completed_pass_count: u32,
    pub total_pass_distance: f32,
    pub total_pass_advance: f32,
    pub longest_pass_distance: f32,
}

impl PassTeamStats {
    pub fn average_pass_distance(&self) -> f32 {
        if self.completed_pass_count == 0 {
            0.0
        } else {
            self.total_pass_distance / self.completed_pass_count as f32
        }
    }

    pub fn average_pass_advance(&self) -> f32 {
        if self.completed_pass_count == 0 {
            0.0
        } else {
            self.total_pass_advance / self.completed_pass_count as f32
        }
    }
}

#[derive(Debug, Clone)]
struct PendingPassTouch {
    player: PlayerId,
    is_team_0: bool,
    time: f32,
    frame: usize,
    ball_position: glam::Vec3,
}

#[derive(Debug, Clone, Default)]
pub struct PassCalculator {
    player_stats: HashMap<PlayerId, PassPlayerStats>,
    team_zero_stats: PassTeamStats,
    team_one_stats: PassTeamStats,
    events: Vec<PassEvent>,
    last_touch: Option<PendingPassTouch>,
    current_last_completed_pass_player: Option<PlayerId>,
}

impl PassCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, PassPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &PassTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &PassTeamStats {
        &self.team_one_stats
    }

    pub fn events(&self) -> &[PassEvent] {
        &self.events
    }

    fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_completed_pass = false;
            stats.time_since_last_completed_pass = stats
                .last_completed_pass_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_completed_pass = stats
                .last_completed_pass_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    fn pass_event_for_touch(
        &self,
        touch: &TouchEvent,
        receiver: &PlayerId,
        ball_position: glam::Vec3,
    ) -> Option<PassEvent> {
        let previous = self.last_touch.as_ref()?;
        if previous.player == *receiver || previous.is_team_0 != touch.team_is_team_0 {
            return None;
        }

        let duration = touch.time - previous.time;
        if !(0.0..=PASS_MAX_DURATION_SECONDS).contains(&duration) {
            return None;
        }

        let ball_delta = ball_position - previous.ball_position;
        let ball_travel_distance = ball_delta.length();
        if ball_travel_distance < PASS_MIN_BALL_TRAVEL_DISTANCE {
            return None;
        }

        let team_forward_sign = if touch.team_is_team_0 { 1.0 } else { -1.0 };
        Some(PassEvent {
            time: touch.time,
            frame: touch.frame,
            passer: previous.player.clone(),
            receiver: receiver.clone(),
            is_team_0: touch.team_is_team_0,
            start_time: previous.time,
            start_frame: previous.frame,
            duration,
            ball_travel_distance,
            ball_advance_distance: ball_delta.y * team_forward_sign,
        })
    }

    fn record_pass(&mut self, frame: &FrameInfo, event: PassEvent) {
        let passer_stats = self.player_stats.entry(event.passer.clone()).or_default();
        passer_stats.completed_pass_count += 1;
        passer_stats.total_pass_distance += event.ball_travel_distance;
        passer_stats.total_pass_advance += event.ball_advance_distance;
        passer_stats.longest_pass_distance = passer_stats
            .longest_pass_distance
            .max(event.ball_travel_distance);
        passer_stats.last_completed_pass_time = Some(event.time);
        passer_stats.last_completed_pass_frame = Some(event.frame);
        passer_stats.time_since_last_completed_pass = Some((frame.time - event.time).max(0.0));
        passer_stats.frames_since_last_completed_pass =
            Some(frame.frame_number.saturating_sub(event.frame));

        self.player_stats
            .entry(event.receiver.clone())
            .or_default()
            .received_pass_count += 1;

        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        team_stats.completed_pass_count += 1;
        team_stats.total_pass_distance += event.ball_travel_distance;
        team_stats.total_pass_advance += event.ball_advance_distance;
        team_stats.longest_pass_distance = team_stats
            .longest_pass_distance
            .max(event.ball_travel_distance);

        self.current_last_completed_pass_player = Some(event.passer.clone());
        self.events.push(event);
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        events: &FrameEventsState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.begin_sample(frame);
        if !live_play {
            self.last_touch = None;
            self.current_last_completed_pass_player = None;
            return Ok(());
        }

        let Some(ball_position) = ball.position() else {
            return Ok(());
        };

        for touch in &events.touch_events {
            let Some(player) = touch.player.clone() else {
                self.last_touch = None;
                continue;
            };

            if let Some(pass_event) = self.pass_event_for_touch(touch, &player, ball_position) {
                self.record_pass(frame, pass_event);
            }

            self.last_touch = Some(PendingPassTouch {
                player,
                is_team_0: touch.team_is_team_0,
                time: touch.time,
                frame: touch.frame,
                ball_position,
            });
        }

        if let Some(player_id) = self.current_last_completed_pass_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_completed_pass = true;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "pass_tests.rs"]
mod tests;
