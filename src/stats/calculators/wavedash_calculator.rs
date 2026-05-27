use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WavedashCalculator {
    pub(super) player_stats: HashMap<PlayerId, WavedashStats>,
    pub(super) events: Vec<WavedashEvent>,
    pub(super) active_candidates: HashMap<PlayerId, ActiveWavedashCandidate>,
    pub(super) previous_dodge_active: HashMap<PlayerId, bool>,
    pub(super) current_last_wavedash_player: Option<PlayerId>,
}

impl WavedashCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, WavedashStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[WavedashEvent] {
        &self.events
    }

    pub(super) fn horizontal_speed(player: &PlayerSample) -> f32 {
        player
            .velocity()
            .map(|velocity| velocity.truncate().length())
            .unwrap_or(0.0)
    }

    pub(super) fn normalize_score(value: f32, min_value: f32, max_value: f32) -> f32 {
        if max_value <= min_value {
            return 0.0;
        }

        ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
    }

    pub(super) fn landing_uprightness(player: &PlayerSample) -> Option<f32> {
        let rigid_body = player.rigid_body.as_ref()?;
        Some((quat_to_glam(&rigid_body.rotation) * glam::Vec3::Z).dot(glam::Vec3::Z))
    }
}
