use super::*;

const DEMO_REPEAT_FRAME_WINDOW: usize = 8;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DemoPlayerStats {
    pub demos_inflicted: u32,
    pub demos_taken: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DemoTeamStats {
    pub demos_inflicted: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DemoCalculator {
    player_stats: HashMap<PlayerId, DemoPlayerStats>,
    player_teams: HashMap<PlayerId, bool>,
    team_zero_stats: DemoTeamStats,
    team_one_stats: DemoTeamStats,
    timeline: Vec<TimelineEvent>,
    last_seen_frame: HashMap<(PlayerId, PlayerId), usize>,
}

impl DemoCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, DemoPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &DemoTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &DemoTeamStats {
        &self.team_one_stats
    }

    pub fn timeline(&self) -> &[TimelineEvent] {
        &self.timeline
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
        for player in &players.players {
            self.player_teams
                .insert(player.player_id.clone(), player.is_team_0);
        }

        if !events.demo_events.is_empty() {
            for demo in &events.demo_events {
                self.record_demo(&demo.attacker, &demo.victim, demo.time, demo.frame);
            }
            return Ok(());
        }

        for demo in &events.active_demos {
            self.record_demo(&demo.attacker, &demo.victim, frame.time, frame.frame_number);
        }

        Ok(())
    }
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

        self.player_stats
            .entry(attacker.clone())
            .or_default()
            .demos_inflicted += 1;
        self.player_stats
            .entry(victim.clone())
            .or_default()
            .demos_taken += 1;

        match self.player_teams.get(attacker).copied() {
            Some(true) => self.team_zero_stats.demos_inflicted += 1,
            Some(false) => self.team_one_stats.demos_inflicted += 1,
            None => {}
        }

        self.timeline.push(TimelineEvent {
            time,
            kind: TimelineEventKind::Kill,
            player_id: Some(attacker.clone()),
            is_team_0: self.player_teams.get(attacker).copied(),
        });
        self.timeline.push(TimelineEvent {
            time,
            kind: TimelineEventKind::Death,
            player_id: Some(victim.clone()),
            is_team_0: self.player_teams.get(victim).copied(),
        });
    }
}
