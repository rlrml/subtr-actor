use super::*;

const AIR_DRIBBLE_MIN_BALL_Z: f32 = 300.0;
pub(crate) const AIR_DRIBBLE_MIN_PLAYER_Z: f32 = 100.0;
const AIR_DRIBBLE_MAX_HORIZONTAL_GAP: f32 = BALL_RADIUS_Z * 3.0;
const AIR_DRIBBLE_MAX_ABOVE_CAR_GAP: f32 = 360.0;
const AIR_DRIBBLE_MAX_BELOW_CAR_GAP: f32 = 100.0;
pub(crate) const AIR_DRIBBLE_MIN_DURATION: f32 = 0.65;
const AIR_DRIBBLE_MIN_TOUCHES: u32 = 3;
const AIR_DRIBBLE_MIN_AIR_TOUCHES: u32 = 2;
const WALL_TAKEOFF_MIN_Z: f32 = 120.0;
const SIDE_WALL_START_ABS_X: f32 = 3200.0;
const BACK_WALL_START_ABS_Y: f32 = 4600.0;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct AirDribbleStats {
    pub count: u32,
    #[serde(default)]
    pub ground_to_air_count: u32,
    #[serde(default)]
    pub wall_to_air_count: u32,
    #[serde(default)]
    pub total_touch_count: u32,
    #[serde(default)]
    pub max_touch_count: u32,
    pub total_time: f32,
    pub total_straight_line_distance: f32,
    pub total_path_distance: f32,
    pub longest_time: f32,
    pub furthest_distance: f32,
    pub fastest_speed: f32,
    pub speed_sum: f32,
    pub average_horizontal_gap_sum: f32,
    pub average_vertical_gap_sum: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

impl AirDribbleStats {
    fn count_average(&self, value: f32) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            value / self.count as f32
        }
    }

    pub fn average_time(&self) -> f32 {
        self.count_average(self.total_time)
    }

    pub fn average_straight_line_distance(&self) -> f32 {
        self.count_average(self.total_straight_line_distance)
    }

    pub fn average_path_distance(&self) -> f32 {
        self.count_average(self.total_path_distance)
    }

    pub fn average_speed(&self) -> f32 {
        self.count_average(self.speed_sum)
    }

    pub fn average_touch_count(&self) -> f32 {
        self.count_average(self.total_touch_count as f32)
    }

    pub fn average_horizontal_gap(&self) -> f32 {
        self.count_average(self.average_horizontal_gap_sum)
    }

    pub fn average_vertical_gap(&self) -> f32 {
        self.count_average(self.average_vertical_gap_sum)
    }

    fn record_event(&mut self, event: &BallCarryEvent) {
        if let Some(origin) = event.air_dribble_origin {
            self.labeled_event_counts
                .increment([air_dribble_origin_label(origin)]);
        }
        self.sync_legacy_counts();
    }

    pub fn event_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_event_counts.count_matching(labels)
    }

    pub fn complete_labeled_event_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[&AIR_DRIBBLE_ORIGIN_LABELS],
            &self.labeled_event_counts,
        )
    }

    fn sync_legacy_counts(&mut self) {
        self.count = self.labeled_event_counts.total();
        self.ground_to_air_count = self
            .event_count_with_labels(&[air_dribble_origin_label(AirDribbleOrigin::GroundToAir)]);
        self.wall_to_air_count =
            self.event_count_with_labels(&[air_dribble_origin_label(AirDribbleOrigin::WallToAir)]);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum AirDribbleOrigin {
    GroundToAir,
    WallToAir,
}

const AIR_DRIBBLE_ORIGIN_LABELS: [StatLabel; 2] = [
    StatLabel::new("origin", "ground_to_air"),
    StatLabel::new("origin", "wall_to_air"),
];

fn air_dribble_origin_label(origin: AirDribbleOrigin) -> StatLabel {
    StatLabel::new("origin", origin.as_label_value())
}

impl AirDribbleOrigin {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::GroundToAir => "ground_to_air",
            Self::WallToAir => "wall_to_air",
        }
    }
}

pub(crate) struct AirDribblePolicy;

impl AirDribblePolicy {
    pub(crate) fn is_sample(
        player_position: glam::Vec3,
        ball_position: glam::Vec3,
        horizontal_gap: f32,
        vertical_gap: f32,
    ) -> bool {
        ball_position.z >= AIR_DRIBBLE_MIN_BALL_Z
            && player_position.z >= AIR_DRIBBLE_MIN_PLAYER_Z
            && !player_is_on_wall(player_position)
            && horizontal_gap <= AIR_DRIBBLE_MAX_HORIZONTAL_GAP
            && (-AIR_DRIBBLE_MAX_BELOW_CAR_GAP..=AIR_DRIBBLE_MAX_ABOVE_CAR_GAP)
                .contains(&vertical_gap)
    }

    pub(crate) fn is_air_touch_position(player_position: glam::Vec3) -> bool {
        player_position.z > PLAYER_GROUND_Z_THRESHOLD && !player_is_on_wall(player_position)
    }

    pub(crate) fn kind_requires_airborne(kind: BallCarryKind) -> bool {
        kind == BallCarryKind::AirDribble
    }

    pub(crate) fn is_valid_sequence(
        sequence: &CompletedBallControlSequence<BallCarryKind>,
    ) -> bool {
        sequence.kind != BallCarryKind::AirDribble
            || (sequence.touch_count >= AIR_DRIBBLE_MIN_TOUCHES
                && sequence.air_touch_count >= AIR_DRIBBLE_MIN_AIR_TOUCHES)
    }

    pub(crate) fn origin(start_position: glam::Vec3) -> AirDribbleOrigin {
        if start_position.z >= WALL_TAKEOFF_MIN_Z
            && (start_position.x.abs() >= SIDE_WALL_START_ABS_X
                || start_position.y.abs() >= BACK_WALL_START_ABS_Y)
        {
            AirDribbleOrigin::WallToAir
        } else {
            AirDribbleOrigin::GroundToAir
        }
    }

    pub(crate) fn apply_event(stats: &mut AirDribbleStats, event: &BallCarryEvent) {
        stats.record_event(event);
        stats.total_time += event.duration;
        stats.total_straight_line_distance += event.straight_line_distance;
        stats.total_path_distance += event.path_distance;
        stats.longest_time = stats.longest_time.max(event.duration);
        stats.furthest_distance = stats.furthest_distance.max(event.straight_line_distance);
        stats.fastest_speed = stats.fastest_speed.max(event.average_speed);
        stats.speed_sum += event.average_speed;
        stats.average_horizontal_gap_sum += event.average_horizontal_gap;
        stats.average_vertical_gap_sum += event.average_vertical_gap;
        stats.total_touch_count += event.touch_count;
        stats.max_touch_count = stats.max_touch_count.max(event.touch_count);
    }
}

#[cfg(test)]
#[path = "air_dribble_tests.rs"]
mod tests;
