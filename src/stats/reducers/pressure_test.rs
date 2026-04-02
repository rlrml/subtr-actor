use boxcars::{Quaternion, RigidBody, Vector3f};

use super::*;
use crate::stats::reducers::StatsReducer;

fn ball(y: f32) -> BallSample {
    BallSample {
        rigid_body: RigidBody {
            sleeping: false,
            location: Vector3f {
                x: 0.0,
                y,
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

fn sample(frame_number: usize, time: f32, ball_y: f32) -> CoreSample {
    CoreSample {
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
        touch_events: Vec::new(),
        dodge_refreshed_events: Vec::new(),
        player_stat_events: Vec::new(),
        goal_events: Vec::new(),
    }
}

#[test]
fn pressure_reducer_tracks_labeled_half_time_with_neutral_zone() {
    let mut reducer = PressureCalculator::new();

    reducer.on_sample(&sample(0, 0.0, -250.0)).unwrap();
    reducer.on_sample(&sample(1, 1.0, 0.0)).unwrap();
    reducer.on_sample(&sample(2, 2.0, 250.0)).unwrap();

    let stats = reducer.stats();
    assert_eq!(stats.tracked_time, 3.0);
    assert_eq!(stats.team_zero_side_time, 1.0);
    assert_eq!(stats.team_one_side_time, 1.0);
    assert_eq!(stats.neutral_time, 1.0);
    assert_eq!(
        stats.time_with_labels(&[StatLabel::new("field_half", "team_zero_side")]),
        1.0
    );
    assert_eq!(
        stats.time_with_labels(&[StatLabel::new("field_half", "team_one_side")]),
        1.0
    );
    assert_eq!(
        stats.time_with_labels(&[StatLabel::new("field_half", "neutral")]),
        1.0
    );
}
