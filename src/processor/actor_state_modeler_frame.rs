use super::*;

impl ActorStateModeler {
    /// Processes a frame, including handling of new, updated, and deleted actors.
    pub fn process_frame(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.recently_deleted_actor_states.clear();
        if let Some(err) = frame
            .deleted_actors
            .iter()
            .map(|n| self.delete_actor(n))
            .find(|r| r.is_err())
        {
            return err.map(|_| ());
        }
        if let Some(err) = frame
            .new_actors
            .iter()
            .map(|n| self.new_actor(n))
            .find(|r| r.is_err())
        {
            return err;
        }
        if let Some(err) = frame
            .updated_actors
            .iter()
            .map(|u| self.update_attribute(u, frame_index))
            .find(|r| r.is_err())
        {
            return err.map(|_| ());
        }
        Ok(())
    }
}
