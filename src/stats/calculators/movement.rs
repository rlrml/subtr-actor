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
struct MovementClassification {
    speed_band: MovementSpeedBand,
    height_band: PlayerVerticalBand,
}

impl MovementClassification {
    fn labels(self) -> [StatLabel; 2] {
        [self.speed_band.as_label(), self.height_band.as_label()]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
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
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
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
        let mut entries: Vec<_> = ALL_PLAYER_VERTICAL_BANDS
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
pub struct MovementCalculator {
    player_stats: HashMap<PlayerId, MovementStats>,
    player_teams: HashMap<PlayerId, bool>,
    previous_positions: HashMap<PlayerId, glam::Vec3>,
    team_zero_stats: MovementStats,
    team_one_stats: MovementStats,
}

impl MovementCalculator {
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

    fn classify_movement(speed: f32, height_band: PlayerVerticalBand) -> MovementClassification {
        let speed_band = if speed >= SUPERSONIC_SPEED_THRESHOLD {
            MovementSpeedBand::Supersonic
        } else if speed >= BOOST_SPEED_THRESHOLD {
            MovementSpeedBand::Boost
        } else {
            MovementSpeedBand::Slow
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
            PlayerVerticalBand::Ground => stats.time_on_ground += dt,
            PlayerVerticalBand::LowAir => stats.time_low_air += dt,
            PlayerVerticalBand::HighAir => stats.time_high_air += dt,
        }

        stats.labeled_tracked_time.add(classification.labels(), dt);
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        vertical_state: &PlayerVerticalState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        if frame.dt == 0.0 {
            for player in &players.players {
                if let Some(position) = player.position() {
                    self.previous_positions
                        .insert(player.player_id.clone(), position);
                }
            }
            return Ok(());
        }

        for player in &players.players {
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
                stats.tracked_time += frame.dt;
                stats.speed_integral += speed * frame.dt;
                team_stats.tracked_time += frame.dt;
                team_stats.speed_integral += speed * frame.dt;

                if let Some(previous_position) = self.previous_positions.get(&player.player_id) {
                    let distance = position.distance(*previous_position);
                    stats.total_distance += distance;
                    team_stats.total_distance += distance;
                }

                let height_band = vertical_state
                    .band_for_player(&player.player_id)
                    .unwrap_or_else(|| PlayerVerticalBand::from_height(position.z));
                let classification = Self::classify_movement(speed, height_band);
                Self::apply_classification(stats, classification, frame.dt);
                Self::apply_classification(team_stats, classification, frame.dt);
            }

            self.previous_positions
                .insert(player.player_id.clone(), position);
        }

        Ok(())
    }
}
