use super::*;

const DEFAULT_FIRST_MAN_STINT_END_GRACE_SECONDS: f32 = 0.35;

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

#[derive(Debug, Clone, PartialEq)]
pub struct RotationStatsAccumulator {
    player_stats: HashMap<PlayerId, RotationPlayerStats>,
    team_zero_stats: RotationTeamStats,
    team_one_stats: RotationTeamStats,
    first_man_stints: HashMap<PlayerId, FirstManStintAccumulatorState>,
    first_man_stint_end_grace_seconds: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct FirstManStintAccumulatorState {
    active: bool,
    current_first_man_time: f32,
    non_first_man_seconds: f32,
}

impl Default for RotationStatsAccumulator {
    fn default() -> Self {
        Self {
            player_stats: HashMap::default(),
            team_zero_stats: RotationTeamStats::default(),
            team_one_stats: RotationTeamStats::default(),
            first_man_stints: HashMap::default(),
            first_man_stint_end_grace_seconds: DEFAULT_FIRST_MAN_STINT_END_GRACE_SECONDS,
        }
    }
}

impl RotationStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_first_man_stint_end_grace_seconds(first_man_stint_end_grace_seconds: f32) -> Self {
        Self {
            first_man_stint_end_grace_seconds,
            ..Self::default()
        }
    }

    pub fn set_first_man_stint_end_grace_seconds(
        &mut self,
        first_man_stint_end_grace_seconds: f32,
    ) {
        self.first_man_stint_end_grace_seconds = first_man_stint_end_grace_seconds;
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
        {
            let stats = self.player_stats.entry(event.player.clone()).or_default();
            if event.active {
                stats.active_game_time += event.duration;
                stats.tracked_time += event.duration;
                match event.current_role_state {
                    RoleState::FirstMan => stats.time_first_man += event.duration,
                    RoleState::SecondMan => stats.time_second_man += event.duration,
                    RoleState::ThirdMan => stats.time_third_man += event.duration,
                    RoleState::Ambiguous => stats.time_ambiguous_role += event.duration,
                    RoleState::Unknown => {}
                }
                match event.current_depth_state {
                    PlayDepthState::BehindPlay => stats.time_behind_play += event.duration,
                    PlayDepthState::LevelWithPlay => stats.time_level_with_play += event.duration,
                    PlayDepthState::AheadOfPlay => stats.time_ahead_of_play += event.duration,
                    PlayDepthState::Unknown => {}
                }
            }
        }
        self.apply_first_man_stint(&event.player, event.current_role_state, event.duration);
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.current_role_state = event.current_role_state;
        stats.current_depth_state = event.current_depth_state;
    }

    pub fn apply_team_event(&mut self, event: &RotationTeamEvent) {
        let stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        stats.first_man_changes_for_team += 1;
        stats.rotation_count += 1;
        self.player_stats
            .entry(event.previous_first_man.clone())
            .or_default()
            .lost_first_man_count += 1;
        self.player_stats
            .entry(event.next_first_man.clone())
            .or_default()
            .became_first_man_count += 1;
    }

    fn apply_first_man_stint(
        &mut self,
        player_id: &PlayerId,
        role_state: RoleState,
        duration: f32,
    ) {
        let stats = self.player_stats.entry(player_id.clone()).or_default();
        let state = self.first_man_stints.entry(player_id.clone()).or_default();
        if role_state == RoleState::FirstMan {
            if !state.active {
                state.active = true;
                state.current_first_man_time = 0.0;
                stats.first_man_stint_count += 1;
            }
            state.current_first_man_time += duration;
            state.non_first_man_seconds = 0.0;
            stats.longest_first_man_stint_time = stats
                .longest_first_man_stint_time
                .max(state.current_first_man_time);
            return;
        }

        if state.active {
            state.non_first_man_seconds += duration;
            if state.non_first_man_seconds > self.first_man_stint_end_grace_seconds {
                state.active = false;
                state.current_first_man_time = 0.0;
                state.non_first_man_seconds = 0.0;
            }
        }
    }
}
