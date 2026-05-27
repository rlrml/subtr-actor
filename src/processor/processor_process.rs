use super::*;

impl<'a> ReplayProcessor<'a> {
    pub fn process<H: Collector>(&mut self, handler: &mut H) -> SubtrActorResult<()> {
        let mut target_time = TimeAdvance::NextFrame;
        let frames = &self
            .replay
            .network_frames
            .as_ref()
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::NoNetworkFrames,
            ))?
            .frames;

        for (index, frame) in frames.iter().enumerate() {
            self.process_processor_frame(frame, index)?;
            let mut current_time = match &target_time {
                TimeAdvance::Time(t) => *t,
                TimeAdvance::NextFrame => frame.time,
            };

            while current_time <= frame.time {
                target_time = handler.process_frame(self, frame, index, current_time)?;
                if let TimeAdvance::Time(new_target) = target_time {
                    current_time = new_target;
                } else {
                    break;
                }
            }
        }
        handler.finish_replay(self)?;
        Ok(())
    }
}
