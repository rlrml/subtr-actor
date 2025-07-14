use super::*;
use boxcars::Quaternion;
use boxcars::Vector3f;

#[test]
fn test_get_interpolated_rigid_body() {
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
    let start_time = 0.0f32;
    let end_time = 1.0f32;
    let time = 0.5f32;

    let result = get_interpolated_rigid_body(&start_body, start_time, &end_body, end_time, time);

    match result {
        Ok(interpolated_body) => {
            assert_eq!(interpolated_body.location.x, 0.5);
            assert_eq!(interpolated_body.location.y, 0.5);
            assert_eq!(interpolated_body.location.z, 0.5);
            // Add further assertions for rotation, linear_velocity and angular_velocity as needed.
        }
        Err(e) => panic!("Interpolation failed: {e:?}"),
    };
}

#[test]
fn test_find_update_in_direction() {
    let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let current_index = 4; // Starting search from number 5
    let predicate = |&x: &i32| if x % 2 == 0 { Some(x) } else { None }; // Looking for the first even number

    // Test forward search.
    let result_forward =
        util::find_in_direction(&items, current_index, SearchDirection::Forward, predicate);
    // Check that the result is as expected.
    assert_eq!(result_forward, Some((5, 6))); // First even number after index 4 is 6 at index 5

    // Test backward search.
    let result_backward =
        util::find_in_direction(&items, current_index, SearchDirection::Backward, predicate);
    // Check that the result is as expected.
    assert_eq!(result_backward, Some((3, 4))); // First even number before index 4 is 4 at index 3
}
