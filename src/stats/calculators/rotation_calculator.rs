use super::*;

#[derive(Debug, Clone)]
pub struct RotationCalculatorConfig {
    pub role_depth_margin: f32,
    pub first_man_ambiguity_margin: f32,
    pub first_man_debounce_seconds: f32,
}

impl Default for RotationCalculatorConfig {
    fn default() -> Self {
        Self {
            role_depth_margin: DEFAULT_ROLE_DEPTH_MARGIN,
            first_man_ambiguity_margin: DEFAULT_FIRST_MAN_AMBIGUITY_MARGIN,
            first_man_debounce_seconds: DEFAULT_FIRST_MAN_DEBOUNCE_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RotationCalculator {
    pub(crate) config: RotationCalculatorConfig,
    pub(crate) player_stats: HashMap<PlayerId, RotationPlayerStats>,
    pub(crate) team_zero_stats: RotationTeamStats,
    pub(crate) team_one_stats: RotationTeamStats,
    pub(crate) team_zero_tracker: TeamFirstManTracker,
    pub(crate) team_one_tracker: TeamFirstManTracker,
    pub(crate) player_events: Vec<RotationPlayerEvent>,
    pub(crate) team_events: Vec<RotationTeamEvent>,
    pub(crate) last_emitted_player_states: HashMap<PlayerId, RotationPlayerEventState>,
    pub(crate) first_man_stints: HashMap<PlayerId, FirstManStintState>,
}

impl RotationCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: RotationCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn config(&self) -> &RotationCalculatorConfig {
        &self.config
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, RotationPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &RotationTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &RotationTeamStats {
        &self.team_one_stats
    }

    pub fn player_events(&self) -> &[RotationPlayerEvent] {
        &self.player_events
    }

    pub fn team_events(&self) -> &[RotationTeamEvent] {
        &self.team_events
    }
}
