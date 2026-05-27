use super::*;

impl TouchStateCalculator {
    pub(super) fn confirmed_touch_events(
        &self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
    ) -> Vec<TouchEvent> {
        let mut touch_events = Vec::new();
        let mut confirmed_players = HashSet::new();

        self.append_explicit_touch_events(events, &mut touch_events, &mut confirmed_players);
        self.append_velocity_touch_candidate(
            frame,
            ball,
            players,
            &mut touch_events,
            &mut confirmed_players,
        );
        self.append_dodge_refresh_touch_candidates(
            events,
            &mut touch_events,
            &mut confirmed_players,
        );
        touch_events
    }

    fn append_explicit_touch_events(
        &self,
        events: &FrameEventsState,
        touch_events: &mut Vec<TouchEvent>,
        confirmed_players: &mut HashSet<PlayerId>,
    ) {
        for event in &events.touch_events {
            let event = self.enrich_explicit_touch_event(event);
            insert_confirmed_player(&event, confirmed_players);
            touch_events.push(event);
        }
    }

    fn append_velocity_touch_candidate(
        &self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_events: &mut Vec<TouchEvent>,
        confirmed_players: &mut HashSet<PlayerId>,
    ) {
        if !touch_events.is_empty() || !self.is_touch_candidate(frame, ball) {
            return;
        }
        let Some(candidate) = self.candidate_touch_event(frame, ball, players) else {
            return;
        };

        for contested_candidate in self.contested_touch_candidates(&candidate) {
            insert_confirmed_player(&contested_candidate, confirmed_players);
            touch_events.push(contested_candidate);
        }
        insert_confirmed_player(&candidate, confirmed_players);
        touch_events.push(candidate);
    }

    fn append_dodge_refresh_touch_candidates(
        &self,
        events: &FrameEventsState,
        touch_events: &mut Vec<TouchEvent>,
        confirmed_players: &mut HashSet<PlayerId>,
    ) {
        for dodge_refresh in &events.dodge_refreshed_events {
            if !confirmed_players.insert(dodge_refresh.player.clone()) {
                continue;
            }
            let Some(candidate) = self.candidate_for_player(&dodge_refresh.player) else {
                continue;
            };
            touch_events.push(candidate);
        }
    }
}

fn insert_confirmed_player(event: &TouchEvent, confirmed_players: &mut HashSet<PlayerId>) {
    if let Some(player_id) = event.player.clone() {
        confirmed_players.insert(player_id);
    }
}
