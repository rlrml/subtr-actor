use super::*;

#[derive(Debug, Clone, Default)]
pub struct BackboardCalculator {
    stats: BackboardStatsAccumulator,
    events: EventStream<BackboardBounceEvent>,
}

impl BackboardCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, BackboardPlayerStats> {
        self.stats.player_stats()
    }

    pub fn team_zero_stats(&self) -> &BackboardTeamStats {
        self.stats.team_zero_stats()
    }

    pub fn team_one_stats(&self) -> &BackboardTeamStats {
        self.stats.team_one_stats()
    }

    pub fn events(&self) -> &[BackboardBounceEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[BackboardBounceEvent] {
        self.events.new_events()
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        backboard_bounce_state: &BackboardBounceState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        self.stats.begin_sample(frame);
        self.stats
            .apply_events(frame, &backboard_bounce_state.bounce_events);
        self.events
            .extend(backboard_bounce_state.bounce_events.iter().cloned());
        Ok(())
    }
}
