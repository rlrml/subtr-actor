use super::*;

#[derive(Debug, Clone, Default)]
pub struct FlipResetTracker {
    pub(crate) flip_reset_events: Vec<FlipResetEvent>,
    pub(crate) current_frame_flip_reset_events: Vec<FlipResetEvent>,
    pub(crate) post_wall_dodge_events: Vec<PostWallDodgeEvent>,
    pub(crate) current_frame_post_wall_dodge_events: Vec<PostWallDodgeEvent>,
    pub(crate) flip_reset_followup_dodge_events: Vec<FlipResetFollowupDodgeEvent>,
    pub(crate) current_frame_flip_reset_followup_dodge_events: Vec<FlipResetFollowupDodgeEvent>,
    pub(crate) recent_wall_contact_time: HashMap<PlayerId, f32>,
    pub(crate) recent_flip_reset_candidates: HashMap<PlayerId, FlipResetEvent>,
    pub(crate) recent_flip_reset_proximity_event_time: HashMap<PlayerId, f32>,
    pub(crate) current_frame_dodge_rising_edges: Vec<PlayerId>,
    pub(crate) previous_dodge_active: HashMap<PlayerId, bool>,
}

impl FlipResetTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_frame(
        &mut self,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.update_flip_reset_events(processor, frame.time, frame_index)?;
        self.update_dodge_rising_edges(processor)?;
        self.update_flip_reset_followup_dodge_events(processor, frame, frame_index)?;
        self.update_post_wall_dodge_events(processor, frame, frame_index)?;
        Ok(())
    }
}
