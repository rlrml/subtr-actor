use super::*;

pub(crate) const BALL_CARRY_KIND_LABELS: [StatLabel; 2] = [
    StatLabel::new("kind", "carry"),
    StatLabel::new("kind", "air_dribble"),
];

pub(crate) fn ball_carry_kind_label(kind: BallCarryKind) -> StatLabel {
    match kind {
        BallCarryKind::Carry => StatLabel::new("kind", "carry"),
        BallCarryKind::AirDribble => StatLabel::new("kind", "air_dribble"),
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BallCarryStats {
    pub carry_count: u32,
    pub total_carry_time: f32,
    pub total_straight_line_distance: f32,
    pub total_path_distance: f32,
    pub longest_carry_time: f32,
    pub furthest_carry_distance: f32,
    pub fastest_carry_speed: f32,
    pub carry_speed_sum: f32,
    pub average_horizontal_gap_sum: f32,
    pub average_vertical_gap_sum: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

impl BallCarryStats {
    fn pct_count_average(&self, value: f32) -> f32 {
        if self.carry_count == 0 {
            0.0
        } else {
            value / self.carry_count as f32
        }
    }

    pub fn average_carry_time(&self) -> f32 {
        self.pct_count_average(self.total_carry_time)
    }

    pub fn average_straight_line_distance(&self) -> f32 {
        self.pct_count_average(self.total_straight_line_distance)
    }

    pub fn average_path_distance(&self) -> f32 {
        self.pct_count_average(self.total_path_distance)
    }

    pub fn average_carry_speed(&self) -> f32 {
        self.pct_count_average(self.carry_speed_sum)
    }

    pub fn average_horizontal_gap(&self) -> f32 {
        self.pct_count_average(self.average_horizontal_gap_sum)
    }

    pub fn average_vertical_gap(&self) -> f32 {
        self.pct_count_average(self.average_vertical_gap_sum)
    }

    fn record_event(&mut self, event: &BallCarryEvent) {
        self.labeled_event_counts
            .increment([ball_carry_kind_label(event.kind)]);
        self.carry_count = self.labeled_event_counts.total();
    }

    pub fn event_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_event_counts.count_matching(labels)
    }

    pub fn complete_labeled_event_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[&BALL_CARRY_KIND_LABELS],
            &self.labeled_event_counts,
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BallCarryStatsAccumulator {
    player_stats: HashMap<PlayerId, BallCarryStats>,
    player_air_dribble_stats: HashMap<PlayerId, AirDribbleStats>,
    team_zero_stats: BallCarryStats,
    team_one_stats: BallCarryStats,
    team_zero_air_dribble_stats: AirDribbleStats,
    team_one_air_dribble_stats: AirDribbleStats,
}

impl BallCarryStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, BallCarryStats> {
        &self.player_stats
    }

    pub fn player_air_dribble_stats(&self) -> &HashMap<PlayerId, AirDribbleStats> {
        &self.player_air_dribble_stats
    }

    pub fn team_zero_stats(&self) -> &BallCarryStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &BallCarryStats {
        &self.team_one_stats
    }

    pub fn team_zero_air_dribble_stats(&self) -> &AirDribbleStats {
        &self.team_zero_air_dribble_stats
    }

    pub fn team_one_air_dribble_stats(&self) -> &AirDribbleStats {
        &self.team_one_air_dribble_stats
    }

    pub fn apply_event(&mut self, event: &BallCarryEvent) {
        match event.kind {
            BallCarryKind::Carry => {
                let player_stats = self
                    .player_stats
                    .entry(event.player_id.clone())
                    .or_default();
                Self::apply_carry_event(player_stats, event);

                let team_stats = if event.is_team_0 {
                    &mut self.team_zero_stats
                } else {
                    &mut self.team_one_stats
                };
                Self::apply_carry_event(team_stats, event);
            }
            BallCarryKind::AirDribble => {
                let player_stats = self
                    .player_air_dribble_stats
                    .entry(event.player_id.clone())
                    .or_default();
                AirDribblePolicy::apply_event(player_stats, event);

                let team_stats = if event.is_team_0 {
                    &mut self.team_zero_air_dribble_stats
                } else {
                    &mut self.team_one_air_dribble_stats
                };
                AirDribblePolicy::apply_event(team_stats, event);
            }
        }
    }

    fn apply_carry_event(stats: &mut BallCarryStats, event: &BallCarryEvent) {
        stats.record_event(event);
        stats.total_carry_time += event.duration;
        stats.total_straight_line_distance += event.straight_line_distance;
        stats.total_path_distance += event.path_distance;
        stats.longest_carry_time = stats.longest_carry_time.max(event.duration);
        stats.furthest_carry_distance = stats
            .furthest_carry_distance
            .max(event.straight_line_distance);
        stats.fastest_carry_speed = stats.fastest_carry_speed.max(event.average_speed);
        stats.carry_speed_sum += event.average_speed;
        stats.average_horizontal_gap_sum += event.average_horizontal_gap;
        stats.average_vertical_gap_sum += event.average_vertical_gap;
    }
}
