use super::*;

#[derive(Debug, Clone, Default)]
pub struct OneTimerCalculator {
    pub(super) player_stats: HashMap<PlayerId, OneTimerPlayerStats>,
    pub(super) team_zero_stats: OneTimerTeamStats,
    pub(super) team_one_stats: OneTimerTeamStats,
    pub(super) events: Vec<OneTimerEvent>,
    pub(super) processed_pass_events: usize,
    pub(super) current_last_one_timer_player: Option<PlayerId>,
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

    pub(super) fn begin_sample(&mut self, frame: &FrameInfo) {
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
}
