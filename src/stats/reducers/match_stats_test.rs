use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};

use super::*;
use crate::stats::reducers::StatsReducer;

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

fn player(player_id: u64, is_team_0: bool, match_goals: i32) -> PlayerSample {
    PlayerSample {
        player_id: RemoteId::Steam(player_id),
        is_team_0,
        rigid_body: Some(rigid_body(if is_team_0 { -1000.0 } else { 1000.0 })),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active: false,
        powerslide_active: false,
        match_goals: Some(match_goals),
        match_assists: Some(0),
        match_saves: Some(0),
        match_shots: Some(match_goals.max(1)),
        match_score: Some(match_goals * 100),
    }
}

fn sample(
    frame_number: usize,
    time: f32,
    dt: f32,
    ball_y: f32,
    team_zero_goals: i32,
    goal_event: Option<GoalEvent>,
) -> CoreSample {
    CoreSample {
        frame_number,
        time,
        dt,
        seconds_remaining: None,
        game_state: None,
        ball_has_been_hit: Some(true),
        kickoff_countdown_time: None,
        team_zero_score: Some(team_zero_goals),
        team_one_score: Some(0),
        possession_team_is_team_0: Some(true),
        scored_on_team_is_team_0: goal_event
            .as_ref()
            .map(|event| !event.scoring_team_is_team_0),
        current_in_game_team_player_counts: Some([1, 1]),
        ball: Some(BallSample {
            rigid_body: rigid_body(ball_y),
        }),
        players: vec![player(1, true, team_zero_goals), player(2, false, 0)],
        active_demos: Vec::new(),
        demo_events: Vec::new(),
        boost_pad_events: Vec::new(),
        touch_events: Vec::new(),
        dodge_refreshed_events: Vec::new(),
        player_stat_events: Vec::new(),
        goal_events: goal_event.into_iter().collect(),
    }
}

#[test]
fn classifies_counter_attack_goals_from_recent_defensive_pressure() {
    let mut reducer = MatchStatsCalculator::new();

    for (index, ball_y) in [-4200.0, -4000.0, -3600.0, -3200.0, -2600.0, -1800.0, 1200.0]
        .into_iter()
        .enumerate()
    {
        reducer
            .on_sample(&sample(index, index as f32 + 1.0, 1.0, ball_y, 0, None))
            .unwrap();
    }

    reducer
        .on_sample(&sample(
            8,
            8.0,
            1.0,
            4800.0,
            1,
            Some(GoalEvent {
                time: 8.0,
                frame: 8,
                scoring_team_is_team_0: true,
                player: Some(RemoteId::Steam(1)),
                team_zero_score: Some(1),
                team_one_score: Some(0),
            }),
        ))
        .unwrap();

    let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
    assert_eq!(stats.goal_buildup.counter_attack_goal_count, 1);
    assert_eq!(stats.goal_buildup.sustained_pressure_goal_count, 0);
    assert_eq!(stats.goal_buildup.other_buildup_goal_count, 0);
}

#[test]
fn classifies_sustained_pressure_goals_after_long_attacking_spell() {
    let mut reducer = MatchStatsCalculator::new();

    for (index, ball_y) in [
        800.0, 1400.0, 2200.0, 2800.0, 3200.0, 3600.0, 4100.0, 4600.0,
    ]
    .into_iter()
    .enumerate()
    {
        reducer
            .on_sample(&sample(index, index as f32 + 1.0, 1.0, ball_y, 0, None))
            .unwrap();
    }

    reducer
        .on_sample(&sample(
            9,
            9.0,
            1.0,
            5000.0,
            1,
            Some(GoalEvent {
                time: 9.0,
                frame: 9,
                scoring_team_is_team_0: true,
                player: Some(RemoteId::Steam(1)),
                team_zero_score: Some(1),
                team_one_score: Some(0),
            }),
        ))
        .unwrap();

    let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
    assert_eq!(stats.goal_buildup.counter_attack_goal_count, 0);
    assert_eq!(stats.goal_buildup.sustained_pressure_goal_count, 1);
    assert_eq!(stats.goal_buildup.other_buildup_goal_count, 0);
}
