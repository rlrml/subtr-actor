use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MovementSpeedBand {
    Slow,
    Boost,
    Supersonic,
}

const ALL_MOVEMENT_SPEED_BANDS: [MovementSpeedBand; 3] = [
    MovementSpeedBand::Slow,
    MovementSpeedBand::Boost,
    MovementSpeedBand::Supersonic,
];

impl MovementSpeedBand {
    fn as_label(self) -> StatLabel {
        let value = match self {
            Self::Slow => "slow",
            Self::Boost => "boost",
            Self::Supersonic => "supersonic",
        };
        StatLabel::new("speed_band", value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MovementHeightBand {
    Ground,
    LowAir,
    HighAir,
}

const ALL_MOVEMENT_HEIGHT_BANDS: [MovementHeightBand; 3] = [
    MovementHeightBand::Ground,
    MovementHeightBand::LowAir,
    MovementHeightBand::HighAir,
];

impl MovementHeightBand {
    fn as_label(self) -> StatLabel {
        let value = match self {
            Self::Ground => "ground",
            Self::LowAir => "low_air",
            Self::HighAir => "high_air",
        };
        StatLabel::new("height_band", value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MovementClassification {
    speed_band: MovementSpeedBand,
    height_band: MovementHeightBand,
}

impl MovementClassification {
    fn labels(self) -> [StatLabel; 2] {
        [self.speed_band.as_label(), self.height_band.as_label()]
    }
}

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
    #[serde(skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_tracked_time: LabeledFloatSums,
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

    pub fn tracked_time_with_labels(&self, labels: &[StatLabel]) -> f32 {
        self.labeled_tracked_time.sum_matching(labels)
    }

    pub fn complete_labeled_tracked_time(&self) -> LabeledFloatSums {
        let mut entries: Vec<_> = ALL_MOVEMENT_HEIGHT_BANDS
            .into_iter()
            .flat_map(|height_band| {
                ALL_MOVEMENT_SPEED_BANDS.into_iter().map(move |speed_band| {
                    let mut labels = vec![speed_band.as_label(), height_band.as_label()];
                    labels.sort();
                    LabeledFloatSumEntry {
                        value: self.labeled_tracked_time.sum_exact(&labels),
                        labels,
                    }
                })
            })
            .collect();

        entries.sort_by(|left, right| left.labels.cmp(&right.labels));

        LabeledFloatSums { entries }
    }

    pub fn with_complete_labeled_tracked_time(mut self) -> Self {
        self.labeled_tracked_time = self.complete_labeled_tracked_time();
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct MovementReducer {
    player_stats: HashMap<PlayerId, MovementStats>,
    player_teams: HashMap<PlayerId, bool>,
    previous_positions: HashMap<PlayerId, glam::Vec3>,
    team_zero_stats: MovementStats,
    team_one_stats: MovementStats,
    live_play_tracker: LivePlayTracker,
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

    fn classify_movement(speed: f32, height: f32) -> MovementClassification {
        let speed_band = if speed >= SUPERSONIC_SPEED_THRESHOLD {
            MovementSpeedBand::Supersonic
        } else if speed >= BOOST_SPEED_THRESHOLD {
            MovementSpeedBand::Boost
        } else {
            MovementSpeedBand::Slow
        };

        let height_band = if height <= GROUND_Z_THRESHOLD {
            MovementHeightBand::Ground
        } else if height >= HIGH_AIR_Z_THRESHOLD {
            MovementHeightBand::HighAir
        } else {
            MovementHeightBand::LowAir
        };

        MovementClassification {
            speed_band,
            height_band,
        }
    }

    fn apply_classification(
        stats: &mut MovementStats,
        classification: MovementClassification,
        dt: f32,
    ) {
        match classification.speed_band {
            MovementSpeedBand::Slow => stats.time_slow_speed += dt,
            MovementSpeedBand::Boost => stats.time_boost_speed += dt,
            MovementSpeedBand::Supersonic => stats.time_supersonic_speed += dt,
        }

        match classification.height_band {
            MovementHeightBand::Ground => stats.time_on_ground += dt,
            MovementHeightBand::LowAir => stats.time_low_air += dt,
            MovementHeightBand::HighAir => stats.time_high_air += dt,
        }

        stats.labeled_tracked_time.add(classification.labels(), dt);
    }
}

impl StatsReducer for MovementReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        let live_play = self.live_play_tracker.is_live_play(sample);
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

                let classification = Self::classify_movement(speed, position.z);
                Self::apply_classification(stats, classification, sample.dt);
                Self::apply_classification(team_stats, classification, sample.dt);
            }

            self.previous_positions
                .insert(player.player_id.clone(), position);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use boxcars::RemoteId;

    use super::*;

    fn rigid_body(x: f32, y: f32, z: f32, vx: f32) -> boxcars::RigidBody {
        boxcars::RigidBody {
            sleeping: false,
            location: boxcars::Vector3f { x, y, z },
            rotation: boxcars::Quaternion {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            linear_velocity: Some(boxcars::Vector3f {
                x: vx,
                y: 0.0,
                z: 0.0,
            }),
            angular_velocity: Some(boxcars::Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
        }
    }

    fn sample(frame_number: usize, time: f32, z: f32, vx: f32) -> StatsSample {
        StatsSample {
            frame_number,
            time,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: Some([1, 1]),
            ball: None,
            players: vec![PlayerSample {
                player_id: RemoteId::Steam(1),
                is_team_0: true,
                rigid_body: Some(rigid_body(frame_number as f32, 0.0, z, vx)),
                boost_amount: None,
                last_boost_amount: None,
                boost_active: false,
                dodge_active: false,
                powerslide_active: false,
                match_goals: None,
                match_assists: None,
                match_saves: None,
                match_shots: None,
                match_score: None,
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        }
    }

    #[test]
    fn movement_reducer_tracks_labeled_time_bands() {
        let mut reducer = MovementReducer::new();

        reducer.on_sample(&sample(0, 0.0, 0.0, 200.0)).unwrap();
        reducer.on_sample(&sample(1, 1.0, 0.0, 200.0)).unwrap();
        reducer.on_sample(&sample(2, 2.0, 300.0, 1600.0)).unwrap();
        reducer.on_sample(&sample(3, 3.0, 900.0, 2400.0)).unwrap();

        let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
        assert_eq!(stats.tracked_time, 4.0);
        assert_eq!(
            stats.tracked_time_with_labels(&[StatLabel::new("speed_band", "slow")]),
            2.0
        );
        assert_eq!(
            stats.tracked_time_with_labels(&[StatLabel::new("height_band", "ground")]),
            2.0
        );
        assert_eq!(
            stats.tracked_time_with_labels(&[
                StatLabel::new("speed_band", "boost"),
                StatLabel::new("height_band", "low_air"),
            ]),
            1.0
        );
        assert_eq!(
            stats.tracked_time_with_labels(&[
                StatLabel::new("speed_band", "supersonic"),
                StatLabel::new("height_band", "high_air"),
            ]),
            1.0
        );
    }

    #[test]
    fn movement_stats_complete_labeled_time_adds_zero_entries() {
        let mut stats = MovementStats::default();
        stats.labeled_tracked_time.add(
            [
                StatLabel::new("speed_band", "boost"),
                StatLabel::new("height_band", "low_air"),
            ],
            1.25,
        );

        let completed = stats.complete_labeled_tracked_time();

        assert_eq!(completed.entries.len(), 9);
        assert_eq!(
            completed.sum_exact(&[
                StatLabel::new("speed_band", "boost"),
                StatLabel::new("height_band", "low_air"),
            ]),
            1.25
        );
        assert_eq!(
            completed.sum_exact(&[
                StatLabel::new("speed_band", "slow"),
                StatLabel::new("height_band", "ground"),
            ]),
            0.0
        );
    }
}
