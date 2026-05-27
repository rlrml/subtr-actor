use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(super) fn append_moment_mechanic_events(
        &self,
        events: &mut Vec<MechanicEvent>,
    ) -> SubtrActorResult<()> {
        for (index, value) in self
            .module_array("dodge_reset", "on_ball_events")
            .iter()
            .enumerate()
        {
            events.push(parse_dodge_reset_mechanic_event(value, index)?);
        }
        self.append_speed_flip_mechanic_events(events)?;
        self.append_half_flip_mechanic_events(events)?;
        self.append_half_volley_mechanic_events(events)?;
        self.append_replay_mechanic_events(events)
    }

    fn append_replay_mechanic_events(
        &self,
        events: &mut Vec<MechanicEvent>,
    ) -> SubtrActorResult<()> {
        for (index, value) in self.module_array("flick", "events").iter().enumerate() {
            events.push(parse_flick_mechanic_event(value, index)?);
        }
        for (index, value) in self
            .module_array("musty_flick", "events")
            .iter()
            .enumerate()
        {
            events.push(parse_musty_flick_mechanic_event(value, index)?);
        }
        Ok(())
    }
}
