use super::*;

pub(super) fn flick_event_confidence(
    setup: &FlickSetupSummary,
    time_since_dodge: f32,
    ball_speed_change: f32,
    impulse_away_alignment: f32,
    vertical_impulse: f32,
) -> f32 {
    let timing_score = 1.0 - (time_since_dodge / FLICK_MAX_DODGE_TO_TOUCH_SECONDS).clamp(0.0, 1.0);
    let setup_duration_score = flick_normalize_score(setup.duration, FLICK_MIN_SETUP_SECONDS, 0.75);
    let horizontal_control_score =
        1.0 - (setup.average_horizontal_gap / FLICK_MAX_CONTROL_HORIZONTAL_GAP).clamp(0.0, 1.0);
    let vertical_control_score = 1.0
        - ((setup.average_vertical_gap - 110.0).abs() / FLICK_MAX_CONTROL_VERTICAL_GAP)
            .clamp(0.0, 1.0);
    let impulse_score =
        flick_normalize_score(ball_speed_change, FLICK_MIN_BALL_SPEED_CHANGE, 1450.0);
    let away_score = flick_normalize_score(
        impulse_away_alignment,
        FLICK_MIN_IMPULSE_AWAY_ALIGNMENT,
        0.85,
    );
    let vertical_score = flick_normalize_score(vertical_impulse, 100.0, 750.0);

    0.16 * timing_score
        + 0.19 * setup_duration_score
        + 0.12 * horizontal_control_score
        + 0.10 * vertical_control_score
        + 0.22 * impulse_score
        + 0.15 * away_score
        + 0.06 * vertical_score
}

fn flick_normalize_score(value: f32, min_value: f32, max_value: f32) -> f32 {
    if max_value <= min_value {
        return 0.0;
    }
    ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
}
