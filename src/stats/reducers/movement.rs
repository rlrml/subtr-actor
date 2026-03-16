use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct MovementStats {
    pub tracked_time: f32,
    pub total_distance: f32,
    pub speed_integral: f32,
    pub time_slow_speed: f32,
    pub time_boost_speed: f32,
    pub time_supersonic_speed: f32,
    pub time_on_ground: f32,
    pub time_low_air: f32,
    pub time_high_air: f32,
}

impl MovementStats {
    pub fn average_speed(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.speed_integral / self.tracked_time
        }
    }

    pub fn average_speed_pct(&self) -> f32 {
        self.average_speed() * 100.0 / CAR_MAX_SPEED
    }

    pub fn slow_speed_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_slow_speed * 100.0 / self.tracked_time
        }
    }

    pub fn boost_speed_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_boost_speed * 100.0 / self.tracked_time
        }
    }

    pub fn supersonic_speed_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_supersonic_speed * 100.0 / self.tracked_time
        }
    }

    pub fn on_ground_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_on_ground * 100.0 / self.tracked_time
        }
    }

    pub fn low_air_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_low_air * 100.0 / self.tracked_time
        }
    }

    pub fn high_air_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_high_air * 100.0 / self.tracked_time
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MovementReducer {
    player_stats: HashMap<PlayerId, MovementStats>,
    player_teams: HashMap<PlayerId, bool>,
    previous_positions: HashMap<PlayerId, glam::Vec3>,
    team_zero_stats: MovementStats,
    team_one_stats: MovementStats,
}

impl MovementReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, MovementStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &MovementStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &MovementStats {
        &self.team_one_stats
    }
}

impl StatsReducer for MovementReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        let live_play = sample.is_live_play();
        if sample.dt == 0.0 {
            for player in &sample.players {
                if let Some(position) = player.position() {
                    self.previous_positions
                        .insert(player.player_id.clone(), position);
                }
            }
            return Ok(());
        }

        for player in &sample.players {
            self.player_teams
                .insert(player.player_id.clone(), player.is_team_0);
            let Some(position) = player.position() else {
                continue;
            };
            let speed = player.speed().unwrap_or(0.0);
            let stats = self
                .player_stats
                .entry(player.player_id.clone())
                .or_default();
            let team_stats = if player.is_team_0 {
                &mut self.team_zero_stats
            } else {
                &mut self.team_one_stats
            };

            if live_play {
                stats.tracked_time += sample.dt;
                stats.speed_integral += speed * sample.dt;
                team_stats.tracked_time += sample.dt;
                team_stats.speed_integral += speed * sample.dt;

                if let Some(previous_position) = self.previous_positions.get(&player.player_id) {
                    let distance = position.distance(*previous_position);
                    stats.total_distance += distance;
                    team_stats.total_distance += distance;
                }

                if speed >= SUPERSONIC_SPEED_THRESHOLD {
                    stats.time_supersonic_speed += sample.dt;
                    team_stats.time_supersonic_speed += sample.dt;
                } else if speed >= BOOST_SPEED_THRESHOLD {
                    stats.time_boost_speed += sample.dt;
                    team_stats.time_boost_speed += sample.dt;
                } else {
                    stats.time_slow_speed += sample.dt;
                    team_stats.time_slow_speed += sample.dt;
                }

                if position.z <= GROUND_Z_THRESHOLD {
                    stats.time_on_ground += sample.dt;
                    team_stats.time_on_ground += sample.dt;
                } else if position.z >= HIGH_AIR_Z_THRESHOLD {
                    stats.time_high_air += sample.dt;
                    team_stats.time_high_air += sample.dt;
                } else {
                    stats.time_low_air += sample.dt;
                    team_stats.time_low_air += sample.dt;
                }
            }

            self.previous_positions
                .insert(player.player_id.clone(), position);
        }

        Ok(())
    }
}
