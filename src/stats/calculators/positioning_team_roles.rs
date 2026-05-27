use super::positioning_team_role_time::TeamRoleTime;
use super::*;

impl PositioningCalculator {
    pub(crate) fn record_team_roles(
        &mut self,
        frame: &FrameInfo,
        is_team_0: bool,
        team_present_player_count: usize,
        team_roster_count: usize,
        team_players: &[(&PlayerSample, glam::Vec3)],
        event_deltas: &mut HashMap<PlayerId, PositioningEvent>,
    ) {
        if team_roster_count < 2
            || team_present_player_count < team_roster_count
            || team_players.len() < 2
        {
            self.record_no_teammate_time(frame, team_players, event_deltas);
            return;
        }

        let mut sorted_team: Vec<_> = team_players
            .iter()
            .map(|(info, pos)| (info.player_id.clone(), normalized_y(is_team_0, *pos)))
            .collect();
        sorted_team.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

        let team_spread = sorted_team.last().map(|(_, y)| *y).unwrap_or(0.0)
            - sorted_team.first().map(|(_, y)| *y).unwrap_or(0.0);
        if team_spread <= self.config.most_back_forward_threshold_y {
            for (player_id, _) in &sorted_team {
                self.add_role_time(
                    frame,
                    event_deltas,
                    player_id,
                    is_team_0,
                    TeamRoleTime::Other,
                );
            }
            return;
        }

        let min_y = sorted_team.first().map(|(_, y)| *y).unwrap_or(0.0);
        let max_y = sorted_team.last().map(|(_, y)| *y).unwrap_or(0.0);
        let can_assign_mid_role = sorted_team.len() == 3;
        for (player_id, y) in &sorted_team {
            let near_back = (*y - min_y) <= self.config.most_back_forward_threshold_y;
            let near_front = (max_y - *y) <= self.config.most_back_forward_threshold_y;
            let role = if near_back && !near_front {
                TeamRoleTime::MostBack
            } else if near_front && !near_back {
                TeamRoleTime::MostForward
            } else if can_assign_mid_role {
                TeamRoleTime::Mid
            } else {
                TeamRoleTime::Other
            };
            self.add_role_time(frame, event_deltas, player_id, is_team_0, role);
        }
    }

    fn record_no_teammate_time(
        &mut self,
        frame: &FrameInfo,
        team_players: &[(&PlayerSample, glam::Vec3)],
        event_deltas: &mut HashMap<PlayerId, PositioningEvent>,
    ) {
        for (player, _) in team_players {
            self.player_stats
                .entry(player.player_id.clone())
                .or_default()
                .time_no_teammates += frame.dt;
            Self::event_delta(event_deltas, frame, &player.player_id, player.is_team_0)
                .time_no_teammates += frame.dt;
        }
    }
}
