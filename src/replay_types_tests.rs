use super::*;
use crate::glam_to_vec;

fn rigid_body(position: glam::Vec3, velocity: Option<glam::Vec3>) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: boxcars::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: velocity.map(|velocity| glam_to_vec(&velocity)),
        angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
    }
}

#[test]
fn shot_event_metadata_calculates_speed_distance_and_goal_alignment() {
    let ball = rigid_body(
        glam::Vec3::new(300.0, 1000.0, 120.0),
        Some(glam::Vec3::new(0.0, 2000.0, 0.0)),
    );
    let player = rigid_body(
        glam::Vec3::new(300.0, 850.0, 20.0),
        Some(glam::Vec3::new(0.0, 1200.0, 0.0)),
    );

    let metadata = ShotEventMetadata::from_rigid_bodies(true, &ball, Some(&player));

    assert_eq!(metadata.ball_position, ball.location);
    assert_eq!(metadata.ball_velocity, ball.linear_velocity);
    assert_eq!(metadata.ball_speed, Some(2000.0));
    assert_eq!(metadata.player_position, Some(player.location));
    assert_eq!(metadata.player_speed, Some(1200.0));
    assert!((metadata.player_distance_to_ball.unwrap() - 180.27756).abs() < 0.001);
    assert_eq!(metadata.target_goal_position.y, 5120.0);
    assert_eq!(metadata.target_goal_position.z, ball.location.z);
    assert_eq!(metadata.distance_to_goal_line, 4120.0);
    assert!(metadata.distance_to_goal_center > 4130.0);
    assert!(metadata.ball_goal_alignment.unwrap() > 0.99);
    assert!(metadata.ball_speed_toward_goal.unwrap() > 1990.0);
}

#[test]
fn shot_event_metadata_uses_orange_goal_direction() {
    let ball = rigid_body(
        glam::Vec3::new(0.0, -1000.0, 120.0),
        Some(glam::Vec3::new(0.0, -1500.0, 0.0)),
    );

    let metadata = ShotEventMetadata::from_rigid_bodies(false, &ball, None);

    assert_eq!(metadata.target_goal_position.y, -5120.0);
    assert_eq!(metadata.distance_to_goal_line, 4120.0);
    assert_eq!(metadata.player_position, None);
    assert!(metadata.ball_goal_alignment.unwrap() > 0.99);
    assert!(metadata.ball_speed_toward_goal.unwrap() > 1490.0);
}
