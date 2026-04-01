use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};

use super::*;

fn rigid_body(y: f32) -> RigidBody {
    RigidBody {
        sleeping: false,
        location: Vector3f { x: 0.0, y, z: 17.0 },
        rotation: Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }),
        angular_velocity: Some(Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }),
    }
}

fn player(player_id: u64, is_team_0: bool, y: f32) -> PlayerSample {
    PlayerSample {
        player_id: RemoteId::Steam(player_id),
        is_team_0,
        rigid_body: Some(rigid_body(y)),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active: false,
        powerslide_active: false,
        match_goals: Some(0),
        match_assists: Some(0),
        match_saves: Some(0),
        match_shots: Some(0),
        match_score: Some(0),
    }
}

fn sample(
    frame_number: usize,
    time: f32,
    touch_players: &[(u64, bool)],
    kickoff_phase_active: bool,
) -> StatsSample {
    StatsSample {
        frame_number,
        time,
        dt: 1.0,
        seconds_remaining: None,
        game_state: kickoff_phase_active.then_some(55),
        ball_has_been_hit: Some(!kickoff_phase_active),
        kickoff_countdown_time: kickoff_phase_active.then_some(3),
        team_zero_score: Some(0),
        team_one_score: Some(0),
        possession_team_is_team_0: None,
        scored_on_team_is_team_0: None,
        current_in_game_team_player_counts: Some([2, 1]),
        ball: Some(BallSample {
            rigid_body: rigid_body(0.0),
        }),
        players: vec![
            player(1, true, -400.0),
            player(2, true, -100.0),
            player(3, false, 300.0),
        ],
        active_demos: Vec::new(),
        demo_events: Vec::new(),
        boost_pad_events: Vec::new(),
        touch_events: touch_players
            .iter()
            .map(|(player_id, team_is_team_0)| TouchEvent {
                time,
                frame: frame_number,
                player: Some(RemoteId::Steam(*player_id)),
                team_is_team_0: *team_is_team_0,
                closest_approach_distance: None,
            })
            .collect(),
        dodge_refreshed_events: Vec::new(),
        player_stat_events: Vec::new(),
        goal_events: Vec::new(),
    }
}

#[test]
fn counts_defenders_caught_ahead_of_play_on_goal_frames() {
    let mut reducer = PositioningReducer::new();
    let sample = StatsSample {
        frame_number: 10,
        time: 10.0,
        dt: 1.0,
        seconds_remaining: None,
        game_state: None,
        ball_has_been_hit: Some(true),
        kickoff_countdown_time: None,
        team_zero_score: Some(1),
        team_one_score: Some(0),
        possession_team_is_team_0: Some(true),
        scored_on_team_is_team_0: Some(false),
        current_in_game_team_player_counts: Some([1, 3]),
        ball: Some(BallSample {
            rigid_body: rigid_body(4800.0),
        }),
        players: vec![
            player(1, true, 0.0),
            player(2, false, -1800.0),
            player(3, false, -700.0),
            player(4, false, 3200.0),
        ],
        active_demos: Vec::new(),
        demo_events: Vec::new(),
        boost_pad_events: Vec::new(),
        touch_events: Vec::new(),
        dodge_refreshed_events: Vec::new(),
        player_stat_events: Vec::new(),
        goal_events: vec![GoalEvent {
            time: 10.0,
            frame: 10,
            scoring_team_is_team_0: true,
            player: Some(RemoteId::Steam(1)),
            team_zero_score: Some(1),
            team_one_score: Some(0),
        }],
    };

    reducer.on_sample(&sample).unwrap();

    assert_eq!(
        reducer
            .player_stats()
            .get(&RemoteId::Steam(2))
            .unwrap()
            .times_caught_ahead_of_play_on_conceded_goals,
        1
    );
    assert_eq!(
        reducer
            .player_stats()
            .get(&RemoteId::Steam(3))
            .unwrap()
            .times_caught_ahead_of_play_on_conceded_goals,
        1
    );
    assert_eq!(
        reducer
            .player_stats()
            .get(&RemoteId::Steam(4))
            .unwrap()
            .times_caught_ahead_of_play_on_conceded_goals,
        0
    );
}

#[test]
fn player_possession_is_exclusive_and_resets_on_kickoff() {
    let mut reducer = PositioningReducer::new();

    reducer.on_sample(&sample(0, 0.0, &[], false)).unwrap();
    reducer
        .on_sample(&sample(1, 1.0, &[(1, true)], false))
        .unwrap();
    reducer.on_sample(&sample(2, 2.0, &[], false)).unwrap();
    reducer
        .on_sample(&sample(3, 3.0, &[(2, true)], false))
        .unwrap();
    reducer.on_sample(&sample(4, 4.0, &[], false)).unwrap();
    reducer.on_sample(&sample(5, 5.0, &[], true)).unwrap();
    reducer.on_sample(&sample(6, 6.0, &[], false)).unwrap();

    let player_one = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
    let player_two = reducer.player_stats().get(&RemoteId::Steam(2)).unwrap();
    let player_three = reducer.player_stats().get(&RemoteId::Steam(3)).unwrap();

    assert_eq!(player_one.time_has_possession, 2.0);
    assert_eq!(player_two.time_has_possession, 1.0);
    assert_eq!(player_three.time_has_possession, 0.0);
    assert_eq!(
        player_one.time_has_possession
            + player_two.time_has_possession
            + player_three.time_has_possession,
        3.0
    );
    assert_eq!(player_one.time_no_possession, 1.0);
    assert_eq!(player_two.time_no_possession, 2.0);
    assert_eq!(player_three.time_no_possession, 3.0);
}
