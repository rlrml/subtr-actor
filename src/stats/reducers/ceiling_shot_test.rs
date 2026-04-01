use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};

use super::*;

fn rigid_body(
    location: glam::Vec3,
    rotation: glam::Quat,
    linear_velocity: glam::Vec3,
    angular_velocity: glam::Vec3,
) -> RigidBody {
    RigidBody {
        sleeping: false,
        location: boxcars::Vector3f {
            x: location.x,
            y: location.y,
            z: location.z,
        },
        rotation: Quaternion {
            x: rotation.x,
            y: rotation.y,
            z: rotation.z,
            w: rotation.w,
        },
        linear_velocity: Some(Vector3f {
            x: linear_velocity.x,
            y: linear_velocity.y,
            z: linear_velocity.z,
        }),
        angular_velocity: Some(Vector3f {
            x: angular_velocity.x,
            y: angular_velocity.y,
            z: angular_velocity.z,
        }),
    }
}

fn sample(
    frame_number: usize,
    time: f32,
    player_body: RigidBody,
    ball_body: RigidBody,
    touch: bool,
) -> StatsSample {
    StatsSample {
        frame_number,
        time,
        dt: if frame_number == 0 { 0.0 } else { 1.0 / 120.0 },
        seconds_remaining: None,
        game_state: None,
        ball_has_been_hit: Some(true),
        kickoff_countdown_time: None,
        team_zero_score: Some(0),
        team_one_score: Some(0),
        possession_team_is_team_0: Some(true),
        scored_on_team_is_team_0: None,
        current_in_game_team_player_counts: Some([1, 1]),
        ball: Some(BallSample {
            rigid_body: ball_body,
        }),
        players: vec![
            PlayerSample {
                player_id: RemoteId::Steam(1),
                is_team_0: true,
                rigid_body: Some(player_body),
                boost_amount: None,
                last_boost_amount: None,
                boost_active: false,
                dodge_active: false,
                powerslide_active: false,
                match_goals: None,
                match_assists: None,
                match_saves: None,
                match_shots: None,
                match_score: None,
            },
            PlayerSample {
                player_id: RemoteId::Steam(2),
                is_team_0: false,
                rigid_body: Some(rigid_body(
                    glam::Vec3::new(3000.0, 0.0, 17.0),
                    glam::Quat::IDENTITY,
                    glam::Vec3::ZERO,
                    glam::Vec3::ZERO,
                )),
                boost_amount: None,
                last_boost_amount: None,
                boost_active: false,
                dodge_active: false,
                powerslide_active: false,
                match_goals: None,
                match_assists: None,
                match_saves: None,
                match_shots: None,
                match_score: None,
            },
        ],
        active_demos: Vec::new(),
        demo_events: Vec::new(),
        boost_pad_events: Vec::new(),
        touch_events: if touch {
            vec![TouchEvent {
                time,
                frame: frame_number,
                team_is_team_0: true,
                player: Some(RemoteId::Steam(1)),
                closest_approach_distance: Some(0.0),
            }]
        } else {
            Vec::new()
        },
        dodge_refreshed_events: Vec::new(),
        player_stat_events: Vec::new(),
        goal_events: Vec::new(),
    }
}

#[test]
fn counts_touch_shortly_after_ceiling_contact_as_ceiling_shot() {
    let mut reducer = CeilingShotReducer::new();

    let on_ceiling = sample(
        0,
        0.0,
        rigid_body(
            glam::Vec3::new(0.0, -900.0, 1988.0),
            glam::Quat::from_rotation_y(std::f32::consts::PI),
            glam::Vec3::new(0.0, 400.0, 0.0),
            glam::Vec3::ZERO,
        ),
        rigid_body(
            glam::Vec3::new(220.0, -720.0, 1350.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(0.0, 0.0, 0.0),
            glam::Vec3::ZERO,
        ),
        false,
    );
    reducer.on_sample(&on_ceiling).unwrap();

    let ceiling_shot_touch = sample(
        1,
        1.0 / 120.0,
        rigid_body(
            glam::Vec3::new(0.0, -760.0, 1780.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(700.0, 260.0, -520.0),
            glam::Vec3::ZERO,
        ),
        rigid_body(
            glam::Vec3::new(115.0, -735.0, 1835.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(1250.0, 540.0, 420.0),
            glam::Vec3::ZERO,
        ),
        true,
    );
    reducer.on_sample(&ceiling_shot_touch).unwrap();

    assert_eq!(reducer.events().len(), 1);
    assert_eq!(reducer.player_stats()[&RemoteId::Steam(1)].count, 1);
    assert!(reducer.events()[0].confidence >= CEILING_SHOT_MIN_CONFIDENCE);
    assert!(
        reducer.events()[0].time_since_ceiling_contact
            <= CEILING_SHOT_MAX_TOUCH_AFTER_CONTACT_SECONDS
    );
}

#[test]
fn rejects_touch_without_prior_ceiling_contact() {
    let mut reducer = CeilingShotReducer::new();

    let ordinary_touch = sample(
        0,
        0.0,
        rigid_body(
            glam::Vec3::new(0.0, -760.0, 540.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(700.0, 260.0, 0.0),
            glam::Vec3::ZERO,
        ),
        rigid_body(
            glam::Vec3::new(115.0, -735.0, 585.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(1250.0, 540.0, 420.0),
            glam::Vec3::ZERO,
        ),
        true,
    );
    reducer.on_sample(&ordinary_touch).unwrap();

    assert!(reducer.events().is_empty());
}
