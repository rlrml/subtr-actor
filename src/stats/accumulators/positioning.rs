use super::*;

/// Per-player accumulated positioning stats: time in roles/zones and possession distances.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PositioningStats {
    pub active_game_time: f32,
    pub tracked_time: f32,
    pub sum_distance_to_teammates: f32,
    pub sum_distance_to_ball: f32,
    pub sum_distance_to_ball_has_possession: f32,
    pub time_has_possession: f32,
    pub sum_distance_to_ball_no_possession: f32,
    pub time_no_possession: f32,
    pub time_demolished: f32,
    pub time_no_teammates: f32,
    pub time_most_back: f32,
    pub time_most_forward: f32,
    pub time_mid_role: f32,
    pub time_other_role: f32,
    #[serde(rename = "time_defensive_third")]
    pub time_defensive_zone: f32,
    #[serde(rename = "time_neutral_third")]
    pub time_neutral_zone: f32,
    #[serde(rename = "time_offensive_third")]
    pub time_offensive_zone: f32,
    pub time_defensive_half: f32,
    pub time_offensive_half: f32,
    pub time_closest_to_ball_team: f32,
    pub time_closest_to_ball_absolute: f32,
    pub time_farthest_from_ball: f32,
    pub time_behind_ball: f32,
    pub time_level_with_ball: f32,
    pub time_in_front_of_ball: f32,
}

/// Per-team accumulated positioning stats.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PositioningTeamStats {
    pub tracked_time: f32,
    pub time_closest_to_ball_team: f32,
    pub time_closest_to_ball_absolute: f32,
}

impl PositioningTeamStats {
    fn pct(&self, value: f32) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            value * 100.0 / self.tracked_time
        }
    }

    pub fn closest_to_ball_team_pct(&self) -> f32 {
        self.pct(self.time_closest_to_ball_team)
    }

    pub fn closest_to_ball_absolute_pct(&self) -> f32 {
        self.pct(self.time_closest_to_ball_absolute)
    }
}

impl PositioningStats {
    pub fn average_distance_to_teammates(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.sum_distance_to_teammates / self.tracked_time
        }
    }

    pub fn average_distance_to_ball(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.sum_distance_to_ball / self.tracked_time
        }
    }

    pub fn average_distance_to_ball_has_possession(&self) -> f32 {
        if self.time_has_possession == 0.0 {
            0.0
        } else {
            self.sum_distance_to_ball_has_possession / self.time_has_possession
        }
    }

    pub fn average_distance_to_ball_no_possession(&self) -> f32 {
        if self.time_no_possession == 0.0 {
            0.0
        } else {
            self.sum_distance_to_ball_no_possession / self.time_no_possession
        }
    }

    fn pct(&self, value: f32) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            value * 100.0 / self.tracked_time
        }
    }

    pub fn most_back_pct(&self) -> f32 {
        self.pct(self.time_most_back)
    }

    pub fn most_forward_pct(&self) -> f32 {
        self.pct(self.time_most_forward)
    }

    pub fn mid_role_pct(&self) -> f32 {
        self.pct(self.time_mid_role)
    }

    pub fn other_role_pct(&self) -> f32 {
        self.pct(self.time_other_role)
    }

    pub fn defensive_third_pct(&self) -> f32 {
        self.pct(self.time_defensive_zone)
    }

    pub fn neutral_third_pct(&self) -> f32 {
        self.pct(self.time_neutral_zone)
    }

    pub fn offensive_third_pct(&self) -> f32 {
        self.pct(self.time_offensive_zone)
    }

    pub fn defensive_half_pct(&self) -> f32 {
        self.pct(self.time_defensive_half)
    }

    pub fn offensive_half_pct(&self) -> f32 {
        self.pct(self.time_offensive_half)
    }

    pub fn closest_to_ball_team_pct(&self) -> f32 {
        self.pct(self.time_closest_to_ball_team)
    }

    pub fn closest_to_ball_absolute_pct(&self) -> f32 {
        self.pct(self.time_closest_to_ball_absolute)
    }

    pub fn farthest_from_ball_pct(&self) -> f32 {
        self.pct(self.time_farthest_from_ball)
    }

    pub fn behind_ball_pct(&self) -> f32 {
        self.pct(self.time_behind_ball)
    }

    pub fn level_with_ball_pct(&self) -> f32 {
        self.pct(self.time_level_with_ball)
    }

    pub fn in_front_of_ball_pct(&self) -> f32 {
        self.pct(self.time_in_front_of_ball)
    }
}

