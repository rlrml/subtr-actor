use super::*;

/// Per-player accumulated one-timer stats.
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

/// Per-team accumulated one-timer stats.
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

/// Accumulates one-timer stats over the replay.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OneTimerStatsAccumulator {
    player_stats: HashMap<PlayerId, OneTimerPlayerStats>,
    team_zero_stats: OneTimerTeamStats,
    team_one_stats: OneTimerTeamStats,
    current_last_one_timer_player: Option<PlayerId>,
}

impl OneTimerStatsAccumulator {
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

    pub fn begin_sample(&mut self, frame: &FrameInfo) {
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

    pub fn clear_current_last(&mut self) {
        self.current_last_one_timer_player = None;
    }

    pub fn apply_event(&mut self, frame: &FrameInfo, event: &OneTimerEvent) {
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
    }

    pub fn finish_sample(&mut self) {
        if let Some(player_id) = self.current_last_one_timer_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_one_timer = true;
            }
        }
    }
}
