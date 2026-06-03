use super::*;

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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PassStatsAccumulator {
    player_stats: HashMap<PlayerId, PassPlayerStats>,
    team_zero_stats: PassTeamStats,
    team_one_stats: PassTeamStats,
    current_last_completed_pass_player: Option<PlayerId>,
}

impl PassStatsAccumulator {
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

    pub fn current_last_completed_pass_player(&self) -> Option<&PlayerId> {
        self.current_last_completed_pass_player.as_ref()
    }

    pub fn begin_sample(&mut self, frame: &FrameInfo) {
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

    pub fn clear_current_last(&mut self) {
        self.current_last_completed_pass_player = None;
    }

    pub fn apply_event(&mut self, frame: &FrameInfo, event: &PassEvent) {
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
    }

    pub fn finish_sample(&mut self) {
        if let Some(player_id) = self.current_last_completed_pass_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_completed_pass = true;
            }
        }
    }
}