/// Rebuilds [`PositioningStats`] from the per-facet event streams plus the
/// continuous distance signal. This is the only accumulation path — the native
/// projection and event-based playback reconstruction both run it, so they
/// agree by construction.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PositioningStatsAccumulator {
    player_stats: HashMap<PlayerId, PositioningStats>,
    team_zero_stats: PositioningTeamStats,
    team_one_stats: PositioningTeamStats,
}

impl PositioningStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, PositioningStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &PositioningTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &PositioningTeamStats {
        &self.team_one_stats
    }

    pub fn apply_activity_event(&mut self, event: &PlayerActivityEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.active_game_time += event.duration;
        match event.state {
            ActivityState::Tracked => stats.tracked_time += event.duration,
            ActivityState::Demolished => stats.time_demolished += event.duration,
        }
    }

    pub fn apply_field_third_event(&mut self, event: &FieldThirdEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        match event.state {
            FieldThirdState::Defensive => stats.time_defensive_zone += event.duration,
            FieldThirdState::Neutral => stats.time_neutral_zone += event.duration,
            FieldThirdState::Offensive => stats.time_offensive_zone += event.duration,
        }
    }

    pub fn apply_field_half_event(&mut self, event: &FieldHalfEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        match event.state {
            FieldHalfState::Defensive => stats.time_defensive_half += event.duration,
            FieldHalfState::Offensive => stats.time_offensive_half += event.duration,
        }
    }

    pub fn apply_ball_depth_event(&mut self, event: &BallDepthEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        match event.state {
            BallDepthState::BehindBall => stats.time_behind_ball += event.duration,
            BallDepthState::LevelWithBall => stats.time_level_with_ball += event.duration,
            BallDepthState::AheadOfBall => stats.time_in_front_of_ball += event.duration,
        }
    }

    pub fn apply_depth_role_event(&mut self, event: &DepthRoleEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        match event.state {
            DepthRoleState::NoTeammates => stats.time_no_teammates += event.duration,
            DepthRoleState::MostBack => stats.time_most_back += event.duration,
            DepthRoleState::MostForward => stats.time_most_forward += event.duration,
            DepthRoleState::Mid => stats.time_mid_role += event.duration,
            DepthRoleState::Other => stats.time_other_role += event.duration,
        }
    }

    pub fn apply_ball_proximity_event(&mut self, event: &BallProximityEvent) {
        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        if event.state.closest_to_ball_team {
            // Exactly one player per team is closest at any tracked moment, so
            // summing closest spans doubles as the team's tracked time.
            team_stats.tracked_time += event.duration;
            team_stats.time_closest_to_ball_team += event.duration;
            stats.time_closest_to_ball_team += event.duration;
        }
        if event.state.closest_to_ball_absolute {
            team_stats.time_closest_to_ball_absolute += event.duration;
            stats.time_closest_to_ball_absolute += event.duration;
        }
        if event.state.farthest_from_ball {
            stats.time_farthest_from_ball += event.duration;
        }
    }

    /// Seed the distance portion of a player's stats from the cumulative
    /// [`PositioningSignalSnapshot`]. Distance is a continuous magnitude rather than an event,
    /// so these fields are carried directly instead of being reconstructed from events.
    pub fn apply_signal(&mut self, player: &PlayerId, signal: &PositioningSignalSnapshot) {
        let stats = self.player_stats.entry(player.clone()).or_default();
        stats.sum_distance_to_teammates = signal.sum_distance_to_teammates;
        stats.sum_distance_to_ball = signal.sum_distance_to_ball;
        stats.sum_distance_to_ball_has_possession = signal.sum_distance_to_ball_has_possession;
        stats.time_has_possession = signal.time_has_possession;
        stats.sum_distance_to_ball_no_possession = signal.sum_distance_to_ball_no_possession;
        stats.time_no_possession = signal.time_no_possession;
    }
}

#[cfg(test)]
#[path = "positioning_stats_tests.rs"]
mod tests;
