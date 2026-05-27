use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(super) fn append_double_tap_mechanic_events(
        &self,
        events: &mut Vec<MechanicEvent>,
    ) -> SubtrActorResult<()> {
        for (index, value) in self.module_array("double_tap", "events").iter().enumerate() {
            let event = parse_double_tap_event(value)?;
            events.push(span_mechanic_event(
                "double_tap",
                index,
                event.backboard_frame,
                event.frame,
                event.backboard_time,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        Ok(())
    }

    pub(super) fn append_one_timer_mechanic_events(
        &self,
        events: &mut Vec<MechanicEvent>,
    ) -> SubtrActorResult<()> {
        for (index, value) in self.module_array("one_timer", "events").iter().enumerate() {
            let event = parse_one_timer_event(value)?;
            events.push(span_mechanic_event(
                "one_timer",
                index,
                event.pass_start_frame,
                event.frame,
                event.pass_start_time,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        Ok(())
    }

    pub(super) fn append_pass_mechanic_events(
        &self,
        events: &mut Vec<MechanicEvent>,
    ) -> SubtrActorResult<()> {
        for (index, value) in self.module_array("pass", "events").iter().enumerate() {
            let event = parse_pass_event(value)?;
            events.push(span_mechanic_event(
                "pass",
                index,
                event.start_frame,
                event.frame,
                event.start_time,
                event.time,
                event.passer,
                event.is_team_0,
            ));
        }
        Ok(())
    }
}
