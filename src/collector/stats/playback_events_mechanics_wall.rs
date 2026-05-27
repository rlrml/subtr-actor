use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(super) fn append_wall_mechanic_events(
        &self,
        events: &mut Vec<MechanicEvent>,
    ) -> SubtrActorResult<()> {
        for (index, value) in self
            .module_array("wall_aerial", "events")
            .iter()
            .enumerate()
        {
            let event = parse_wall_aerial_event(value)?;
            let mut mechanic_event = span_mechanic_event(
                "wall_aerial",
                index,
                event.wall_contact_frame,
                event.frame,
                event.wall_contact_time,
                event.time,
                event.player,
                event.is_team_0,
            );
            mechanic_event.properties = vec![mechanic_event_text_property(
                "wall",
                event.wall.as_label_value(),
            )];
            events.push(mechanic_event);
        }
        self.append_wall_aerial_shot_mechanic_events(events)
    }

    fn append_wall_aerial_shot_mechanic_events(
        &self,
        events: &mut Vec<MechanicEvent>,
    ) -> SubtrActorResult<()> {
        for (index, value) in self
            .module_array("wall_aerial_shot", "events")
            .iter()
            .enumerate()
        {
            let event = parse_wall_aerial_shot_event(value)?;
            let mut mechanic_event = span_mechanic_event(
                "wall_aerial_shot",
                index,
                event.wall_contact_frame,
                event.frame,
                event.wall_contact_time,
                event.time,
                event.player,
                event.is_team_0,
            );
            mechanic_event.properties = vec![mechanic_event_text_property(
                "wall",
                event.wall.as_label_value(),
            )];
            events.push(mechanic_event);
        }
        Ok(())
    }
}
