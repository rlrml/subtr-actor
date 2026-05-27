use super::*;

impl TouchCalculator {
    pub(crate) fn credit_ball_movement(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        possession_state: &PossessionState,
        fifty_fifty_state: &FiftyFiftyState,
        live_play: bool,
    ) {
        let current_ball_position = ball.position();
        if self.reset_ball_movement_tracking_if_needed(current_ball_position, live_play) {
            return;
        }

        let current_ball_position = current_ball_position.expect("checked above");
        let Some(previous_ball_position) = self.previous_ball_position else {
            self.previous_ball_position = Some(current_ball_position);
            return;
        };
        self.previous_ball_position = Some(current_ball_position);

        let delta = current_ball_position - previous_ball_position;
        let travel_distance = delta.length();
        if travel_distance <= f32::EPSILON {
            return;
        }

        if self.handle_fifty_fifty_movement(delta, travel_distance, fifty_fifty_state) {
            return;
        }

        let (Some(player_id), Some(team_is_team_0)) = (
            possession_state.active_player_before_sample.as_ref(),
            possession_state.active_team_before_sample,
        ) else {
            return;
        };
        self.apply_ball_movement_credit(
            frame.frame_number,
            frame.time,
            player_id,
            team_is_team_0,
            delta,
            travel_distance,
        );
    }

    fn reset_ball_movement_tracking_if_needed(
        &mut self,
        current_ball_position: Option<glam::Vec3>,
        live_play: bool,
    ) -> bool {
        if !live_play {
            self.previous_ball_position = current_ball_position;
            self.pending_fifty_fifty_movement = None;
            return true;
        }
        if current_ball_position.is_none() {
            self.previous_ball_position = None;
            self.pending_fifty_fifty_movement = None;
            return true;
        }
        false
    }
}
