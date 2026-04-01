use boxcars::RemoteId;

use super::*;

fn rigid_body(x: f32, y: f32, z: f32, vx: f32, vy: f32, vz: f32) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: boxcars::Vector3f { x, y, z },
        rotation: boxcars::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(boxcars::Vector3f {
            x: vx,
            y: vy,
            z: vz,
        }),
        angular_velocity: Some(boxcars::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }),
    }
}

#[allow(clippy::too_many_arguments)]
fn sample(
    frame_number: usize,
    time: f32,
    ball_x: f32,
    ball_y: f32,
    ball_vx: f32,
    ball_vy: f32,
    touch_events: Vec<TouchEvent>,
    kickoff_countdown_time: Option<i32>,
    ball_has_been_hit: Option<bool>,
) -> StatsSample {
    StatsSample {
        frame_number,
        time,
        dt: 1.0 / 120.0,
        seconds_remaining: Some(100),
        game_state: Some(0),
        ball_has_been_hit,
        kickoff_countdown_time,
        team_zero_score: Some(0),
        team_one_score: Some(0),
        possession_team_is_team_0: None,
        scored_on_team_is_team_0: None,
        current_in_game_team_player_counts: Some([1, 1]),
        ball: Some(BallSample {
            rigid_body: rigid_body(ball_x, ball_y, 110.0, ball_vx, ball_vy, 0.0),
        }),
        players: vec![
            PlayerSample {
                player_id: RemoteId::Steam(1),
                is_team_0: true,
                rigid_body: Some(rigid_body(-120.0, -40.0, 17.0, 0.0, 0.0, 0.0)),
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
                rigid_body: Some(rigid_body(120.0, 40.0, 17.0, 0.0, 0.0, 0.0)),
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
        touch_events,
        dodge_refreshed_events: Vec::new(),
        player_stat_events: Vec::new(),
        goal_events: Vec::new(),
    }
}

#[test]
fn contested_touch_builds_horizontal_plane() {
    let touch_events = vec![
        TouchEvent {
            time: 0.0,
            frame: 10,
            team_is_team_0: true,
            player: Some(RemoteId::Steam(1)),
            closest_approach_distance: Some(0.0),
        },
        TouchEvent {
            time: 0.0,
            frame: 10,
            team_is_team_0: false,
            player: Some(RemoteId::Steam(2)),
            closest_approach_distance: Some(0.0),
        },
    ];
    let sample = sample(
        10,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        touch_events.clone(),
        Some(0),
        Some(false),
    );
    let active = FiftyFiftyReducer::contested_touch(&sample, &touch_events, true).unwrap();

    assert!(active.is_kickoff);
    assert_eq!(active.team_zero_player, Some(RemoteId::Steam(1)));
    assert_eq!(active.team_one_player, Some(RemoteId::Steam(2)));
    assert!(active.plane_normal_vec().z.abs() <= f32::EPSILON);
    assert!(active.plane_normal_vec().length() > 0.99);
}

#[test]
fn winning_team_uses_ball_side_and_velocity() {
    let touch_events = vec![
        TouchEvent {
            time: 0.0,
            frame: 10,
            team_is_team_0: true,
            player: Some(RemoteId::Steam(1)),
            closest_approach_distance: Some(0.0),
        },
        TouchEvent {
            time: 0.0,
            frame: 10,
            team_is_team_0: false,
            player: Some(RemoteId::Steam(2)),
            closest_approach_distance: Some(0.0),
        },
    ];
    let start = sample(
        10,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        touch_events.clone(),
        None,
        Some(true),
    );
    let active = FiftyFiftyReducer::contested_touch(&start, &touch_events, false).unwrap();

    let blue_side = sample(
        11,
        0.4,
        -220.0,
        0.0,
        -300.0,
        0.0,
        Vec::new(),
        None,
        Some(true),
    );
    let orange_side = sample(
        12,
        0.4,
        220.0,
        0.0,
        300.0,
        0.0,
        Vec::new(),
        None,
        Some(true),
    );

    assert_eq!(
        FiftyFiftyReducer::winning_team_from_ball(&active, &blue_side),
        Some(false)
    );
    assert_eq!(
        FiftyFiftyReducer::winning_team_from_ball(&active, &orange_side),
        Some(true)
    );
}

#[test]
fn reducer_tracks_kickoff_wins_and_possession_after() {
    let mut reducer = FiftyFiftyReducer::new();
    reducer.apply_event(&FiftyFiftyEvent {
        start_time: 0.1,
        start_frame: 10,
        resolve_time: 0.6,
        resolve_frame: 16,
        is_kickoff: true,
        team_zero_player: Some(RemoteId::Steam(1)),
        team_one_player: Some(RemoteId::Steam(2)),
        team_zero_position: [-120.0, -40.0, 17.0],
        team_one_position: [120.0, 40.0, 17.0],
        midpoint: [0.0, 0.0, 17.0],
        plane_normal: [0.95, 0.31, 0.0],
        winning_team_is_team_0: Some(false),
        possession_team_is_team_0: Some(false),
    });

    assert_eq!(reducer.stats().count, 1);
    assert_eq!(reducer.stats().kickoff_count, 1);
    assert_eq!(reducer.stats().team_one_wins, 1);
    assert_eq!(reducer.stats().kickoff_team_one_wins, 1);
    assert_eq!(reducer.stats().team_one_possession_after_count, 1);
    assert_eq!(reducer.stats().kickoff_team_one_possession_after_count, 1);

    let blue = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
    assert_eq!(blue.count, 1);
    assert_eq!(blue.losses, 1);
    assert_eq!(blue.kickoff_losses, 1);
    assert_eq!(blue.possession_after_count, 0);

    let orange = reducer.player_stats().get(&RemoteId::Steam(2)).unwrap();
    assert_eq!(orange.count, 1);
    assert_eq!(orange.wins, 1);
    assert_eq!(orange.kickoff_wins, 1);
    assert_eq!(orange.possession_after_count, 1);
    assert_eq!(orange.kickoff_possession_after_count, 1);
}
