use super::*;

impl HalfFlipCalculator {
    pub(super) fn apply_event(&mut self, event: HalfFlipEvent) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_half_flip = false;
        }

        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.record_event(&event);
        stats.is_last_half_flip = true;
        stats.time_since_last_half_flip = Some(0.0);
        stats.frames_since_last_half_flip = Some(0);

        self.current_last_half_flip_player = Some(event.player.clone());
        self.events.push(event);
    }

    pub(super) fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_half_flip = false;
            stats.time_since_last_half_flip = stats
                .last_half_flip_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_half_flip = stats
                .last_half_flip_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }

        if let Some(player_id) = self.current_last_half_flip_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_half_flip = true;
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
            self.current_last_half_flip_player = None;
            return Ok(());
        }

        self.begin_sample(frame);

        for player in &players.players {
            self.maybe_start_candidate(frame, player);
        }

        let mut visible_players = HashSet::new();
        for player in &players.players {
            visible_players.insert(player.player_id.clone());
            if let Some(candidate) = self.active_candidates.get_mut(&player.player_id) {
                Self::update_candidate(candidate, frame, player);
            }
        }

        self.finalize_candidates(frame, false);
        self.active_candidates.retain(|player_id, candidate| {
            visible_players.contains(player_id)
                && frame.time - candidate.start_time <= HALF_FLIP_MAX_CANDIDATE_SECONDS
        });

        Ok(())
    }

    pub fn finalize(&mut self, frame: &FrameInfo) {
        self.finalize_candidates(frame, true);
    }
}
