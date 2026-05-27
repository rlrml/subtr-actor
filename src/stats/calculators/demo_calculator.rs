use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DemoCalculator {
    pub(super) player_stats: HashMap<PlayerId, DemoPlayerStats>,
    pub(super) player_teams: HashMap<PlayerId, bool>,
    pub(super) team_zero_stats: DemoTeamStats,
    pub(super) team_one_stats: DemoTeamStats,
    pub(super) timeline: Vec<TimelineEvent>,
    pub(super) last_seen_frame: HashMap<(PlayerId, PlayerId), usize>,
    pub(super) active_pairs: HashSet<(PlayerId, PlayerId)>,
}

impl DemoCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, DemoPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &DemoTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &DemoTeamStats {
        &self.team_one_stats
    }

    pub fn timeline(&self) -> &[TimelineEvent] {
        &self.timeline
    }

    pub(super) fn should_count_demo(
        &mut self,
        attacker: &PlayerId,
        victim: &PlayerId,
        frame_number: usize,
    ) -> bool {
        let key = (attacker.clone(), victim.clone());
        let already_counted = self
            .last_seen_frame
            .get(&key)
            .map(|previous_frame| {
                frame_number.saturating_sub(*previous_frame) <= DEMO_REPEAT_FRAME_WINDOW
            })
            .unwrap_or(false);
        self.last_seen_frame.insert(key, frame_number);
        !already_counted
    }
}
