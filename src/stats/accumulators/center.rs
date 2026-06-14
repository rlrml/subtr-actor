use super::*;

/// Per-player accumulated centering-pass stats.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CenterPlayerStats {
    pub count: u32,
    pub total_ball_travel_distance: f32,
    pub total_ball_advance_distance: f32,
    pub total_lateral_centering_distance: f32,
    pub longest_center_distance: f32,
    pub is_last_center: bool,
    pub last_center_time: Option<f32>,
    pub last_center_frame: Option<usize>,
    pub time_since_last_center: Option<f32>,
    pub frames_since_last_center: Option<usize>,
}

impl CenterPlayerStats {
    pub fn average_ball_travel_distance(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_travel_distance / self.count as f32
        }
    }

    pub fn average_ball_advance_distance(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_advance_distance / self.count as f32
        }
    }

    pub fn average_lateral_centering_distance(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_lateral_centering_distance / self.count as f32
        }
    }
}

/// Per-team accumulated centering-pass stats.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CenterTeamStats {
    pub count: u32,
    pub total_ball_travel_distance: f32,
    pub total_ball_advance_distance: f32,
    pub total_lateral_centering_distance: f32,
    pub longest_center_distance: f32,
}

impl CenterTeamStats {
    pub fn average_ball_travel_distance(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_travel_distance / self.count as f32
        }
    }

    pub fn average_ball_advance_distance(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_advance_distance / self.count as f32
        }
    }

    pub fn average_lateral_centering_distance(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_lateral_centering_distance / self.count as f32
        }
    }
}

/// Accumulates centering-pass stats over the replay.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CenterStatsAccumulator {
    player_stats: HashMap<PlayerId, CenterPlayerStats>,
    team_zero_stats: CenterTeamStats,
    team_one_stats: CenterTeamStats,
    current_last_center_player: Option<PlayerId>,
}

impl CenterStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, CenterPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &CenterTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &CenterTeamStats {
        &self.team_one_stats
    }

    pub fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_center = false;
            stats.time_since_last_center = stats
                .last_center_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_center = stats
                .last_center_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    pub fn clear_current_last(&mut self) {
        self.current_last_center_player = None;
    }

    pub fn apply_event(&mut self, frame: &FrameInfo, event: &CenterEvent) {
        let player_stats = self.player_stats.entry(event.player.clone()).or_default();
        player_stats.count += 1;
        player_stats.total_ball_travel_distance += event.ball_travel_distance;
        player_stats.total_ball_advance_distance += event.ball_advance_distance;
        player_stats.total_lateral_centering_distance += event.lateral_centering_distance;
        player_stats.longest_center_distance = player_stats
            .longest_center_distance
            .max(event.ball_travel_distance);
        player_stats.last_center_time = Some(event.time);
        player_stats.last_center_frame = Some(event.frame);
        player_stats.time_since_last_center = Some((frame.time - event.time).max(0.0));
        player_stats.frames_since_last_center =
            Some(frame.frame_number.saturating_sub(event.frame));

        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        team_stats.count += 1;
        team_stats.total_ball_travel_distance += event.ball_travel_distance;
        team_stats.total_ball_advance_distance += event.ball_advance_distance;
        team_stats.total_lateral_centering_distance += event.lateral_centering_distance;
        team_stats.longest_center_distance = team_stats
            .longest_center_distance
            .max(event.ball_travel_distance);

        self.current_last_center_player = Some(event.player.clone());
    }

    pub fn finish_sample(&mut self) {
        if let Some(player_id) = self.current_last_center_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_center = true;
            }
        }
    }
}
