use subtr_actor::{Collector, ProcessorView, SubtrActorResult, TimeAdvance};

pub(crate) struct GoalScanCollector;

impl Collector for GoalScanCollector {
    fn process_frame(
        &mut self,
        _processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        _frame_number: usize,
        _current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        Ok(TimeAdvance::NextFrame)
    }
}
