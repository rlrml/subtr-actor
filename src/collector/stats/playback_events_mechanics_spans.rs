use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(super) fn append_ball_carry_mechanic_events(
        &self,
        events: &mut Vec<MechanicEvent>,
    ) -> SubtrActorResult<()> {
        for (index, value) in self.module_array("ball_carry", "events").iter().enumerate() {
            events.push(parse_ball_carry_mechanic_event(value, index)?);
        }
        Ok(())
    }

    pub(super) fn append_span_mechanic_events(
        &self,
        events: &mut Vec<MechanicEvent>,
    ) -> SubtrActorResult<()> {
        self.append_ceiling_shot_mechanic_events(events)?;
        self.append_center_mechanic_events(events)?;
        self.append_double_tap_mechanic_events(events)?;
        self.append_one_timer_mechanic_events(events)?;
        self.append_pass_mechanic_events(events)
    }

    fn append_ceiling_shot_mechanic_events(
        &self,
        events: &mut Vec<MechanicEvent>,
    ) -> SubtrActorResult<()> {
        for (index, value) in self
            .module_array("ceiling_shot", "events")
            .iter()
            .enumerate()
        {
            let event = parse_ceiling_shot_event(value)?;
            events.push(span_mechanic_event(
                "ceiling_shot",
                index,
                event.ceiling_contact_frame,
                event.frame,
                event.ceiling_contact_time,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        Ok(())
    }

    fn append_center_mechanic_events(
        &self,
        events: &mut Vec<MechanicEvent>,
    ) -> SubtrActorResult<()> {
        for (index, value) in self.module_array("center", "events").iter().enumerate() {
            let event = parse_center_event(value)?;
            events.push(span_mechanic_event(
                "center",
                index,
                event.start_frame,
                event.frame,
                event.start_time,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        Ok(())
    }
}
