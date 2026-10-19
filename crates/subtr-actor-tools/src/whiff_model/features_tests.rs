use super::*;

fn player(is_team_0: bool, frames: usize) -> NumericPlayer {
    NumericPlayer {
        id: format!("p{is_team_0}"),
        is_team_0,
        pos: vec![[0.0, 0.0, 17.0]; frames],
        vel: vec![[1000.0, 0.0, 0.0]; frames],
        ang_vel: vec![[0.0; 3]; frames],
        forward: vec![[1.0, 0.0, 0.0]; frames],
        up: vec![[0.0, 0.0, 1.0]; frames],
        boost: vec![50.0; frames],
        boost_active: vec![false; frames],
        dodge_active: vec![false; frames],
        double_jump_active: vec![false; frames],
        contact_gap: vec![900.0; frames],
        local_ball: vec![[500.0, 0.0, 0.0]; frames],
    }
}

/// 6 s at 30 Hz with a single close approach by the first player at t = 3 s.
fn approach() -> NumericTelemetry {
    let frames = 180;
    let time: Vec<f64> = (0..frames).map(|i| i as f64 / 30.0).collect();
    let mut focal = player(true, frames);
    for (i, t) in time.iter().enumerate() {
        focal.contact_gap[i] = 40.0 + 600.0 * (t - 3.0).abs();
    }
    NumericTelemetry {
        time,
        live: vec![true; frames],
        ball_pos: vec![[600.0, 0.0, 93.0]; frames],
        ball_vel: vec![[0.0; 3]; frames],
        players: vec![focal, player(false, frames)],
        touches: Vec::new(),
    }
}

#[test]
fn candidate_is_the_gap_minimum_during_live_play() {
    let mut r = approach();
    assert_eq!(candidate_indices(&r, 0), vec![90]);
    assert!(candidate_indices(&r, 1).is_empty());
    r.live[90] = false;
    assert_ne!(candidate_indices(&r, 0), vec![90]);
}

#[test]
fn features_ignore_everything_after_the_resolution_delay() {
    let r = approach();
    let baseline = candidate_features(&r, 0, 90);
    let mut later = approach();
    let cutoff = r.time[90] + RESOLUTION_DELAY_SECONDS;
    for i in 0..later.time.len() {
        if later.time[i] > cutoff + 1e-9 {
            later.ball_vel[i] = [3000.0, 1000.0, 500.0];
            later.players[0].contact_gap[i] = 0.0;
            later.players[1].contact_gap[i] = 0.0;
            later.players[0].dodge_active[i] = true;
        }
    }
    later.touches.push((1, 130));
    let after = candidate_features(&later, 0, 90);
    for (name, (a, b)) in FEATURE_NAMES.iter().zip(baseline.iter().zip(&after)) {
        assert!(
            a == b || (a.is_nan() && b.is_nan()),
            "{name} changed: {a} vs {b}"
        );
    }
}

#[test]
fn touches_are_split_by_focal_teammate_and_opponent() {
    let mut r = approach();
    r.touches = vec![(0, 92), (1, 85)];
    let f = candidate_features(&r, 0, 90);
    let index = |name: &str| FEATURE_NAMES.iter().position(|n| *n == name).unwrap();
    assert!((f[index("focal_touch_dt")] - 2.0 / 30.0).abs() < 1e-9);
    assert!((f[index("opponent_touch_dt")] + 5.0 / 30.0).abs() < 1e-9);
    assert_eq!(f[index("teammate_touch_dt")], NO_TOUCH);
}
