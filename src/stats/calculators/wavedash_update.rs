use super::*;

impl WavedashCalculator {
    pub(super) fn update_active_candidates(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
    ) {
        let mut finished = Vec::new();
        let mut visible_players = HashSet::new();

        for player in &players.players {
            visible_players.insert(player.player_id.clone());
            self.maybe_start_candidate(frame, player);
            self.update_candidate(frame, player, &mut finished);
        }

        self.finish_candidates(finished);
        self.active_candidates
            .retain(|player_id, _| visible_players.contains(player_id));
    }

    fn update_candidate(
        &self,
        frame: &FrameInfo,
        player: &PlayerSample,
        finished: &mut Vec<(PlayerId, Option<WavedashEvent>)>,
    ) {
        let Some(candidate) = self.active_candidates.get(&player.player_id).cloned() else {
            return;
        };
        if frame.time - candidate.dodge_time > WAVEDASH_MAX_CANDIDATE_SECONDS {
            finished.push((player.player_id.clone(), None));
            return;
        }
        if let Some(event) = Self::candidate_event(&player.player_id, candidate, frame, player) {
            finished.push((player.player_id.clone(), Some(event)));
        }
    }

    fn finish_candidates(&mut self, finished: Vec<(PlayerId, Option<WavedashEvent>)>) {
        for (player_id, event) in finished {
            self.active_candidates.remove(&player_id);
            if let Some(event) = event {
                self.apply_event(event);
            }
        }
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        if !live_play {
            self.active_candidates.clear();
            self.current_last_wavedash_player = None;
            return Ok(());
        }

        self.begin_sample(frame);
        self.update_active_candidates(frame, players);

        Ok(())
    }
}
