use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(super) fn append_speed_flip_mechanic_events(
        &self,
        events: &mut Vec<MechanicEvent>,
    ) -> SubtrActorResult<()> {
        for (index, value) in self.module_array("speed_flip", "events").iter().enumerate() {
            let event = parse_speed_flip_event(value)?;
            events.push(moment_mechanic_event(
                "speed_flip",
                index,
                event.frame,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        Ok(())
    }

    pub(super) fn append_half_flip_mechanic_events(
        &self,
        events: &mut Vec<MechanicEvent>,
    ) -> SubtrActorResult<()> {
        for (index, value) in self.module_array("half_flip", "events").iter().enumerate() {
            let event = parse_half_flip_event(value)?;
            events.push(moment_mechanic_event(
                "half_flip",
                index,
                event.frame,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        Ok(())
    }

    pub(super) fn append_half_volley_mechanic_events(
        &self,
        events: &mut Vec<MechanicEvent>,
    ) -> SubtrActorResult<()> {
        for (index, value) in self
            .module_array("half_volley", "events")
            .iter()
            .enumerate()
        {
            let event = parse_half_volley_event(value)?;
            events.push(moment_mechanic_event(
                "half_volley",
                index,
                event.frame,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        Ok(())
    }

    pub(super) fn append_wavedash_mechanic_events(
        &self,
        events: &mut Vec<MechanicEvent>,
    ) -> SubtrActorResult<()> {
        for (index, value) in self.module_array("wavedash", "events").iter().enumerate() {
            let event = parse_wavedash_event(value)?;
            events.push(span_mechanic_event(
                "wavedash",
                index,
                event.dodge_frame,
                event.frame,
                event.dodge_time,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        Ok(())
    }
}
