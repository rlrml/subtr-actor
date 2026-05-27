use super::*;

#[derive(Debug, Clone)]
pub struct PositioningCalculatorConfig {
    pub most_back_forward_threshold_y: f32,
    pub level_ball_depth_margin: f32,
}

impl Default for PositioningCalculatorConfig {
    fn default() -> Self {
        Self {
            most_back_forward_threshold_y: DEFAULT_MOST_BACK_FORWARD_THRESHOLD_Y,
            level_ball_depth_margin: DEFAULT_LEVEL_BALL_DEPTH_MARGIN,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PositioningCalculator {
    pub(crate) config: PositioningCalculatorConfig,
    pub(crate) player_stats: HashMap<PlayerId, PositioningStats>,
    pub(crate) previous_ball_position: Option<glam::Vec3>,
    pub(crate) previous_player_positions: HashMap<PlayerId, glam::Vec3>,
    pub(crate) events: Vec<PositioningEvent>,
}

impl PositioningCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: PositioningCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn config(&self) -> &PositioningCalculatorConfig {
        &self.config
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, PositioningStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[PositioningEvent] {
        &self.events
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play: bool,
        possession_player_before_sample: Option<&PlayerId>,
    ) -> SubtrActorResult<()> {
        self.process_sample(
            frame,
            gameplay,
            ball,
            players,
            events,
            live_play,
            possession_player_before_sample,
        )
    }
}
