use super::positioning_ball_depth::ball_depth_fractions;

#[test]
fn ball_depth_fractions_treat_near_ball_band_as_level() {
    let (behind, level, in_front) = ball_depth_fractions(150.0, -100.0, 100.0);
    assert_eq!(behind, 0.0);
    assert_eq!(level, 1.0);
    assert_eq!(in_front, 0.0);
}

#[test]
fn ball_depth_fractions_split_crossing_time_across_all_three_buckets() {
    let (behind, level, in_front) = ball_depth_fractions(150.0, -300.0, 300.0);
    assert!((behind - 0.25).abs() < 1e-6);
    assert!((level - 0.5).abs() < 1e-6);
    assert!((in_front - 0.25).abs() < 1e-6);
}

#[test]
fn ball_depth_fractions_count_boundary_point_as_in_front_not_level() {
    let (behind, level, in_front) = ball_depth_fractions(150.0, 150.0, 150.0);
    assert_eq!(behind, 0.0);
    assert_eq!(level, 0.0);
    assert_eq!(in_front, 1.0);
}
