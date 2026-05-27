use super::*;

pub(crate) struct BallDepthPositioningSample {
    pub(crate) dt: f32,
    pub(crate) level_margin: f32,
    pub(crate) is_team_0: bool,
    pub(crate) previous_position: glam::Vec3,
    pub(crate) position: glam::Vec3,
    pub(crate) previous_ball_position: glam::Vec3,
    pub(crate) ball_position: glam::Vec3,
}

pub(crate) fn record_ball_depth_positioning(
    stats: &mut PositioningStats,
    delta: &mut PositioningEvent,
    sample: BallDepthPositioningSample,
) {
    let previous_delta = normalized_y(sample.is_team_0, sample.previous_position)
        - normalized_y(sample.is_team_0, sample.previous_ball_position);
    let current_delta = normalized_y(sample.is_team_0, sample.position)
        - normalized_y(sample.is_team_0, sample.ball_position);
    let (behind_ball, level_ball, in_front_ball) =
        ball_depth_fractions(sample.level_margin, previous_delta, current_delta);
    stats.time_behind_ball += sample.dt * behind_ball;
    stats.time_level_with_ball += sample.dt * level_ball;
    stats.time_in_front_of_ball += sample.dt * in_front_ball;
    delta.time_behind_ball += sample.dt * behind_ball;
    delta.time_level_with_ball += sample.dt * level_ball;
    delta.time_in_front_of_ball += sample.dt * in_front_ball;
}

pub(crate) fn ball_depth_fractions(
    level_margin: f32,
    start_delta: f32,
    end_delta: f32,
) -> (f32, f32, f32) {
    let behind_fraction = interval_fraction_below_threshold(start_delta, end_delta, -level_margin);
    let level_fraction =
        interval_fraction_in_scalar_range(start_delta, end_delta, -level_margin, level_margin);
    let in_front_fraction = (1.0 - behind_fraction - level_fraction).clamp(0.0, 1.0);
    (behind_fraction, level_fraction, in_front_fraction)
}
