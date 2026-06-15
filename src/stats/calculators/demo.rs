use super::*;

const DEMO_REPEAT_FRAME_WINDOW: usize = 8;

/// A single demolition, linking the demoer (`attacker`) to the demoee
/// (`victim`). Each demolition emits exactly one of these events; the attacker
/// side contributes a demo inflicted and the victim side a demo taken.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DemolitionEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub attacker: PlayerId,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub victim: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attacker_is_team_0: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub victim_is_team_0: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attacker_position: Option<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub victim_position: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DemoCalculator {
    player_teams: HashMap<PlayerId, bool>,
    timeline: EventStream<DemolitionEvent>,
    last_seen_frame: HashMap<(PlayerId, PlayerId), usize>,
    active_pairs: HashSet<(PlayerId, PlayerId)>,
}

impl DemoCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[DemolitionEvent] {
        self.timeline.all()
    }

    pub fn new_events(&self) -> &[DemolitionEvent] {
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

        self.timeline.push(DemolitionEvent {
            time,
            frame: frame_number,
            attacker: attacker.clone(),
            victim: victim.clone(),
            attacker_is_team_0: self.player_teams.get(attacker).copied(),
            victim_is_team_0: self.player_teams.get(victim).copied(),
            attacker_position,
            victim_position,
        });
    }
}

#[cfg(test)]
#[path = "demo_tests.rs"]
mod tests;
