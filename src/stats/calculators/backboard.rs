use super::*;

/// Derives backboard-play stats from detected backboard bounces.
#[derive(Debug, Clone, Default)]
pub struct BackboardCalculator {
    events: EventStream<BackboardBounceEvent>,
}

impl BackboardCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[BackboardBounceEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[BackboardBounceEvent] {
        self.events.new_events()
    }

    pub fn update(
        &mut self,
        _frame: &FrameInfo,
        backboard_bounce_state: &BackboardBounceState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        self.events
            .extend(backboard_bounce_state.bounce_events.iter().cloned());
        Ok(())
    }
}
