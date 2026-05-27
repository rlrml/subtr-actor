use super::*;

impl DemoCalculator {
    pub(super) fn record_demo(
        &mut self,
        attacker: &PlayerId,
        victim: &PlayerId,
        time: f32,
        frame_number: usize,
    ) {
        if !self.should_count_demo(attacker, victim, frame_number) {
            return;
        }

        self.player_stats
            .entry(attacker.clone())
            .or_default()
            .demos_inflicted += 1;
        self.player_stats
            .entry(victim.clone())
            .or_default()
            .demos_taken += 1;
        self.record_team_demo(attacker);
        self.push_timeline_events(attacker, victim, time, frame_number);
    }

    fn record_team_demo(&mut self, attacker: &PlayerId) {
        match self.player_teams.get(attacker).copied() {
            Some(true) => self.team_zero_stats.demos_inflicted += 1,
            Some(false) => self.team_one_stats.demos_inflicted += 1,
            None => {}
        }
    }

    fn push_timeline_events(
        &mut self,
        attacker: &PlayerId,
        victim: &PlayerId,
        time: f32,
        frame_number: usize,
    ) {
        self.timeline.push(TimelineEvent {
            time,
            frame: Some(frame_number),
            kind: TimelineEventKind::Kill,
            player_id: Some(attacker.clone()),
            is_team_0: self.player_teams.get(attacker).copied(),
        });
        self.timeline.push(TimelineEvent {
            time,
            frame: Some(frame_number),
            kind: TimelineEventKind::Death,
            player_id: Some(victim.clone()),
            is_team_0: self.player_teams.get(victim).copied(),
        });
    }
}
