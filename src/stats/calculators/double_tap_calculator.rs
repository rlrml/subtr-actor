use super::*;

#[derive(Debug, Clone, Default)]
pub struct DoubleTapCalculator {
    pub(super) player_stats: HashMap<PlayerId, DoubleTapPlayerStats>,
    pub(super) team_zero_stats: DoubleTapTeamStats,
    pub(super) team_one_stats: DoubleTapTeamStats,
    pub(super) events: Vec<DoubleTapEvent>,
    pub(super) pending_backboard_bounces: Vec<PendingBackboardBounce>,
    pub(super) current_last_double_tap_player: Option<PlayerId>,
}

impl DoubleTapCalculator {
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

    pub fn events(&self) -> &[DoubleTapEvent] {
        &self.events
    }

    pub(super) fn begin_sample(&mut self, frame: &FrameInfo) {
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
}
