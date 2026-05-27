use super::*;

impl SpeedFlipCalculator {
    pub fn update_parts(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        let kickoff_approach_active = Self::kickoff_approach_active(gameplay);
        if !live_play && !kickoff_approach_active {
            self.active_candidates.clear();
            self.current_kickoff_start_time = None;
            self.kickoff_approach_active_last_frame = false;
            return Ok(());
        }

        self.begin_sample(frame);

        if kickoff_approach_active && !self.kickoff_approach_active_last_frame {
            self.reset_kickoff_state();
        }

        self.update_kickoff_start_time(frame, kickoff_approach_active, players);

        for player in &players.players {
            self.maybe_start_candidate(frame, gameplay, ball, player, live_play);
        }

        for (player_id, candidate) in &mut self.active_candidates {
            let Some(player) = Self::player_by_id(players, player_id) else {
                continue;
            };
            Self::update_candidate(candidate, frame, ball, player);
        }

        self.finalize_candidates(frame, false);
        self.active_candidates.retain(|_, candidate| {
            frame.time - candidate.start_time <= SPEED_FLIP_MAX_CANDIDATE_SECONDS
        });

        if !kickoff_approach_active {
            self.current_kickoff_start_time = None;
        }

        self.kickoff_approach_active_last_frame = kickoff_approach_active;
        Ok(())
    }

    pub fn finalize_parts(&mut self, frame: &FrameInfo) {
        self.finalize_candidates(frame, true);
    }
}
