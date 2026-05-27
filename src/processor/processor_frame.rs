use super::*;

impl<'a> ReplayProcessor<'a> {
    pub(crate) fn process_processor_frame(
        &mut self,
        frame: &boxcars::Frame,
        index: usize,
    ) -> SubtrActorResult<()> {
        self.actor_state.process_frame(frame, index)?;
        self.update_mappings(frame)?;
        self.update_ball_id(frame)?;
        self.update_boost_amounts(frame, index)?;
        self.update_boost_pad_events(frame, index)?;
        self.update_touch_events(frame, index)?;
        self.update_dodge_refreshed_events(frame, index)?;
        self.update_goal_events(frame, index)?;
        self.update_player_stat_events(frame, index)?;
        self.update_demolishes(frame, index)?;
        Ok(())
    }
}
