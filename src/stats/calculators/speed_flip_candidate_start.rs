use super::*;

impl SpeedFlipCalculator {
    pub(super) fn maybe_start_candidate(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        player: &PlayerSample,
        _live_play: bool,
    ) {
        let was_dodge_active = self
            .previous_dodge_active
            .insert(player.player_id.clone(), player.dodge_active)
            .unwrap_or(false);
        if !player.dodge_active || was_dodge_active {
            return;
        }

        let is_kickoff = Self::kickoff_approach_active(gameplay);
        let kickoff_start_time = if is_kickoff {
            let Some(kickoff_start_time) = self.current_kickoff_start_time else {
                return;
            };
            if frame.time - kickoff_start_time > SPEED_FLIP_MAX_START_AFTER_KICKOFF_SECONDS {
                return;
            }
            Some(kickoff_start_time)
        } else {
            None
        };

        let Some(rigid_body) = player.rigid_body.as_ref() else {
            return;
        };
        let Some(player_position) = player.position() else {
            return;
        };
        if player_position.z > SPEED_FLIP_MAX_GROUND_Z {
            return;
        }

        let start_speed = player.speed().unwrap_or(0.0);
        let Some(best_alignment) = Self::candidate_alignment(ball, player, is_kickoff) else {
            return;
        };
        if best_alignment < SPEED_FLIP_MIN_ALIGNMENT {
            return;
        }
        let Some(start_velocity_xy) = player.velocity().map(|velocity| velocity.truncate()) else {
            return;
        };
        let Some(start_forward_xy) = Self::forward_xy(player) else {
            return;
        };

        self.insert_candidate(
            frame,
            player,
            rigid_body,
            is_kickoff,
            kickoff_start_time,
            player_position,
            start_speed,
            best_alignment,
            start_velocity_xy,
            start_forward_xy,
        );
    }
}
