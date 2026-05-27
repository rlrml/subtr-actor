use super::*;

pub(crate) fn apply_movement_stats(
    stats: &mut MovementStats,
    dt: f32,
    speed: f32,
    distance: f32,
    classification: MovementClassification,
) {
    stats.tracked_time += dt;
    stats.speed_integral += speed * dt;
    if distance > 0.0 {
        stats.total_distance += distance;
    }
    MovementCalculator::apply_classification(stats, classification, dt);
}

impl MovementCalculator {
    pub(super) fn apply_classification(
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
}

pub(crate) fn movement_event(
    frame: &FrameInfo,
    player: &PlayerSample,
    speed: f32,
    distance: f32,
    classification: MovementClassification,
) -> MovementEvent {
    MovementEvent {
        time: frame.time,
        frame: frame.frame_number,
        player: player.player_id.clone(),
        is_team_0: player.is_team_0,
        dt: frame.dt,
        speed,
        distance,
        speed_band: classification.speed_band.as_label_value().to_owned(),
        height_band: classification.height_band.as_label().value.to_owned(),
    }
}
