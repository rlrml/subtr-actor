use super::*;

impl PositioningCalculator {
    pub(crate) fn add_role_time(
        &mut self,
        frame: &FrameInfo,
        event_deltas: &mut HashMap<PlayerId, PositioningEvent>,
        player_id: &PlayerId,
        is_team_0: bool,
        role: TeamRoleTime,
    ) {
        let stats = self.player_stats.entry(player_id.clone()).or_default();
        let delta = Self::event_delta(event_deltas, frame, player_id, is_team_0);
        match role {
            TeamRoleTime::MostBack => {
                stats.time_most_back += frame.dt;
                delta.time_most_back += frame.dt;
            }
            TeamRoleTime::MostForward => {
                stats.time_most_forward += frame.dt;
                delta.time_most_forward += frame.dt;
            }
            TeamRoleTime::Mid => {
                stats.time_mid_role += frame.dt;
                delta.time_mid_role += frame.dt;
            }
            TeamRoleTime::Other => {
                stats.time_other_role += frame.dt;
                delta.time_other_role += frame.dt;
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum TeamRoleTime {
    MostBack,
    MostForward,
    Mid,
    Other,
}
