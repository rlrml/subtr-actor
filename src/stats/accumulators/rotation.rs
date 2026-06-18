use super::*;

/// Per-player accumulated rotation stats: time per man-role and first-man stints.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationPlayerStats {
    pub active_game_time: f32,
    pub time_first_man: f32,
    pub time_second_man: f32,
    pub time_third_man: f32,
    pub time_ambiguous_role: f32,
    pub longest_first_man_stint_time: f32,
    pub first_man_stint_count: u32,
    pub became_first_man_count: u32,
    pub lost_first_man_count: u32,
    pub current_role_state: RoleState,
}

impl RotationPlayerStats {
    fn role_pct(&self, value: f32) -> f32 {
        if self.active_game_time == 0.0 {
            0.0
        } else {
            value * 100.0 / self.active_game_time
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

    pub fn average_first_man_stint_time(&self) -> f32 {
        if self.first_man_stint_count == 0 {
            0.0
        } else {
            self.time_first_man / self.first_man_stint_count as f32
        }
    }
}

/// Per-team accumulated rotation stats.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationTeamStats {
    pub first_man_changes_for_team: u32,
    pub rotation_count: u32,
}

/// Rebuilds [`RotationPlayerStats`] / [`RotationTeamStats`] from rotation role
/// spans and first-man change events. First-man stints are derived here rather
/// than shipped as their own event stream: a stint continues while the gap
/// between consecutive first-man spans stays within the configured grace.
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
    current_stint_time: f32,
    last_first_man_end_time: Option<f32>,
}

impl Default for RotationStatsAccumulator {
    fn default() -> Self {
        Self::with_first_man_stint_end_grace_seconds(
            RotationCalculatorConfig::default().first_man_stint_end_grace_seconds,
        )
    }
}

impl RotationStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_first_man_stint_end_grace_seconds(first_man_stint_end_grace_seconds: f32) -> Self {
        Self {
            player_stats: HashMap::default(),
            team_zero_stats: RotationTeamStats::default(),
            team_one_stats: RotationTeamStats::default(),
            first_man_stints: HashMap::default(),
            first_man_stint_end_grace_seconds,
        }
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

    /// Apply one role span (or a partial extension of one — events for a
    /// player must arrive in chronological order).
    pub fn apply_role_event(&mut self, event: &RotationRoleEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.active_game_time += event.duration;
        match event.state {
            RoleState::FirstMan => stats.time_first_man += event.duration,
            RoleState::SecondMan => stats.time_second_man += event.duration,
            RoleState::ThirdMan => stats.time_third_man += event.duration,
            RoleState::Ambiguous => stats.time_ambiguous_role += event.duration,
            RoleState::Unknown => {}
        }
        stats.current_role_state = event.state;

        if event.state == RoleState::FirstMan {
            let stint = self
                .first_man_stints
                .entry(event.player.clone())
                .or_default();
            let continues_stint = stint
                .last_first_man_end_time
                .is_some_and(|end| event.time - end <= self.first_man_stint_end_grace_seconds);
            if continues_stint {
                stint.current_stint_time += event.duration;
            } else {
                stint.current_stint_time = event.duration;
                stats.first_man_stint_count += 1;
            }
            stint.last_first_man_end_time = Some(event.end_time);
            stats.longest_first_man_stint_time = stats
                .longest_first_man_stint_time
                .max(stint.current_stint_time);
        }
    }

    pub fn apply_first_man_change_event(&mut self, event: &FirstManChangeEvent) {
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
}
