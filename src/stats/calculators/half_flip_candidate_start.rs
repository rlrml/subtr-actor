use super::*;

impl HalfFlipCalculator {
    pub(super) fn maybe_start_candidate(&mut self, frame: &FrameInfo, player: &PlayerSample) {
        let was_dodge_active = self
            .previous_dodge_active
            .insert(player.player_id.clone(), player.dodge_active)
            .unwrap_or(false);
        if !player.dodge_active || was_dodge_active {
            return;
        }

        let Some(position) = player.position() else {
            return;
        };
        if position.z > HALF_FLIP_MAX_START_Z {
            return;
        }

        let velocity_xy = Self::horizontal_velocity(player).unwrap_or(glam::Vec2::ZERO);
        let start_speed = velocity_xy.length();
        if start_speed < HALF_FLIP_MIN_START_SPEED {
            return;
        }

        let Some(start_forward_xy) = Self::forward_xy(player) else {
            return;
        };
        let velocity_direction = velocity_xy.normalize_or_zero();
        let start_backward_alignment = -start_forward_xy.dot(velocity_direction);
        if start_backward_alignment < HALF_FLIP_MIN_START_BACKWARD_ALIGNMENT {
            return;
        }

        self.active_candidates.insert(
            player.player_id.clone(),
            ActiveHalfFlipCandidate {
                is_team_0: player.is_team_0,
                start_time: frame.time,
                start_frame: frame.frame_number,
                latest_time: frame.time,
                latest_frame: frame.frame_number,
                start_position: position.to_array(),
                end_position: position.to_array(),
                start_speed,
                end_speed: start_speed,
                start_forward_xy,
                start_backward_alignment,
                best_reorientation_alignment: 0.0,
                best_forward_reversal: 0.0,
                max_forward_vertical: Self::forward_vector(player)
                    .map_or(0.0, |forward| forward.z.abs()),
            },
        );
    }
}
