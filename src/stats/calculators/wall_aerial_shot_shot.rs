use super::*;

impl WallAerialShotCalculator {
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

    pub(super) fn shot_event(
        &self,
        players: &PlayerFrameState,
        event: &PlayerStatEvent,
    ) -> Option<WallAerialShotEvent> {
        if event.kind != PlayerStatEventKind::Shot {
            return None;
        }
        let armed = self.armed_shots.get(&event.player)?;
        let time_since_takeoff = event.time - armed.takeoff_time;
        if !(0.0..=WALL_AERIAL_SHOT_MAX_TAKEOFF_TO_SHOT_SECONDS).contains(&time_since_takeoff) {
            return None;
        }

        let player_position = event
            .shot
            .as_ref()
            .and_then(|shot| shot.player_position.as_ref().map(vec_to_glam))
            .or_else(|| Self::player_position(players, &event.player))?;
        if player_is_on_wall(player_position) || player_position.z < WALL_AERIAL_MIN_TOUCH_PLAYER_Z
        {
            return None;
        }

        let shot = event.shot.as_ref()?;
        let ball_position = vec_to_glam(&shot.ball_position);
        if ball_position.z < WALL_AERIAL_MIN_TOUCH_BALL_Z {
            return None;
        }

        Some(wall_aerial_shot_event(
            event,
            armed,
            time_since_takeoff,
            player_position,
            ball_position,
            shot,
        ))
    }
}

fn wall_aerial_shot_event(
    event: &PlayerStatEvent,
    armed: &ArmedWallAerialShot,
    time_since_takeoff: f32,
    player_position: glam::Vec3,
    ball_position: glam::Vec3,
    shot: &ShotEventMetadata,
) -> WallAerialShotEvent {
    WallAerialShotEvent {
        time: event.time,
        frame: event.frame,
        player: event.player.clone(),
        is_team_0: event.is_team_0,
        wall: armed.wall,
        wall_contact_time: armed.wall_contact_time,
        wall_contact_frame: armed.wall_contact_frame,
        takeoff_time: armed.takeoff_time,
        takeoff_frame: armed.takeoff_frame,
        time_since_takeoff,
        wall_contact_position: armed.wall_contact_position.to_array(),
        takeoff_position: armed.takeoff_position.to_array(),
        player_position: player_position.to_array(),
        ball_position: ball_position.to_array(),
        ball_speed: shot.ball_speed,
        goal_alignment: shot.ball_goal_alignment,
        confidence: wall_aerial_shot_confidence(
            time_since_takeoff,
            player_position,
            shot.ball_speed,
            shot.ball_goal_alignment,
        ),
    }
}
