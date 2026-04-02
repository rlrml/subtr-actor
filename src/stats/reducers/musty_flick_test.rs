use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};

use super::*;
use crate::stats::reducers::StatsReducer;

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
    player_dodge_active: bool,
    ball_body: RigidBody,
    touch: bool,
) -> CoreSample {
    CoreSample {
        frame_number,
        time,
        dt: 1.0 / 120.0,
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
                dodge_active: player_dodge_active,
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
fn counts_recent_backflip_style_touch_as_musty_flick() {
    let mut reducer = MustyFlickCalculator::new();

    let baseline = sample(
        0,
        0.0,
        rigid_body(
            glam::Vec3::new(0.0, 0.0, 220.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(-400.0, 0.0, 250.0),
            glam::Vec3::ZERO,
        ),
        false,
        rigid_body(
            glam::Vec3::new(-80.0, 0.0, 280.0),
            glam::Quat::IDENTITY,
            glam::Vec3::ZERO,
            glam::Vec3::ZERO,
        ),
        false,
    );
    reducer.on_sample(&baseline).unwrap();

    let dodge_start = sample(
        1,
        1.0 / 120.0,
        rigid_body(
            glam::Vec3::new(0.0, 0.0, 220.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(-500.0, 0.0, 300.0),
            glam::Vec3::new(0.0, 6.5, 0.0),
        ),
        true,
        rigid_body(
            glam::Vec3::new(-80.0, 0.0, 280.0),
            glam::Quat::IDENTITY,
            glam::Vec3::ZERO,
            glam::Vec3::ZERO,
        ),
        false,
    );
    reducer.on_sample(&dodge_start).unwrap();

    let musty_touch = sample(
        2,
        2.0 / 120.0,
        rigid_body(
            glam::Vec3::new(0.0, 0.0, 220.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(-800.0, 0.0, 600.0),
            glam::Vec3::new(0.0, 8.0, 0.0),
        ),
        true,
        rigid_body(
            glam::Vec3::new(-80.0, 0.0, 280.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(-1400.0, 0.0, 1000.0),
            glam::Vec3::ZERO,
        ),
        true,
    );
    reducer.on_sample(&musty_touch).unwrap();

    let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(stats.aerial_count, 1);
    assert_eq!(reducer.events().len(), 1);
    assert!(reducer.events()[0].confidence >= MUSTY_MIN_CONFIDENCE);
}

#[test]
fn rejects_backside_touch_without_recent_dodge_start() {
    let mut reducer = MustyFlickCalculator::new();

    let baseline = sample(
        0,
        0.0,
        rigid_body(
            glam::Vec3::new(0.0, 0.0, 220.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(-400.0, 0.0, 250.0),
            glam::Vec3::ZERO,
        ),
        false,
        rigid_body(
            glam::Vec3::new(-80.0, 0.0, 280.0),
            glam::Quat::IDENTITY,
            glam::Vec3::ZERO,
            glam::Vec3::ZERO,
        ),
        false,
    );
    reducer.on_sample(&baseline).unwrap();

    let backside_touch = sample(
        1,
        1.0 / 120.0,
        rigid_body(
            glam::Vec3::new(0.0, 0.0, 220.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(-800.0, 0.0, 600.0),
            glam::Vec3::new(0.0, 8.0, 0.0),
        ),
        false,
        rigid_body(
            glam::Vec3::new(-80.0, 0.0, 280.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(-1400.0, 0.0, 1000.0),
            glam::Vec3::ZERO,
        ),
        true,
    );
    reducer.on_sample(&backside_touch).unwrap();

    assert!(reducer.player_stats().is_empty());
    assert!(reducer.events().is_empty());
}
