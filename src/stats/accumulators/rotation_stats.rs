use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationPlayerStats {
    pub active_game_time: f32,
    pub tracked_time: f32,
    pub time_first_man: f32,
    pub time_second_man: f32,
    pub time_third_man: f32,
    pub time_ambiguous_role: f32,
    pub time_behind_play: f32,
    pub time_level_with_play: f32,
    pub time_ahead_of_play: f32,
    pub longest_first_man_stint_time: f32,
    pub first_man_stint_count: u32,
    pub became_first_man_count: u32,
    pub lost_first_man_count: u32,
    pub current_role_state: RoleState,
    pub current_depth_state: PlayDepthState,
}

impl RotationPlayerStats {
    fn role_pct(&self, value: f32) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            value * 100.0 / self.tracked_time
        }
    }

    pub fn first_man_pct(&self) -> f32 {
        self.role_pct(self.time_first_man)
    }

    pub fn second_man_pct(&self) -> f32 {
        self.role_pct(self.time_second_man)
    }

    pub fn third_man_pct(&self) -> f32 {
        self.role_pct(self.time_third_man)
    }

    pub fn ambiguous_role_pct(&self) -> f32 {
        self.role_pct(self.time_ambiguous_role)
    }

    pub fn behind_play_pct(&self) -> f32 {
        self.role_pct(self.time_behind_play)
    }

    pub fn level_with_play_pct(&self) -> f32 {
        self.role_pct(self.time_level_with_play)
    }

    pub fn ahead_of_play_pct(&self) -> f32 {
        self.role_pct(self.time_ahead_of_play)
    }

    pub fn average_first_man_stint_time(&self) -> f32 {
        if self.first_man_stint_count == 0 {
            0.0
        } else {
            self.time_first_man / self.first_man_stint_count as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationTeamStats {
    pub first_man_changes_for_team: u32,
    pub rotation_count: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RotationStatsAccumulator {
    player_stats: HashMap<PlayerId, RotationPlayerStats>,
    team_zero_stats: RotationTeamStats,
    team_one_stats: RotationTeamStats,
}

impl RotationStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, RotationPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &RotationTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &RotationTeamStats {
        &self.team_one_stats
    }

    pub fn apply_player_event(&mut self, event: &RotationPlayerEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.active_game_time += event.active_game_time;
        stats.tracked_time += event.tracked_time;
        stats.time_first_man += event.time_first_man;
        stats.time_second_man += event.time_second_man;
        stats.time_third_man += event.time_third_man;
        stats.time_ambiguous_role += event.time_ambiguous_role;
        stats.time_behind_play += event.time_behind_play;
        stats.time_level_with_play += event.time_level_with_play;
        stats.time_ahead_of_play += event.time_ahead_of_play;
        stats.longest_first_man_stint_time = stats
            .longest_first_man_stint_time
            .max(event.longest_first_man_stint_time);
        stats.first_man_stint_count += event.first_man_stint_count;
        stats.became_first_man_count += event.became_first_man_count;
        stats.lost_first_man_count += event.lost_first_man_count;
        stats.current_role_state = event.current_role_state;
        stats.current_depth_state = event.current_depth_state;
    }

    pub fn apply_team_event(&mut self, event: &RotationTeamEvent) {
        let stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        stats.first_man_changes_for_team += event.first_man_changes_for_team;
        stats.rotation_count += event.rotation_count;
    }
}
