use super::*;

impl SpeedFlipCalculator {
    pub(super) fn kickoff_approach_active(gameplay: &GameplayState) -> bool {
        gameplay.ball_has_been_hit == Some(false)
    }

    pub(super) fn reset_kickoff_state(&mut self) {
        self.active_candidates.clear();
        self.current_kickoff_start_time = None;
    }

    pub(super) fn kickoff_motion_started(players: &PlayerFrameState) -> bool {
        players.players.iter().any(|player| {
            player.dodge_active
                || player
                    .speed()
                    .is_some_and(|speed| speed >= SPEED_FLIP_KICKOFF_MOTION_SPEED)
        })
    }

    pub(super) fn update_kickoff_start_time(
        &mut self,
        frame: &FrameInfo,
        kickoff_approach_active: bool,
        players: &PlayerFrameState,
    ) {
        if !kickoff_approach_active {
            self.current_kickoff_start_time = None;
            return;
        }

        if self.current_kickoff_start_time.is_none() && Self::kickoff_motion_started(players) {
            self.current_kickoff_start_time = Some(frame.time);
        }
    }
}
