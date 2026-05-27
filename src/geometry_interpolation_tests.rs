use super::*;
use boxcars::{Quaternion, Vector3f};

#[test]
fn interpolated_rigid_body_blends_location() {
    let start_body = boxcars::RigidBody {
        sleeping: false,
        location: Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
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
    };
    let end_body = boxcars::RigidBody {
        sleeping: true,
        location: Vector3f {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        rotation: Quaternion {
            x: 0.0,
            y: 0.0,
            z: 1.0,
            w: 0.0,
        },
        linear_velocity: Some(Vector3f {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }),
        angular_velocity: Some(Vector3f {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }),
    };

    let result = get_interpolated_rigid_body(&start_body, 0.0, &end_body, 1.0, 0.5)
        .expect("interpolation should succeed");

    assert_eq!(result.location.x, 0.5);
    assert_eq!(result.location.y, 0.5);
    assert_eq!(result.location.z, 0.5);
}
