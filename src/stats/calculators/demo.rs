use super::*;

const DEMO_REPEAT_FRAME_WINDOW: usize = 8;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DemoPlayerStats {
    pub demos_inflicted: u32,
    pub demos_taken: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
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
    active_pairs: HashSet<(PlayerId, PlayerId)>,
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
                self.record_demo(
                    &demo.attacker,
                    demo.attacker_location
                        .map(|position| vec_to_glam(&position).to_array()),
                    &demo.victim,
                    Some(vec_to_glam(&demo.victim_location).to_array()),
                    demo.time,
                    demo.frame,
                );
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
            self.record_demo(
                &demo.attacker,
                None,
                &demo.victim,
                None,
                frame.time,
                frame.frame_number,
            );
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
        attacker_position: Option<[f32; 3]>,
        victim: &PlayerId,
        victim_position: Option<[f32; 3]>,
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
            frame: Some(frame_number),
            kind: TimelineEventKind::Kill,
            player_id: Some(attacker.clone()),
            player_position: attacker_position,
            is_team_0: self.player_teams.get(attacker).copied(),
        });
        self.timeline.push(TimelineEvent {
            time,
            frame: Some(frame_number),
            kind: TimelineEventKind::Death,
            player_id: Some(victim.clone()),
            player_position: victim_position,
            is_team_0: self.player_teams.get(victim).copied(),
        });
    }
}

#[cfg(test)]
#[path = "demo_tests.rs"]
mod tests;
