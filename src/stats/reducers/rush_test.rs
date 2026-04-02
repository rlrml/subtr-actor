use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};

use super::*;

fn rigid_body(x: f32, y: f32) -> RigidBody {
    RigidBody {
        sleeping: false,
        location: Vector3f { x, y, z: 17.0 },
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

fn player(player_id: u64, is_team_0: bool, x: f32, y: f32) -> PlayerSample {
    PlayerSample {
        player_id: RemoteId::Steam(player_id),
        is_team_0,
        rigid_body: Some(rigid_body(x, y)),
        boost_amount: Some(50.0),
        last_boost_amount: Some(50.0),
        boost_active: false,
        dodge_active: false,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

fn sample_with_ball_y(players: Vec<PlayerSample>, ball_y: f32) -> CoreSample {
    CoreSample {
        frame_number: 10,
        time: 5.0,
        dt: 1.0 / 120.0,
        seconds_remaining: None,
        game_state: None,
        ball_has_been_hit: Some(true),
        kickoff_countdown_time: None,
        team_zero_score: Some(0),
        team_one_score: Some(0),
        possession_team_is_team_0: Some(true),
        scored_on_team_is_team_0: None,
        current_in_game_team_player_counts: Some([3, 3]),
        ball: Some(BallSample {
            rigid_body: rigid_body(0.0, ball_y),
        }),
        players,
        active_demos: Vec::new(),
        demo_events: Vec::new(),
        boost_pad_events: Vec::new(),
        touch_events: Vec::new(),
        dodge_refreshed_events: Vec::new(),
        player_stat_events: Vec::new(),
        goal_events: Vec::new(),
    }
}

fn sample(players: Vec<PlayerSample>) -> CoreSample {
    sample_with_ball_y(players, -200.0)
}

#[test]
fn classifies_two_v_one_from_turnover_shape() {
    let reducer = RushCalculator::new();
    let sample = sample(vec![
        player(1, true, 0.0, -500.0),
        player(2, true, 300.0, 250.0),
        player(3, true, -1500.0, -2600.0),
        player(4, false, 0.0, 1800.0),
        player(5, false, 800.0, -150.0),
        player(6, false, -900.0, -1800.0),
    ]);

    assert_eq!(reducer.rush_numbers(&sample, true), Some((2, 1)));
}

#[test]
fn counts_rush_once_when_possession_changes() {
    let start_sample = sample(vec![
        player(1, true, 0.0, -500.0),
        player(2, true, 300.0, 250.0),
        player(3, true, -1500.0, -2600.0),
        player(4, false, 0.0, 1800.0),
        player(5, false, 800.0, -150.0),
        player(6, false, -900.0, -1800.0),
    ]);
    let continue_sample = CoreSample {
        frame_number: 11,
        time: 5.1,
        ..sample(vec![
            player(1, true, 0.0, -450.0),
            player(2, true, 300.0, 300.0),
            player(3, true, -1500.0, -2200.0),
            player(4, false, 0.0, 1700.0),
            player(5, false, 800.0, -100.0),
            player(6, false, -900.0, -1700.0),
        ])
    };

    let mut reducer = RushCalculator::with_config(RushCalculatorConfig {
        min_possession_retained_seconds: 0.05,
        ..RushCalculatorConfig::default()
    });
    reducer.update_rush_state(&start_sample, Some(false), Some(true));
    assert_eq!(reducer.stats().team_zero_count, 0);
    assert_eq!(reducer.stats().team_zero_two_v_one_count, 0);
    assert_eq!(reducer.events().len(), 0);

    reducer.update_rush_state(&continue_sample, Some(true), Some(true));
    assert_eq!(reducer.stats().team_zero_count, 1);
    assert_eq!(reducer.stats().team_zero_two_v_one_count, 1);
    assert_eq!(reducer.events().len(), 0);

    reducer.update_rush_state(&continue_sample, Some(true), Some(true));
    assert_eq!(reducer.stats().team_zero_count, 1);
    assert_eq!(reducer.stats().team_zero_two_v_one_count, 1);
}

#[test]
fn does_not_count_rush_when_turnover_starts_at_midfield() {
    let reducer = RushCalculator::new();
    let sample = sample_with_ball_y(
        vec![
            player(1, true, 0.0, -500.0),
            player(2, true, 300.0, 250.0),
            player(3, true, -1500.0, -2600.0),
            player(4, false, 0.0, 1800.0),
            player(5, false, 800.0, -150.0),
            player(6, false, -900.0, -1800.0),
        ],
        0.0,
    );

    assert_eq!(reducer.rush_numbers(&sample, true), None);
}

#[test]
fn records_rush_event_with_start_and_end_frames() {
    let mut reducer = RushCalculator::with_config(RushCalculatorConfig {
        min_possession_retained_seconds: 0.05,
        ..RushCalculatorConfig::default()
    });
    let start_sample = sample(vec![
        player(1, true, 0.0, -500.0),
        player(2, true, 300.0, 250.0),
        player(3, true, -1500.0, -2600.0),
        player(4, false, 0.0, 1800.0),
        player(5, false, 800.0, -150.0),
        player(6, false, -900.0, -1800.0),
    ]);
    let continue_sample = CoreSample {
        frame_number: 11,
        time: 5.1,
        ..sample(vec![
            player(1, true, 0.0, -450.0),
            player(2, true, 300.0, 300.0),
            player(3, true, -1500.0, -2200.0),
            player(4, false, 0.0, 1700.0),
            player(5, false, 800.0, -100.0),
            player(6, false, -900.0, -1700.0),
        ])
    };
    let end_sample = CoreSample {
        frame_number: 12,
        time: 5.2,
        ..sample_with_ball_y(
            vec![
                player(1, true, 0.0, -200.0),
                player(2, true, 300.0, 700.0),
                player(3, true, -1500.0, -1800.0),
                player(4, false, 0.0, 1800.0),
                player(5, false, 800.0, 100.0),
                player(6, false, -900.0, -1500.0),
            ],
            300.0,
        )
    };

    reducer.update_rush_state(&start_sample, Some(false), Some(true));
    reducer.update_rush_state(&continue_sample, Some(true), Some(true));
    reducer.update_rush_state(&end_sample, Some(true), Some(true));

    assert_eq!(reducer.stats().team_zero_count, 1);
    assert_eq!(
        reducer.events(),
        &[RushEvent {
            start_time: 5.0,
            start_frame: 10,
            end_time: 5.1,
            end_frame: 11,
            is_team_0: true,
            attackers: 2,
            defenders: 1,
        }]
    );
}

#[test]
fn does_not_count_short_lived_rush_before_retention_threshold() {
    let mut reducer = RushCalculator::with_config(RushCalculatorConfig {
        min_possession_retained_seconds: 0.2,
        ..RushCalculatorConfig::default()
    });
    let start_sample = sample(vec![
        player(1, true, 0.0, -500.0),
        player(2, true, 300.0, 250.0),
        player(3, true, -1500.0, -2600.0),
        player(4, false, 0.0, 1800.0),
        player(5, false, 800.0, -150.0),
        player(6, false, -900.0, -1800.0),
    ]);
    let brief_continue_sample = CoreSample {
        frame_number: 11,
        time: 5.05,
        ..sample(vec![
            player(1, true, 0.0, -450.0),
            player(2, true, 300.0, 300.0),
            player(3, true, -1500.0, -2200.0),
            player(4, false, 0.0, 1700.0),
            player(5, false, 800.0, -100.0),
            player(6, false, -900.0, -1700.0),
        ])
    };
    let end_sample = CoreSample {
        frame_number: 12,
        time: 5.1,
        ..sample_with_ball_y(
            vec![
                player(1, true, 0.0, -200.0),
                player(2, true, 300.0, 700.0),
                player(3, true, -1500.0, -1800.0),
                player(4, false, 0.0, 1800.0),
                player(5, false, 800.0, 100.0),
                player(6, false, -900.0, -1500.0),
            ],
            300.0,
        )
    };

    reducer.update_rush_state(&start_sample, Some(false), Some(true));
    reducer.update_rush_state(&brief_continue_sample, Some(true), Some(true));
    reducer.update_rush_state(&end_sample, Some(true), Some(true));

    assert_eq!(reducer.stats().team_zero_count, 0);
    assert!(reducer.events().is_empty());
}
