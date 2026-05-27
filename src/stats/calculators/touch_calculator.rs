use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct PendingFiftyFiftyMovement {
    pub(crate) start_frame: usize,
    pub(crate) travel_distance: f32,
    pub(crate) y_delta: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TouchCalculator {
    pub(crate) player_stats: HashMap<PlayerId, TouchStats>,
    pub(crate) events: Vec<TouchStatsEvent>,
    pub(crate) ball_movement_events: Vec<TouchBallMovementEvent>,
    pub(crate) last_touch_events: Vec<TouchLastTouchEvent>,
    pub(crate) current_last_touch_player: Option<PlayerId>,
    pub(crate) previous_ball_velocity: Option<glam::Vec3>,
    pub(crate) previous_ball_position: Option<glam::Vec3>,
    pub(crate) pending_fifty_fifty_movement: Option<PendingFiftyFiftyMovement>,
}

impl TouchCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, TouchStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[TouchStatsEvent] {
        &self.events
    }

    pub fn ball_movement_events(&self) -> &[TouchBallMovementEvent] {
        &self.ball_movement_events
    }

    pub fn last_touch_events(&self) -> &[TouchLastTouchEvent] {
        &self.last_touch_events
    }
}
