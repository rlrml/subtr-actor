use super::*;

impl FiftyFiftyCalculator {
    pub(crate) fn kickoff_phase_active(gameplay: &GameplayState) -> bool {
        gameplay.game_state == Some(GAME_STATE_KICKOFF_COUNTDOWN)
            || gameplay.kickoff_countdown_time.is_some_and(|time| time > 0)
            || gameplay.ball_has_been_hit == Some(false)
    }

    pub(crate) fn contested_touch(
        frame: &FrameInfo,
        players: &PlayerFrameState,
        touch_events: &[TouchEvent],
        is_kickoff: bool,
    ) -> Option<ActiveFiftyFifty> {
        let team_zero_touch = touch_events.iter().find(|touch| touch.team_is_team_0)?;
        let team_one_touch = touch_events.iter().find(|touch| !touch.team_is_team_0)?;
        let team_zero_position = touch_player_position(players, team_zero_touch)?;
        let team_one_position = touch_player_position(players, team_one_touch)?;
        let midpoint = (team_zero_position + team_one_position) * 0.5;
        let plane_normal = fifty_fifty_plane_normal(team_zero_position, team_one_position);

        Some(ActiveFiftyFifty {
            start_time: frame.time,
            start_frame: frame.frame_number,
            last_touch_time: frame.time,
            last_touch_frame: frame.frame_number,
            is_kickoff,
            team_zero_player: team_zero_touch.player.clone(),
            team_one_player: team_one_touch.player.clone(),
            team_zero_touch_time: Some(team_zero_touch.time),
            team_zero_touch_frame: Some(team_zero_touch.frame),
            team_zero_dodge_contact: team_zero_touch.dodge_contact,
            team_one_touch_time: Some(team_one_touch.time),
            team_one_touch_frame: Some(team_one_touch.frame),
            team_one_dodge_contact: team_one_touch.dodge_contact,
            team_zero_position: team_zero_position.to_array(),
            team_one_position: team_one_position.to_array(),
            midpoint: midpoint.to_array(),
            plane_normal: plane_normal.to_array(),
        })
    }

    pub(crate) fn winning_team_from_ball(
        active: &ActiveFiftyFifty,
        ball: &BallFrameState,
    ) -> Option<bool> {
        let ball = ball.sample()?;
        let midpoint = active.midpoint_vec();
        let plane_normal = active.plane_normal_vec();
        let displacement = ball.position() - midpoint;
        let signed_distance = displacement.dot(plane_normal);
        if signed_distance.abs() >= FIFTY_FIFTY_MIN_EXIT_DISTANCE {
            return Some(signed_distance > 0.0);
        }

        let signed_speed = ball.velocity().dot(plane_normal);
        if signed_speed.abs() >= FIFTY_FIFTY_MIN_EXIT_SPEED {
            return Some(signed_speed > 0.0);
        }

        None
    }
}

fn touch_player_position(players: &PlayerFrameState, touch: &TouchEvent) -> Option<glam::Vec3> {
    let player_id = touch.player.as_ref()?;
    players
        .players
        .iter()
        .find(|player| &player.player_id == player_id)
        .and_then(PlayerSample::position)
}

fn fifty_fifty_plane_normal(
    team_zero_position: glam::Vec3,
    team_one_position: glam::Vec3,
) -> glam::Vec3 {
    let mut plane_normal = team_one_position - team_zero_position;
    plane_normal.z = 0.0;
    if plane_normal.length_squared() <= f32::EPSILON {
        glam::Vec3::Y
    } else {
        plane_normal.normalize()
    }
}
