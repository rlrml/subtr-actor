use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MustyFlickCalculator {
    pub(super) player_stats: HashMap<PlayerId, MustyFlickStats>,
    pub(super) events: Vec<MustyFlickEvent>,
    pub(super) recent_dodge_starts: HashMap<PlayerId, RecentDodgeStart>,
    pub(super) previous_dodge_active: HashMap<PlayerId, bool>,
    pub(super) previous_ball_velocity: Option<glam::Vec3>,
    pub(super) current_last_musty_player: Option<PlayerId>,
}

impl MustyFlickCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, MustyFlickStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[MustyFlickEvent] {
        &self.events
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
