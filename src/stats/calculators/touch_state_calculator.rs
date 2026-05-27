use super::*;

#[derive(Clone, Default)]
pub struct TouchStateCalculator {
    pub(super) previous_ball_linear_velocity: Option<glam::Vec3>,
    pub(super) previous_ball_angular_velocity: Option<glam::Vec3>,
    pub(super) current_last_touch: Option<TouchEvent>,
    pub(super) recent_touch_candidates: HashMap<PlayerId, TouchEvent>,
}

impl TouchStateCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub(super) fn prune_recent_touch_candidates(&mut self, current_frame: usize) {
        const TOUCH_CANDIDATE_WINDOW_FRAMES: usize = 4;

        self.recent_touch_candidates.retain(|_, candidate| {
            current_frame.saturating_sub(candidate.frame) <= TOUCH_CANDIDATE_WINDOW_FRAMES
        });
    }
}
