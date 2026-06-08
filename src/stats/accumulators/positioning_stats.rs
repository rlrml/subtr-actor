use super::*;

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
    pub time_closest_to_ball: f32,
    pub time_closest_to_ball_team: f32,
    pub time_closest_to_ball_absolute: f32,
    pub time_farthest_from_ball: f32,
    pub time_behind_ball: f32,
    pub time_level_with_ball: f32,
    pub time_in_front_of_ball: f32,
    pub times_caught_ahead_of_play_on_conceded_goals: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PositioningTeamStats {
    pub tracked_time: f32,
    pub time_closest_to_ball: f32,
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

    pub fn closest_to_ball_pct(&self) -> f32 {
        self.pct(self.time_closest_to_ball)
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

    pub fn defensive_zone_pct(&self) -> f32 {
        self.defensive_third_pct()
    }

    pub fn neutral_zone_pct(&self) -> f32 {
        self.neutral_third_pct()
    }

    pub fn offensive_zone_pct(&self) -> f32 {
        self.offensive_third_pct()
    }

    pub fn defensive_half_pct(&self) -> f32 {
        self.pct(self.time_defensive_half)
    }

    pub fn offensive_half_pct(&self) -> f32 {
        self.pct(self.time_offensive_half)
    }

    pub fn closest_to_ball_pct(&self) -> f32 {
        self.pct(self.time_closest_to_ball)
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

    pub fn apply_event(&mut self, event: &PositioningEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        if event.active {
            stats.active_game_time += event.duration;
        }
        if event.tracked {
            let team_stats = if event.is_team_0 {
                &mut self.team_zero_stats
            } else {
                &mut self.team_one_stats
            };
            if event.closest_to_ball_team || event.closest_to_ball {
                team_stats.tracked_time += event.duration;
                team_stats.time_closest_to_ball += event.duration;
                team_stats.time_closest_to_ball_team += event.duration;
            }
            if event.closest_to_ball_absolute {
                team_stats.time_closest_to_ball_absolute += event.duration;
            }

            stats.tracked_time += event.duration;
            if let Some(distance) = event.distance_to_teammates {
                stats.sum_distance_to_teammates += distance * event.duration;
            }
            if let Some(distance) = event.distance_to_ball {
                stats.sum_distance_to_ball += distance * event.duration;
                match event.possession_state {
                    PositioningPossessionState::HasPossession => {
                        stats.sum_distance_to_ball_has_possession += distance * event.duration;
                    }
                    PositioningPossessionState::NoPossession => {
                        stats.sum_distance_to_ball_no_possession += distance * event.duration;
                    }
                    PositioningPossessionState::Neutral => {}
                }
            }
            match event.possession_state {
                PositioningPossessionState::HasPossession => {
                    stats.time_has_possession += event.duration;
                }
                PositioningPossessionState::NoPossession => {
                    stats.time_no_possession += event.duration;
                }
                PositioningPossessionState::Neutral => {}
            }
            match event.teammate_role {
                PositioningTeammateRoleState::NoTeammates => {
                    stats.time_no_teammates += event.duration;
                }
                PositioningTeammateRoleState::MostBack => {
                    stats.time_most_back += event.duration;
                }
                PositioningTeammateRoleState::MostForward => {
                    stats.time_most_forward += event.duration;
                }
                PositioningTeammateRoleState::Mid => {
                    stats.time_mid_role += event.duration;
                }
                PositioningTeammateRoleState::Other => {
                    stats.time_other_role += event.duration;
                }
                PositioningTeammateRoleState::Unknown => {}
            }
            stats.time_defensive_zone += event.duration * event.defensive_zone_fraction;
            stats.time_neutral_zone += event.duration * event.neutral_zone_fraction;
            stats.time_offensive_zone += event.duration * event.offensive_zone_fraction;
            stats.time_defensive_half += event.duration * event.defensive_half_fraction;
            stats.time_offensive_half += event.duration * event.offensive_half_fraction;
            if event.closest_to_ball {
                stats.time_closest_to_ball += event.duration;
            }
            if event.closest_to_ball_team {
                stats.time_closest_to_ball_team += event.duration;
            }
            if event.closest_to_ball_absolute {
                stats.time_closest_to_ball_absolute += event.duration;
            }
            if event.farthest_from_ball {
                stats.time_farthest_from_ball += event.duration;
            }
            stats.time_behind_ball += event.duration * event.behind_ball_fraction;
            stats.time_level_with_ball += event.duration * event.level_with_ball_fraction;
            stats.time_in_front_of_ball += event.duration * event.in_front_of_ball_fraction;
        }
        if event.demolished {
            stats.time_demolished += event.duration;
        }
        if event.caught_ahead_of_play_on_conceded_goal {
            stats.times_caught_ahead_of_play_on_conceded_goals += 1;
        }
    }

    pub fn apply_events<'a>(&mut self, events: impl IntoIterator<Item = &'a PositioningEvent>) {
        for event in events {
            self.apply_event(event);
        }
    }

    pub fn apply_activity_event(&mut self, event: &PositioningActivityEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        if event.active {
            stats.active_game_time += event.duration;
        }
        if event.tracked {
            stats.tracked_time += event.duration;
        }
        if event.demolished {
            stats.time_demolished += event.duration;
        }
    }

    pub fn apply_distance_event(&mut self, event: &PositioningDistanceEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        if let Some(distance) = event.distance_to_teammates {
            stats.sum_distance_to_teammates += distance * event.duration;
        }
        if let Some(distance) = event.distance_to_ball {
            stats.sum_distance_to_ball += distance * event.duration;
            match event.possession_state {
                PositioningPossessionState::HasPossession => {
                    stats.sum_distance_to_ball_has_possession += distance * event.duration;
                }
                PositioningPossessionState::NoPossession => {
                    stats.sum_distance_to_ball_no_possession += distance * event.duration;
                }
                PositioningPossessionState::Neutral => {}
            }
        }
        match event.possession_state {
            PositioningPossessionState::HasPossession => {
                stats.time_has_possession += event.duration;
            }
            PositioningPossessionState::NoPossession => {
                stats.time_no_possession += event.duration;
            }
            PositioningPossessionState::Neutral => {}
        }
    }

    pub fn apply_field_zone_event(&mut self, event: &PositioningFieldZoneEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.time_defensive_zone += event.duration * event.defensive_zone_fraction;
        stats.time_neutral_zone += event.duration * event.neutral_zone_fraction;
        stats.time_offensive_zone += event.duration * event.offensive_zone_fraction;
        stats.time_defensive_half += event.duration * event.defensive_half_fraction;
        stats.time_offensive_half += event.duration * event.offensive_half_fraction;
    }

    pub fn apply_ball_depth_event(&mut self, event: &PositioningBallDepthEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.time_behind_ball += event.duration * event.behind_ball_fraction;
        stats.time_level_with_ball += event.duration * event.level_with_ball_fraction;
        stats.time_in_front_of_ball += event.duration * event.in_front_of_ball_fraction;
    }

    pub fn apply_teammate_role_event(&mut self, event: &PositioningTeammateRoleEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        match event.teammate_role {
            PositioningTeammateRoleState::NoTeammates => {
                stats.time_no_teammates += event.duration;
            }
            PositioningTeammateRoleState::MostBack => {
                stats.time_most_back += event.duration;
            }
            PositioningTeammateRoleState::MostForward => {
                stats.time_most_forward += event.duration;
            }
            PositioningTeammateRoleState::Mid => {
                stats.time_mid_role += event.duration;
            }
            PositioningTeammateRoleState::Other => {
                stats.time_other_role += event.duration;
            }
            PositioningTeammateRoleState::Unknown => {}
        }
    }

    pub fn apply_ball_proximity_event(&mut self, event: &PositioningBallProximityEvent) {
        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        if event.closest_to_ball_team {
            team_stats.tracked_time += event.duration;
            team_stats.time_closest_to_ball += event.duration;
            team_stats.time_closest_to_ball_team += event.duration;
            stats.time_closest_to_ball += event.duration;
            stats.time_closest_to_ball_team += event.duration;
        }
        if event.closest_to_ball_absolute {
            team_stats.time_closest_to_ball_absolute += event.duration;
            stats.time_closest_to_ball_absolute += event.duration;
        }
        if event.farthest_from_ball {
            stats.time_farthest_from_ball += event.duration;
        }
    }

    pub fn apply_goal_context_event(&mut self, event: &PositioningGoalContextEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        if event.caught_ahead_of_play_on_conceded_goal {
            stats.times_caught_ahead_of_play_on_conceded_goals += 1;
        }
    }
}

#[cfg(test)]
#[path = "positioning_stats_tests.rs"]
mod tests;
