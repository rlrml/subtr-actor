use super::*;

impl<'a> ReplayProcessor<'a> {
    /// Refreshes the cached ball actor id, clearing it if the current ball was deleted.
    pub(crate) fn update_ball_id(&mut self, frame: &boxcars::Frame) -> SubtrActorResult<()> {
        // XXX: This assumes there is only ever one ball, which is safe (I think?)
        if let Some(actor_id) = self.ball_actor_id {
            if frame.deleted_actors.contains(&actor_id) {
                self.ball_actor_id = None;
            }
        } else {
            self.ball_actor_id = self.find_ball_actor();
            if self.ball_actor_id.is_some() {
                return self.update_ball_id(frame);
            }
        }
        Ok(())
    }
}
