use boxcars::RemoteId;

use super::*;

fn rigid_body(x: f32, y: f32, z: f32, vx: f32) -> boxcars::RigidBody {
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
            y: 0.0,
            z: 0.0,
        }),
        angular_velocity: Some(boxcars::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }),
    }
}

fn sample(frame_number: usize, time: f32, z: f32, vx: f32) -> StatsSample {
    StatsSample {
        frame_number,
        time,
        dt: 1.0,
        seconds_remaining: None,
        game_state: None,
        ball_has_been_hit: None,
        kickoff_countdown_time: None,
        team_zero_score: None,
        team_one_score: None,
        possession_team_is_team_0: Some(true),
        scored_on_team_is_team_0: None,
        current_in_game_team_player_counts: Some([1, 1]),
        ball: None,
        players: vec![PlayerSample {
            player_id: RemoteId::Steam(1),
            is_team_0: true,
            rigid_body: Some(rigid_body(frame_number as f32, 0.0, z, vx)),
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
        }],
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
fn movement_reducer_tracks_labeled_time_bands() {
    let mut reducer = MovementReducer::new();

    reducer.on_sample(&sample(0, 0.0, 0.0, 200.0)).unwrap();
    reducer.on_sample(&sample(1, 1.0, 0.0, 200.0)).unwrap();
    reducer.on_sample(&sample(2, 2.0, 300.0, 1600.0)).unwrap();
    reducer.on_sample(&sample(3, 3.0, 900.0, 2400.0)).unwrap();

    let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
    assert_eq!(stats.tracked_time, 4.0);
    assert_eq!(
        stats.tracked_time_with_labels(&[StatLabel::new("speed_band", "slow")]),
        2.0
    );
    assert_eq!(
        stats.tracked_time_with_labels(&[StatLabel::new("height_band", "ground")]),
        2.0
    );
    assert_eq!(
        stats.tracked_time_with_labels(&[
            StatLabel::new("speed_band", "boost"),
            StatLabel::new("height_band", "low_air"),
        ]),
        1.0
    );
    assert_eq!(
        stats.tracked_time_with_labels(&[
            StatLabel::new("speed_band", "supersonic"),
            StatLabel::new("height_band", "high_air"),
        ]),
        1.0
    );
}

#[test]
fn movement_stats_complete_labeled_time_adds_zero_entries() {
    let mut stats = MovementStats::default();
    stats.labeled_tracked_time.add(
        [
            StatLabel::new("speed_band", "boost"),
            StatLabel::new("height_band", "low_air"),
        ],
        1.25,
    );

    let completed = stats.complete_labeled_tracked_time();

    assert_eq!(completed.entries.len(), 9);
    assert_eq!(
        completed.sum_exact(&[
            StatLabel::new("speed_band", "boost"),
            StatLabel::new("height_band", "low_air"),
        ]),
        1.25
    );
    assert_eq!(
        completed.sum_exact(&[
            StatLabel::new("speed_band", "slow"),
            StatLabel::new("height_band", "ground"),
        ]),
        0.0
    );
}
