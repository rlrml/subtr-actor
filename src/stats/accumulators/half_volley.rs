use super::*;

/// Per-player accumulated half-volley stats.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct HalfVolleyPlayerStats {
    pub count: u32,
    pub total_ball_speed: f32,
    pub fastest_ball_speed: f32,
    pub is_last_half_volley: bool,
    pub last_half_volley_time: Option<f32>,
    pub last_half_volley_frame: Option<usize>,
    pub time_since_last_half_volley: Option<f32>,
    pub frames_since_last_half_volley: Option<usize>,
}

impl HalfVolleyPlayerStats {
    pub fn average_ball_speed(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_speed / self.count as f32
        }
    }
}

/// Per-team accumulated half-volley stats.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct HalfVolleyTeamStats {
    pub count: u32,
    pub total_ball_speed: f32,
    pub fastest_ball_speed: f32,
}

impl HalfVolleyTeamStats {
    pub fn average_ball_speed(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_speed / self.count as f32
        }
    }
}

/// Accumulates half-volley stats over the replay.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HalfVolleyStatsAccumulator {
    player_stats: HashMap<PlayerId, HalfVolleyPlayerStats>,
    team_zero_stats: HalfVolleyTeamStats,
    team_one_stats: HalfVolleyTeamStats,
    current_last_half_volley_player: Option<PlayerId>,
}

impl HalfVolleyStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, HalfVolleyPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &HalfVolleyTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &HalfVolleyTeamStats {
        &self.team_one_stats
    }

    pub fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_half_volley = false;
            stats.time_since_last_half_volley = stats
                .last_half_volley_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_half_volley = stats
                .last_half_volley_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    pub fn apply_event(&mut self, event: &HalfVolleyEvent, frame: &FrameInfo) {
        let player_stats = self.player_stats.entry(event.player.clone()).or_default();
        player_stats.count += 1;
        player_stats.total_ball_speed += event.ball_speed;
        player_stats.fastest_ball_speed = player_stats.fastest_ball_speed.max(event.ball_speed);
        player_stats.last_half_volley_time = Some(event.time);
        player_stats.last_half_volley_frame = Some(event.frame);
        player_stats.time_since_last_half_volley = Some((frame.time - event.time).max(0.0));
        player_stats.frames_since_last_half_volley =
            Some(frame.frame_number.saturating_sub(event.frame));

        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        team_stats.count += 1;
        team_stats.total_ball_speed += event.ball_speed;
        team_stats.fastest_ball_speed = team_stats.fastest_ball_speed.max(event.ball_speed);

        self.current_last_half_volley_player = Some(event.player.clone());
    }

    pub fn restore_current_last_event_marker(&mut self) {
        if let Some(player_id) = self.current_last_half_volley_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_half_volley = true;
            }
        }
    }

    pub fn reset_current_last_event_marker(&mut self) {
        self.current_last_half_volley_player = None;
    }
}
