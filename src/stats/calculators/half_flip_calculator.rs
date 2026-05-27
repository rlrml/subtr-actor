use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct HalfFlipCalculator {
    pub(super) player_stats: HashMap<PlayerId, HalfFlipStats>,
    pub(super) events: Vec<HalfFlipEvent>,
    pub(super) active_candidates: HashMap<PlayerId, ActiveHalfFlipCandidate>,
    pub(super) previous_dodge_active: HashMap<PlayerId, bool>,
    pub(super) current_last_half_flip_player: Option<PlayerId>,
}

impl HalfFlipCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, HalfFlipStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[HalfFlipEvent] {
        &self.events
    }

    pub(super) fn normalize_score(value: f32, min_value: f32, max_value: f32) -> f32 {
        if max_value <= min_value {
            return 0.0;
        }
        ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
    }

    pub(super) fn horizontal_velocity(player: &PlayerSample) -> Option<glam::Vec2> {
        let velocity = player.velocity()?.truncate();
        (velocity.length_squared() > f32::EPSILON).then_some(velocity)
    }

    pub(super) fn forward_vector(player: &PlayerSample) -> Option<glam::Vec3> {
        let rigid_body = player.rigid_body.as_ref()?;
        Some(quat_to_glam(&rigid_body.rotation) * glam::Vec3::X)
    }

    pub(super) fn forward_xy(player: &PlayerSample) -> Option<glam::Vec2> {
        let forward_xy = Self::forward_vector(player)?.truncate().normalize_or_zero();
        (forward_xy.length_squared() > f32::EPSILON).then_some(forward_xy)
    }
}
