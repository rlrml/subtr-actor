use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CeilingShotCalculator {
    pub(super) player_stats: HashMap<PlayerId, CeilingShotStats>,
    pub(super) events: Vec<CeilingShotEvent>,
    pub(super) recent_ceiling_contacts: HashMap<PlayerId, RecentCeilingContact>,
    pub(super) previous_ball_velocity: Option<glam::Vec3>,
    pub(super) current_last_ceiling_shot_player: Option<PlayerId>,
}

impl CeilingShotCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, CeilingShotStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[CeilingShotEvent] {
        &self.events
    }

    pub(super) fn normalize_score(value: f32, min_value: f32, max_value: f32) -> f32 {
        if max_value <= min_value {
            return 0.0;
        }

        ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
    }

    pub(super) fn ball_speed_change(
        frame: &FrameInfo,
        ball: &BallFrameState,
        previous_ball_velocity: Option<glam::Vec3>,
    ) -> f32 {
        const BALL_GRAVITY_Z: f32 = -650.0;

        let Some(ball) = ball.sample() else {
            return 0.0;
        };
        let Some(previous_ball_velocity) = previous_ball_velocity else {
            return 0.0;
        };

        let expected_linear_delta = glam::Vec3::new(0.0, 0.0, BALL_GRAVITY_Z * frame.dt.max(0.0));
        let residual_linear_impulse =
            ball.velocity() - previous_ball_velocity - expected_linear_delta;
        residual_linear_impulse.length()
    }
}
