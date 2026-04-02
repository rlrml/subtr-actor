use boxcars::RemoteId;

use super::*;
use crate::stats::reducers::StatsReducer;

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

fn sample(
    frame_number: usize,
    time: f32,
    player_z: f32,
    ball_velocity_x: f32,
    touch: bool,
) -> FrameState {
    FrameState {
        frame_number,
        time,
        dt: 1.0 / 120.0,
        seconds_remaining: None,
        game_state: None,
        ball_has_been_hit: None,
        kickoff_countdown_time: None,
        team_zero_score: None,
        team_one_score: None,
        possession_team_is_team_0: Some(true),
        scored_on_team_is_team_0: None,
        current_in_game_team_player_counts: Some([1, 1]),
        ball: Some(BallSample {
            rigid_body: rigid_body(0.0, 0.0, 120.0, ball_velocity_x, 0.0, 0.0),
        }),
        players: vec![
            PlayerSample {
                player_id: RemoteId::Steam(1),
                is_team_0: true,
                rigid_body: Some(rigid_body(0.0, 0.0, player_z, 0.0, 0.0, 0.0)),
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
                rigid_body: Some(rigid_body(4000.0, 0.0, 0.0, 0.0, 0.0, 0.0)),
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
fn touch_reducer_classifies_touch_strength_and_height_bands() {
    let mut reducer = TouchCalculator::new();

    let baseline = sample(0, 0.0, 0.0, 0.0, false);
    reducer.on_sample(&baseline).unwrap();

    let dribble = sample(1, 1.0 / 120.0, 0.0, 120.0, true);
    reducer.on_sample(&dribble).unwrap();

    let control = sample(2, 2.0 / 120.0, 240.0, 220.0, true);
    reducer.on_sample(&control).unwrap();

    let medium = sample(3, 3.0 / 120.0, 0.0, 720.0, true);
    reducer.on_sample(&medium).unwrap();

    let hard_high_aerial = sample(4, 4.0 / 120.0, 900.0, 1900.0, true);
    reducer.on_sample(&hard_high_aerial).unwrap();

    let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
    assert_eq!(stats.touch_count, 4);
    assert_eq!(stats.dribble_touch_count, 1);
    assert_eq!(stats.control_touch_count, 1);
    assert_eq!(stats.medium_hit_count, 1);
    assert_eq!(stats.hard_hit_count, 1);
    assert_eq!(stats.aerial_touch_count, 2);
    assert_eq!(stats.high_aerial_touch_count, 1);
    assert_eq!(
        stats.touch_count_with_labels(&[StatLabel::new("kind", "dribble")]),
        1
    );
    assert_eq!(
        stats.touch_count_with_labels(&[StatLabel::new("height_band", "low_air")]),
        1
    );
    assert_eq!(
        stats.touch_count_with_labels(&[StatLabel::new("height_band", "high_air")]),
        1
    );
    assert_eq!(
        stats.touch_count_with_labels(&[
            StatLabel::new("kind", "hard_hit"),
            StatLabel::new("height_band", "high_air"),
        ]),
        1
    );
    assert!(stats.last_ball_speed_change.is_some());
    assert!(stats.max_ball_speed_change >= stats.average_ball_speed_change());
}

#[test]
fn touch_stats_complete_labeled_touch_counts_adds_zero_entries() {
    let mut stats = TouchStats::default();
    stats.labeled_touch_counts.increment([
        StatLabel::new("kind", "hard_hit"),
        StatLabel::new("height_band", "high_air"),
    ]);

    let completed = stats.complete_labeled_touch_counts();

    assert_eq!(completed.entries.len(), 12);
    assert_eq!(
        completed.count_exact(&[
            StatLabel::new("kind", "hard_hit"),
            StatLabel::new("height_band", "high_air"),
        ]),
        1
    );
    assert_eq!(
        completed.count_exact(&[
            StatLabel::new("kind", "dribble"),
            StatLabel::new("height_band", "ground"),
        ]),
        0
    );
}
