use super::wall_aerial_event_parts::wall_aerial_event_from_parts;
use super::*;

impl WallAerialCalculator {
    pub(super) fn ball_speed_change(
        frame: &FrameInfo,
        ball: &BallFrameState,
        previous_ball_velocity: Option<glam::Vec3>,
    ) -> f32 {
        const BALL_GRAVITY_Z: f32 = -650.0;

        let Some(ball) = ball.sample() else {
            return 0.0;
        };
        let Some(previous_ball_velocity) = previous_ball_velocity else {
            return 0.0;
        };

        let expected_linear_delta = glam::Vec3::new(0.0, 0.0, BALL_GRAVITY_Z * frame.dt.max(0.0));
        let residual_linear_impulse =
            ball.velocity() - previous_ball_velocity - expected_linear_delta;
        residual_linear_impulse.length()
    }

    pub(super) fn player_position(
        players: &PlayerFrameState,
        player_id: &PlayerId,
    ) -> Option<glam::Vec3> {
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .and_then(PlayerSample::position)
    }

    pub(super) fn controlled_play_event(
        &self,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch: &TouchEvent,
        ball_speed_change: f32,
    ) -> Option<WallAerialEvent> {
        let player_id = touch.player.as_ref()?;
        let armed = self.armed_aerials.get(player_id)?;
        if armed.recorded {
            return None;
        }
        let player_position = Self::player_position(players, player_id)?;
        if !wall_aerial_player_position_is_valid(player_position) {
            return None;
        }
        let ball = ball.sample()?;
        let ball_position = ball.position();
        if !wall_aerial_ball_position_is_valid(ball_position) {
            return None;
        }
        let time_since_takeoff = touch.time - armed.takeoff_time;
        if !(0.0..=WALL_AERIAL_MAX_TAKEOFF_TO_TOUCH_SECONDS).contains(&time_since_takeoff) {
            return None;
        }
        Some(wall_aerial_event_from_parts(
            touch,
            armed,
            player_position,
            ball,
            time_since_takeoff,
            ball_speed_change,
        ))
    }
}

fn wall_aerial_player_position_is_valid(position: glam::Vec3) -> bool {
    !player_is_on_wall(position)
        && position.z >= WALL_AERIAL_MIN_TOUCH_PLAYER_Z
        && position.z >= WALL_AERIAL_MIN_CONTINUATION_PLAYER_Z
}

fn wall_aerial_ball_position_is_valid(position: glam::Vec3) -> bool {
    position.z >= WALL_AERIAL_MIN_TOUCH_BALL_Z
}
