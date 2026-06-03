use super::*;

const DEMO_REPEAT_FRAME_WINDOW: usize = 8;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DemoCalculator {
    stats: DemoStatsAccumulator,
    player_teams: HashMap<PlayerId, bool>,
    timeline: EventStream<TimelineEvent>,
    last_seen_frame: HashMap<(PlayerId, PlayerId), usize>,
    active_pairs: HashSet<(PlayerId, PlayerId)>,
}

impl DemoCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, DemoPlayerStats> {
        self.stats.player_stats()
    }

    pub fn team_zero_stats(&self) -> &DemoTeamStats {
        self.stats.team_zero_stats()
    }

    pub fn team_one_stats(&self) -> &DemoTeamStats {
        self.stats.team_one_stats()
    }

    pub fn timeline(&self) -> &[TimelineEvent] {
        self.timeline.all()
    }

    pub fn new_timeline_events(&self) -> &[TimelineEvent] {
        self.timeline.new_events()
    }

    fn should_count_demo(
        &mut self,
        attacker: &PlayerId,
        victim: &PlayerId,
        frame_number: usize,
    ) -> bool {
        let key = (attacker.clone(), victim.clone());
        let already_counted = self
            .last_seen_frame
            .get(&key)
            .map(|previous_frame| {
                frame_number.saturating_sub(*previous_frame) <= DEMO_REPEAT_FRAME_WINDOW
            })
            .unwrap_or(false);
        self.last_seen_frame.insert(key, frame_number);
        !already_counted
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &FrameEventsState,
    ) -> SubtrActorResult<()> {
        self.timeline.begin_update();
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

impl DemoCalculator {
    fn record_demo(
        &mut self,
        attacker: &PlayerId,
        victim: &PlayerId,
        time: f32,
        frame_number: usize,
    ) {
        if !self.should_count_demo(attacker, victim, frame_number) {
            return;
        }

        let kill_event = TimelineEvent {
            time,
            frame: Some(frame_number),
            kind: TimelineEventKind::Kill,
            player_id: Some(attacker.clone()),
            is_team_0: self.player_teams.get(attacker).copied(),
        };
        self.stats.apply_timeline_event(&kill_event);
        self.timeline.push(kill_event);

        let death_event = TimelineEvent {
            time,
            frame: Some(frame_number),
            kind: TimelineEventKind::Death,
            player_id: Some(victim.clone()),
            is_team_0: self.player_teams.get(victim).copied(),
        };
        self.stats.apply_timeline_event(&death_event);
        self.timeline.push(death_event);
    }
}

#[cfg(test)]
#[path = "demo_tests.rs"]
mod tests;
