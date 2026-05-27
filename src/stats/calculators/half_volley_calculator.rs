use super::*;

#[derive(Debug, Clone, Default)]
pub struct HalfVolleyCalculator {
    pub(super) config: HalfVolleyCalculatorConfig,
    pub(super) player_stats: HashMap<PlayerId, HalfVolleyPlayerStats>,
    pub(super) team_zero_stats: HalfVolleyTeamStats,
    pub(super) team_one_stats: HalfVolleyTeamStats,
    pub(super) events: Vec<HalfVolleyEvent>,
    pub(super) last_floor_bounce: Option<FloorBounce>,
    pub(super) last_ground_contacts: HashMap<PlayerId, GroundContact>,
    pub(super) recent_dodge_starts: HashMap<PlayerId, DodgeStart>,
    pub(super) previous_dodge_active: HashMap<PlayerId, bool>,
    pub(super) previous_ball_velocity: Option<glam::Vec3>,
    pub(super) current_last_half_volley_player: Option<PlayerId>,
}

impl HalfVolleyCalculator {
    pub fn new() -> Self {
        Self::with_config(HalfVolleyCalculatorConfig::default())
    }

    pub fn with_config(config: HalfVolleyCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn config(&self) -> &HalfVolleyCalculatorConfig {
        &self.config
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, HalfVolleyPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &HalfVolleyTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &HalfVolleyTeamStats {
        &self.team_one_stats
    }

    pub fn events(&self) -> &[HalfVolleyEvent] {
        &self.events
    }
}
