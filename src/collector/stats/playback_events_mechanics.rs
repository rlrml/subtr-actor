use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(super) fn mechanic_events_typed(&self) -> SubtrActorResult<Vec<MechanicEvent>> {
        let mut events = Vec::new();
        self.append_ball_carry_mechanic_events(&mut events)?;
        self.append_span_mechanic_events(&mut events)?;
        self.append_wall_mechanic_events(&mut events)?;
        self.append_moment_mechanic_events(&mut events)?;
        self.append_wavedash_mechanic_events(&mut events)?;
        sort_mechanic_events(&mut events);
        Ok(events)
    }
}

fn sort_mechanic_events(events: &mut [MechanicEvent]) {
    events.sort_by(|left, right| {
        let left_time = mechanic_event_start_time(left);
        let right_time = mechanic_event_start_time(right);
        left_time
            .total_cmp(&right_time)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.id.cmp(&right.id))
    });
}
