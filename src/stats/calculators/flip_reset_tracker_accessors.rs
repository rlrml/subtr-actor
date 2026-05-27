use super::*;

impl FlipResetTracker {
    pub fn flip_reset_events(&self) -> &[FlipResetEvent] {
        &self.flip_reset_events
    }

    pub fn current_frame_flip_reset_events(&self) -> &[FlipResetEvent] {
        &self.current_frame_flip_reset_events
    }

    pub fn post_wall_dodge_events(&self) -> &[PostWallDodgeEvent] {
        &self.post_wall_dodge_events
    }

    pub fn current_frame_post_wall_dodge_events(&self) -> &[PostWallDodgeEvent] {
        &self.current_frame_post_wall_dodge_events
    }

    pub fn flip_reset_followup_dodge_events(&self) -> &[FlipResetFollowupDodgeEvent] {
        &self.flip_reset_followup_dodge_events
    }

    pub fn current_frame_flip_reset_followup_dodge_events(&self) -> &[FlipResetFollowupDodgeEvent] {
        &self.current_frame_flip_reset_followup_dodge_events
    }

    pub fn into_events(
        self,
    ) -> (
        Vec<FlipResetEvent>,
        Vec<PostWallDodgeEvent>,
        Vec<FlipResetFollowupDodgeEvent>,
    ) {
        (
            self.flip_reset_events,
            self.post_wall_dodge_events,
            self.flip_reset_followup_dodge_events,
        )
    }
}
