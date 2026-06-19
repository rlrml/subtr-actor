use super::*;

/// Per-player accumulated double-tap stats.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DoubleTapPlayerStats {
    pub count: u32,
    pub is_last_double_tap: bool,
    pub last_double_tap_time: Option<f32>,
    pub last_double_tap_frame: Option<usize>,
    pub time_since_last_double_tap: Option<f32>,
    pub frames_since_last_double_tap: Option<usize>,
}

/// Per-team accumulated double-tap stats.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DoubleTapTeamStats {
    pub count: u32,
}

/// Accumulates double-tap stats over the replay.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DoubleTapStatsAccumulator {
    player_stats: HashMap<PlayerId, DoubleTapPlayerStats>,
    team_zero_stats: DoubleTapTeamStats,
    team_one_stats: DoubleTapTeamStats,
    current_last_double_tap_player: Option<PlayerId>,
}

impl DoubleTapStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, DoubleTapPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &DoubleTapTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &DoubleTapTeamStats {
        &self.team_one_stats
    }

    pub fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_double_tap = false;
            stats.time_since_last_double_tap = stats
                .last_double_tap_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_double_tap = stats
                .last_double_tap_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    pub fn apply_event(&mut self, frame: &FrameInfo, event: &DoubleTapEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.count += 1;
        stats.last_double_tap_time = Some(event.time);
        stats.last_double_tap_frame = Some(event.frame);
        stats.time_since_last_double_tap = Some((frame.time - event.time).max(0.0));
        stats.frames_since_last_double_tap = Some(frame.frame_number.saturating_sub(event.frame));

        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        team_stats.count += 1;
        self.current_last_double_tap_player = Some(event.player.clone());
    }

    pub fn finish_sample(&mut self) {
        if let Some(player_id) = self.current_last_double_tap_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_double_tap = true;
            }
        }
    }
}
