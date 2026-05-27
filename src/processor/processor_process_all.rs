use super::*;

impl<'a> ReplayProcessor<'a> {
    pub fn process_all(&mut self, collectors: &mut [&mut dyn Collector]) -> SubtrActorResult<()> {
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
            for collector in collectors.iter_mut() {
                collector.process_frame(self, frame, index, frame.time)?;
            }
        }
        for collector in collectors.iter_mut() {
            collector.finish_replay(self)?;
        }
        Ok(())
    }
}
