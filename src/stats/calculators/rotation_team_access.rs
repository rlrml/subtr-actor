use super::*;

impl RotationCalculator {
    pub(crate) fn team_tracker(&self, is_team_0: bool) -> &TeamFirstManTracker {
        if is_team_0 {
            &self.team_zero_tracker
        } else {
            &self.team_one_tracker
        }
    }

    pub(crate) fn team_tracker_mut(&mut self, is_team_0: bool) -> &mut TeamFirstManTracker {
        if is_team_0 {
            &mut self.team_zero_tracker
        } else {
            &mut self.team_one_tracker
        }
    }

    pub(crate) fn team_stats_mut(&mut self, is_team_0: bool) -> &mut RotationTeamStats {
        if is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        }
    }
}
