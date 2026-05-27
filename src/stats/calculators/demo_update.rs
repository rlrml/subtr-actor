use super::*;

impl DemoCalculator {
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &FrameEventsState,
    ) -> SubtrActorResult<()> {
        for player in &players.players {
            self.player_teams
                .insert(player.player_id.clone(), player.is_team_0);
        }

        if !events.demo_events.is_empty() {
            for demo in &events.demo_events {
                self.record_demo(&demo.attacker, &demo.victim, demo.time, demo.frame);
            }
            self.active_pairs = active_demo_pairs(events);
            return Ok(());
        }

        let current_active_pairs = active_demo_pairs(events);
        for demo in &events.active_demos {
            if self
                .active_pairs
                .contains(&(demo.attacker.clone(), demo.victim.clone()))
            {
                continue;
            }
            self.record_demo(&demo.attacker, &demo.victim, frame.time, frame.frame_number);
        }
        self.active_pairs = current_active_pairs;

        Ok(())
    }
}

fn active_demo_pairs(events: &FrameEventsState) -> HashSet<(PlayerId, PlayerId)> {
    events
        .active_demos
        .iter()
        .map(|demo| (demo.attacker.clone(), demo.victim.clone()))
        .collect()
}
