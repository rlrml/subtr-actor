use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};

use super::*;
use crate::stats::reducers::StatsReducer;

fn ball(ball_y: f32) -> BallSample {
    BallSample {
        rigid_body: RigidBody {
            sleeping: false,
            location: Vector3f {
                x: 0.0,
                y: ball_y,
                z: 100.0,
            },
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
        },
    }
}

fn sample(frame_number: usize, time: f32, ball_y: f32, touch_teams: &[bool]) -> FrameState {
    FrameState {
        frame_number,
        time,
        dt: 1.0,
        seconds_remaining: None,
        game_state: None,
        ball_has_been_hit: Some(true),
        kickoff_countdown_time: None,
        team_zero_score: Some(0),
        team_one_score: Some(0),
        possession_team_is_team_0: None,
        scored_on_team_is_team_0: None,
        current_in_game_team_player_counts: Some([1, 1]),
        ball: Some(ball(ball_y)),
        players: Vec::new(),
        active_demos: Vec::new(),
        demo_events: Vec::new(),
        boost_pad_events: Vec::new(),
        touch_events: touch_teams
            .iter()
            .enumerate()
            .map(|(index, &team_is_team_0)| TouchEvent {
                time,
                frame: frame_number,
                player: Some(RemoteId::Steam(index as u64 + 1)),
                team_is_team_0,
                closest_approach_distance: None,
            })
            .collect(),
        dodge_refreshed_events: Vec::new(),
        player_stat_events: Vec::new(),
        goal_events: Vec::new(),
    }
}

#[test]
fn possession_reducer_tracks_labeled_possession_time() {
    let mut reducer = PossessionCalculator::new();

    reducer.on_sample(&sample(0, 0.0, 0.0, &[])).unwrap();
    reducer.on_sample(&sample(1, 1.0, 0.0, &[true])).unwrap();
    reducer.on_sample(&sample(2, 2.0, 0.0, &[])).unwrap();
    reducer.on_sample(&sample(3, 3.0, 0.0, &[false])).unwrap();
    reducer.on_sample(&sample(4, 4.0, 0.0, &[false])).unwrap();
    reducer.on_sample(&sample(5, 5.0, 0.0, &[])).unwrap();

    let stats = reducer.stats();
    assert_eq!(stats.tracked_time, 6.0);
    assert_eq!(stats.neutral_time, 2.0);
    assert_eq!(stats.team_zero_time, 3.0);
    assert_eq!(stats.team_one_time, 1.0);
    assert_eq!(
        stats.time_with_labels(&[StatLabel::new("possession_state", "neutral")]),
        2.0
    );
    assert_eq!(
        stats.time_with_labels(&[StatLabel::new("possession_state", "team_zero")]),
        3.0
    );
    assert_eq!(
        stats.time_with_labels(&[StatLabel::new("possession_state", "team_one")]),
        1.0
    );
}

#[test]
fn possession_reducer_tracks_possession_time_by_field_third() {
    let mut reducer = PossessionCalculator::new();

    reducer.on_sample(&sample(0, 0.0, -3000.0, &[])).unwrap();
    reducer.on_sample(&sample(1, 1.0, 0.0, &[true])).unwrap();
    reducer
        .on_sample(&sample(2, 2.0, -3000.0, &[true]))
        .unwrap();
    reducer.on_sample(&sample(3, 3.0, 0.0, &[])).unwrap();
    reducer.on_sample(&sample(4, 4.0, 3000.0, &[])).unwrap();
    reducer
        .on_sample(&sample(5, 5.0, 3000.0, &[false]))
        .unwrap();
    reducer
        .on_sample(&sample(6, 6.0, -3000.0, &[false]))
        .unwrap();

    let stats = reducer.stats();
    assert_eq!(stats.tracked_time, 7.0);
    assert_eq!(
        stats.time_with_labels(&[
            StatLabel::new("possession_state", "neutral"),
            StatLabel::new("field_third", "team_zero_third"),
        ]),
        1.0
    );
    assert_eq!(
        stats.time_with_labels(&[
            StatLabel::new("possession_state", "neutral"),
            StatLabel::new("field_third", "neutral_third"),
        ]),
        1.0
    );
    assert_eq!(
        stats.time_with_labels(&[
            StatLabel::new("possession_state", "neutral"),
            StatLabel::new("field_third", "team_one_third"),
        ]),
        1.0
    );
    assert_eq!(
        stats.time_with_labels(&[
            StatLabel::new("possession_state", "team_zero"),
            StatLabel::new("field_third", "team_zero_third"),
        ]),
        1.0
    );
    assert_eq!(
        stats.time_with_labels(&[
            StatLabel::new("possession_state", "team_zero"),
            StatLabel::new("field_third", "neutral_third"),
        ]),
        1.0
    );
    assert_eq!(
        stats.time_with_labels(&[
            StatLabel::new("possession_state", "team_zero"),
            StatLabel::new("field_third", "team_one_third"),
        ]),
        1.0
    );
    assert_eq!(
        stats.time_with_labels(&[
            StatLabel::new("possession_state", "team_one"),
            StatLabel::new("field_third", "team_zero_third"),
        ]),
        1.0
    );
}
