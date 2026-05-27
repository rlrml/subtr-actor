use super::*;

impl BumpCalculator {
    pub(super) fn detect_bumps(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        frame_events: &FrameEventsState,
        fifty_fifty_state: &FiftyFiftyState,
    ) {
        let current_players: Vec<_> = players
            .players
            .iter()
            .filter_map(|player| {
                Some((
                    player,
                    player.rigid_body.as_ref()?,
                    self.previous_players.get(&player.player_id)?.rigid_body,
                ))
            })
            .collect();

        for left_index in 0..current_players.len() {
            for right_index in (left_index + 1)..current_players.len() {
                let (left, left_body, previous_left_body) = current_players[left_index];
                let (right, right_body, previous_right_body) = current_players[right_index];

                if self.is_recent_demo_pair(frame_events, &left.player_id, &right.player_id) {
                    continue;
                }
                if Self::is_recent_fifty_fifty_pair(
                    frame,
                    fifty_fifty_state,
                    &left.player_id,
                    &right.player_id,
                ) {
                    continue;
                }

                let Some(event) = Self::evaluate_pair(
                    frame,
                    left,
                    left_body,
                    &previous_left_body,
                    right,
                    right_body,
                    &previous_right_body,
                ) else {
                    continue;
                };

                if self.should_count_bump(&event.initiator, &event.victim, frame.frame_number) {
                    self.record_bump(event);
                }
            }
        }
    }
}
