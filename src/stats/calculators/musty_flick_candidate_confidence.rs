use super::*;

pub(super) struct MustyFlickConfidenceInputs {
    pub(super) time_since_dodge: f32,
    pub(super) rear_alignment: f32,
    pub(super) top_alignment: f32,
    pub(super) forward_approach_speed: f32,
    pub(super) pitch_rate: f32,
    pub(super) other_spin: f32,
    pub(super) ball_speed_change: f32,
    pub(super) dodge_start_forward_z: f32,
}

impl MustyFlickCandidateMetrics {
    pub(super) fn confidence(inputs: MustyFlickConfidenceInputs) -> f32 {
        let timing_score =
            (1.0 - inputs.time_since_dodge / MUSTY_MAX_DODGE_TO_TOUCH_SECONDS).clamp(0.0, 1.0);
        let rear_score =
            ((inputs.rear_alignment - MUSTY_MIN_REAR_ALIGNMENT) / 0.70).clamp(0.0, 1.0);
        let top_score = ((inputs.top_alignment - MUSTY_MIN_TOP_ALIGNMENT) / 0.70).clamp(0.0, 1.0);
        let approach_score = ((inputs.forward_approach_speed - MUSTY_MIN_FORWARD_APPROACH_SPEED)
            / 900.0)
            .clamp(0.0, 1.0);
        let pitch_score = ((inputs.pitch_rate - MUSTY_MIN_PITCH_RATE) / 8.0).clamp(0.0, 1.0);
        let pitch_dominance_score = pitch_dominance_score(inputs.pitch_rate, inputs.other_spin);
        let impulse_score =
            ((inputs.ball_speed_change - MUSTY_MIN_BALL_SPEED_CHANGE) / 900.0).clamp(0.0, 1.0);
        let setup_score = ((inputs.dodge_start_forward_z - MUSTY_MIN_DODGE_START_FORWARD_Z)
            / 1.25)
            .clamp(0.0, 1.0);

        0.17 * timing_score
            + 0.17 * rear_score
            + 0.14 * top_score
            + 0.15 * approach_score
            + 0.12 * pitch_score
            + 0.08 * pitch_dominance_score
            + 0.10 * impulse_score
            + 0.07 * setup_score
    }
}

fn pitch_dominance_score(pitch_rate: f32, other_spin: f32) -> f32 {
    let pitch_dominance_ratio = if other_spin <= f32::EPSILON {
        pitch_rate
    } else {
        pitch_rate / other_spin
    };
    ((pitch_dominance_ratio - MUSTY_MIN_PITCH_DOMINANCE_RATIO) / 2.5).clamp(0.0, 1.0)
}
