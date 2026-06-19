use super::*;
use boxcars::Quaternion;

fn rigid_body(
    location: glam::Vec3,
    linear_velocity: glam::Vec3,
    angular_velocity: glam::Vec3,
) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&location),
        rotation: Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(glam_to_vec(&linear_velocity)),
        angular_velocity: Some(glam_to_vec(&angular_velocity)),
    }
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.001,
        "expected {actual} to be within 0.001 of {expected}"
    );
}

#[test]
fn semi_implicit_default_applies_standard_ball_gravity_at_120_hz() {
    let initial = rigid_body(glam::Vec3::ZERO, glam::Vec3::ZERO, glam::Vec3::ZERO);

    let predicted = advance_ball_free_flight(&initial, 1.0, BallTrajectoryConfig::STANDARD_SOCCAR);

    assert_close(
        predicted.linear_velocity.unwrap().z,
        STANDARD_BALL_GRAVITY_Z,
    );
    assert_close(predicted.location.z, -327.70834);
}

#[test]
fn closed_form_integration_uses_constant_acceleration_displacement() {
    let initial = rigid_body(glam::Vec3::ZERO, glam::Vec3::ZERO, glam::Vec3::ZERO);
    let config = BallTrajectoryConfig {
        integration: BallTrajectoryIntegration::ClosedForm,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };

    let predicted = advance_ball_free_flight(&initial, 1.0, config);

    assert_close(
        predicted.linear_velocity.unwrap().z,
        STANDARD_BALL_GRAVITY_Z,
    );
    assert_close(predicted.location.z, -325.0);
}

#[test]
fn clamps_ball_speed_before_integrating_position() {
    let initial = rigid_body(
        glam::Vec3::ZERO,
        glam::Vec3::new(7000.0, 0.0, 0.0),
        glam::Vec3::ZERO,
    );
    let config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        max_speed: STANDARD_BALL_MAX_SPEED,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };

    let predicted =
        advance_ball_free_flight(&initial, 1.0 / ROCKET_LEAGUE_PHYSICS_TICK_RATE_HZ, config);

    assert_close(
        predicted.linear_velocity.unwrap().x,
        STANDARD_BALL_MAX_SPEED,
    );
    assert_close(
        predicted.location.x,
        STANDARD_BALL_MAX_SPEED / ROCKET_LEAGUE_PHYSICS_TICK_RATE_HZ,
    );
}

#[test]
fn angular_velocity_rotates_ball_but_does_not_curve_free_flight_path() {
    let no_spin = rigid_body(
        glam::Vec3::new(10.0, 20.0, 500.0),
        glam::Vec3::new(1000.0, -500.0, 750.0),
        glam::Vec3::ZERO,
    );
    let with_spin = rigid_body(
        glam::Vec3::new(10.0, 20.0, 500.0),
        glam::Vec3::new(1000.0, -500.0, 750.0),
        glam::Vec3::new(0.0, 0.0, 6.0),
    );

    let predicted_no_spin =
        advance_ball_free_flight(&no_spin, 0.5, BallTrajectoryConfig::STANDARD_SOCCAR);
    let predicted_with_spin =
        advance_ball_free_flight(&with_spin, 0.5, BallTrajectoryConfig::STANDARD_SOCCAR);

    assert_eq!(predicted_no_spin.location, predicted_with_spin.location);
    assert_eq!(
        predicted_no_spin.linear_velocity,
        predicted_with_spin.linear_velocity
    );
    assert_ne!(predicted_no_spin.rotation, predicted_with_spin.rotation);
}

#[test]
fn trajectory_samples_include_initial_and_requested_endpoint() {
    let initial = rigid_body(glam::Vec3::ZERO, glam::Vec3::X * 120.0, glam::Vec3::ZERO);
    let config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };

    let samples = predict_ball_free_flight_trajectory(&initial, 0.25, 0.1, config);

    assert_eq!(samples.len(), 4);
    assert_close(samples[0].time, 0.0);
    assert_close(samples[1].time, 0.1);
    assert_close(samples[2].time, 0.2);
    assert_close(samples[3].time, 0.25);
    assert_close(samples[3].rigid_body.location.x, 30.0);
}

#[test]
fn prediction_error_is_zero_for_matching_free_flight_samples() {
    let initial = rigid_body(
        glam::Vec3::new(-100.0, 200.0, 700.0),
        glam::Vec3::new(500.0, -400.0, 1000.0),
        glam::Vec3::ZERO,
    );
    let samples = predict_ball_free_flight_trajectory(
        &initial,
        0.5,
        1.0 / ROCKET_LEAGUE_PHYSICS_TICK_RATE_HZ,
        BallTrajectoryConfig::STANDARD_SOCCAR,
    );
    let observed: Vec<(f32, boxcars::RigidBody)> = samples
        .into_iter()
        .skip(1)
        .map(|sample| (sample.time, sample.rigid_body))
        .collect();

    let error = ball_free_flight_prediction_error(
        &initial,
        &observed,
        BallTrajectoryConfig::STANDARD_SOCCAR,
    )
    .expect("observed samples should produce an error summary");

    assert_eq!(error.sample_count, observed.len());
    assert_close(error.max_position_error, 0.0);
    assert_close(error.rms_position_error, 0.0);
    assert_close(error.max_velocity_error.unwrap(), 0.0);
    assert_close(error.rms_velocity_error.unwrap(), 0.0);
}

#[test]
fn pure_normal_bounce_applies_restitution() {
    let initial = rigid_body(
        glam::Vec3::new(0.0, 0.0, STANDARD_BALL_RADIUS),
        glam::Vec3::new(0.0, 0.0, -1000.0),
        glam::Vec3::ZERO,
    );

    let bounced = bounce_ball_off_surface(
        &initial,
        glam::Vec3::Z,
        BallBounceConfig::STANDARD_SOCCAR,
        BallTrajectoryConfig::STANDARD_SOCCAR,
    );

    assert_close(bounced.linear_velocity.unwrap().z, 600.0);
    assert_close(
        vec_to_glam(&bounced.angular_velocity.unwrap()).length(),
        0.0,
    );
}

#[test]
fn bounce_tangential_impulse_updates_velocity_and_spin() {
    let initial = rigid_body(
        glam::Vec3::new(0.0, 0.0, STANDARD_BALL_RADIUS),
        glam::Vec3::new(500.0, 0.0, -1000.0),
        glam::Vec3::ZERO,
    );

    let bounced = bounce_ball_off_surface(
        &initial,
        glam::Vec3::Z,
        BallBounceConfig::STANDARD_SOCCAR,
        BallTrajectoryConfig::STANDARD_SOCCAR,
    );

    let velocity = bounced.linear_velocity.unwrap();
    let angular_velocity = bounced.angular_velocity.unwrap();
    assert_close(velocity.x, 357.5);
    assert_close(velocity.z, 600.0);
    assert_close(angular_velocity.y, 3.901875);
}

#[test]
fn plane_bounce_keeps_ball_above_ground() {
    let initial = rigid_body(
        glam::Vec3::new(0.0, 0.0, STANDARD_BALL_RADIUS + 0.5),
        glam::Vec3::new(0.0, 0.0, -120.0),
        glam::Vec3::ZERO,
    );
    let config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };

    let predicted = advance_ball_with_plane_bounces(
        &initial,
        1.0 / ROCKET_LEAGUE_PHYSICS_TICK_RATE_HZ,
        config,
        BallBounceConfig::STANDARD_SOCCAR,
        &[BallCollisionPlane::standard_ground()],
    );

    assert_close(predicted.location.z, STANDARD_BALL_RADIUS + 0.3);
    assert_close(predicted.linear_velocity.unwrap().z, 72.0);
}

#[test]
fn sampled_plane_bounce_trajectory_can_continue_after_impact() {
    let initial = rigid_body(
        glam::Vec3::new(0.0, 0.0, STANDARD_BALL_RADIUS + 0.5),
        glam::Vec3::new(0.0, 0.0, -120.0),
        glam::Vec3::ZERO,
    );
    let config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };

    let samples = predict_ball_with_plane_bounces_trajectory(
        &initial,
        2.0 / ROCKET_LEAGUE_PHYSICS_TICK_RATE_HZ,
        1.0 / ROCKET_LEAGUE_PHYSICS_TICK_RATE_HZ,
        config,
        BallBounceConfig::STANDARD_SOCCAR,
        &[BallCollisionPlane::standard_ground()],
    );

    assert_eq!(samples.len(), 3);
    assert_close(samples[1].rigid_body.location.z, STANDARD_BALL_RADIUS + 0.3);
    assert_close(samples[1].rigid_body.linear_velocity.unwrap().z, 72.0);
    assert!(samples[2].rigid_body.location.z > samples[1].rigid_body.location.z);
}

#[test]
fn rolling_ground_contact_can_reach_goal_line() {
    let initial = rigid_body(
        glam::Vec3::new(0.0, 0.0, STANDARD_BALL_RADIUS),
        glam::Vec3::new(0.0, 1000.0, 0.0),
        glam::Vec3::ZERO,
    );

    let crossing = predict_ball_with_plane_bounces_goal_line_crossing(
        &initial,
        BallGoalLineCrossingConfig::team_zero_attacking_goal(),
        BallTrajectoryConfig::STANDARD_SOCCAR,
        BallBounceConfig::STANDARD_SOCCAR,
        &[BallCollisionPlane::standard_ground()],
    )
    .expect("rolling ball should stay on the ground and cross the goal line");

    assert_close(crossing.time, STANDARD_GOAL_LINE_Y / 1000.0);
    assert_close(crossing.position.y, STANDARD_GOAL_LINE_Y);
    assert_close(crossing.position.z, STANDARD_BALL_RADIUS);
    assert!(crossing.inside_goal_mouth);
}

#[test]
fn predicts_free_flight_goal_line_crossing_location() {
    let initial = rigid_body(
        glam::Vec3::new(100.0, 0.0, 200.0),
        glam::Vec3::new(0.0, 1024.0, 0.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };

    let crossing = predict_free_flight_goal_line_crossing(
        &initial,
        BallGoalLineCrossingConfig::team_zero_attacking_goal(),
        trajectory_config,
    )
    .expect("ball should cross the positive goal line");

    assert_close(crossing.time, 5.0);
    assert_close(crossing.position.x, 100.0);
    assert_close(crossing.position.y, STANDARD_GOAL_LINE_Y);
    assert_close(crossing.position.z, 200.0);
    assert!(crossing.inside_goal_mouth);
}

#[test]
fn goal_line_crossing_requires_initial_ball_velocity() {
    let mut initial = rigid_body(
        glam::Vec3::new(100.0, 0.0, 200.0),
        glam::Vec3::new(0.0, 1024.0, 0.0),
        glam::Vec3::ZERO,
    );
    initial.linear_velocity = None;
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };

    let crossing = predict_free_flight_goal_line_crossing(
        &initial,
        BallGoalLineCrossingConfig::team_zero_attacking_goal(),
        trajectory_config,
    );

    assert!(crossing.is_none());
}

#[test]
fn goal_line_crossing_rejects_ball_moving_away_from_target_goal() {
    let initial = rigid_body(
        glam::Vec3::new(0.0, 0.0, 200.0),
        glam::Vec3::new(0.0, -1024.0, 0.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };

    let crossing = predict_free_flight_goal_line_crossing(
        &initial,
        BallGoalLineCrossingConfig::team_zero_attacking_goal(),
        trajectory_config,
    );

    assert!(crossing.is_none());
}

#[test]
fn predicts_goal_line_crossing_after_ground_bounce() {
    let initial = rigid_body(
        glam::Vec3::new(
            0.0,
            STANDARD_GOAL_LINE_Y - 120.0,
            STANDARD_BALL_RADIUS + 0.5,
        ),
        glam::Vec3::new(0.0, 240.0, -120.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };

    let crossing = predict_ball_with_plane_bounces_goal_line_crossing(
        &initial,
        BallGoalLineCrossingConfig::team_zero_attacking_goal(),
        trajectory_config,
        BallBounceConfig::STANDARD_SOCCAR,
        &[BallCollisionPlane::standard_ground()],
    )
    .expect("ball should cross the goal line after bouncing");

    assert_close(crossing.position.y, STANDARD_GOAL_LINE_Y);
    assert!(crossing.position.z >= STANDARD_BALL_RADIUS);
    assert!(crossing.inside_goal_mouth);
}

#[test]
fn standard_goal_line_prediction_planes_include_side_wall_bounces() {
    let initial = rigid_body(
        glam::Vec3::new(
            STANDARD_ARENA_SIDE_WALL_X - STANDARD_BALL_RADIUS - 2.0,
            STANDARD_GOAL_LINE_Y - 240.0,
            200.0,
        ),
        glam::Vec3::new(600.0, 1200.0, 0.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };
    let planes = standard_soccar_goal_line_prediction_planes();

    let crossing = predict_ball_with_plane_bounces_goal_line_crossing(
        &initial,
        BallGoalLineCrossingConfig::team_zero_attacking_goal(),
        trajectory_config,
        BallBounceConfig::STANDARD_SOCCAR,
        &planes,
    )
    .expect("banked ball should cross the goal line");

    assert_close(crossing.position.y, STANDARD_GOAL_LINE_Y);
    assert!(crossing.position.x < initial.location.x);
    assert!(!crossing.inside_goal_mouth);
}

#[test]
fn standard_goal_line_prediction_planes_include_ceiling_bounces() {
    let initial = rigid_body(
        glam::Vec3::new(
            0.0,
            STANDARD_GOAL_LINE_Y - 240.0,
            STANDARD_ARENA_CEILING_Z - STANDARD_BALL_RADIUS - 1.0,
        ),
        glam::Vec3::new(0.0, 1200.0, 600.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };
    let planes = standard_soccar_goal_line_prediction_planes();

    let crossing = predict_ball_with_plane_bounces_goal_line_crossing(
        &initial,
        BallGoalLineCrossingConfig::team_zero_attacking_goal(),
        trajectory_config,
        BallBounceConfig::STANDARD_SOCCAR,
        &planes,
    )
    .expect("ceiling-bounced ball should cross the goal line");

    assert_close(crossing.position.y, STANDARD_GOAL_LINE_Y);
    assert!(crossing.position.z < initial.location.z);
    assert!(!crossing.inside_goal_mouth);
}

#[test]
fn goal_line_prediction_surfaces_block_near_post_shots() {
    let initial = rigid_body(
        glam::Vec3::new(
            STANDARD_GOAL_MOUTH_HALF_WIDTH_X + STANDARD_GOAL_FRAME_RADIUS + STANDARD_BALL_RADIUS
                - 5.0,
            STANDARD_GOAL_LINE_Y - 220.0,
            200.0,
        ),
        glam::Vec3::new(0.0, 1200.0, 0.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };
    let surfaces = standard_soccar_goal_line_prediction_surfaces();

    let crossing = predict_ball_with_surface_bounces_goal_line_crossing(
        &initial,
        BallGoalLineCrossingConfig::team_zero_attacking_goal(),
        trajectory_config,
        BallBounceConfig::STANDARD_SOCCAR,
        &surfaces,
    );

    assert!(crossing.is_none());
}

#[test]
fn goal_line_prediction_surfaces_block_near_crossbar_shots() {
    let initial = rigid_body(
        glam::Vec3::new(
            0.0,
            STANDARD_GOAL_LINE_Y - 220.0,
            STANDARD_GOAL_MOUTH_HEIGHT_Z + STANDARD_GOAL_FRAME_RADIUS + STANDARD_BALL_RADIUS - 5.0,
        ),
        glam::Vec3::new(0.0, 1200.0, 0.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };
    let surfaces = standard_soccar_goal_line_prediction_surfaces();

    let crossing = predict_ball_with_surface_bounces_goal_line_crossing(
        &initial,
        BallGoalLineCrossingConfig::team_zero_attacking_goal(),
        trajectory_config,
        BallBounceConfig::STANDARD_SOCCAR,
        &surfaces,
    );

    assert!(crossing.is_none());
}

#[test]
fn goal_target_hit_reports_goal_line_pass_through() {
    let initial = rigid_body(
        glam::Vec3::new(0.0, STANDARD_GOAL_LINE_Y - 220.0, 200.0),
        glam::Vec3::new(0.0, 1200.0, 0.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };
    let surfaces = standard_soccar_goal_target_prediction_surfaces();

    let hit = predict_ball_with_surface_bounces_goal_target_hit(
        &initial,
        BallGoalLineCrossingConfig::team_zero_attacking_goal(),
        trajectory_config,
        BallBounceConfig::STANDARD_SOCCAR,
        &surfaces,
    )
    .expect("goal-bound shot should pass through the goal line");

    assert_eq!(hit.hit_kind, BallGoalTargetHitKind::GoalLine);
    assert_close(hit.position.x, 0.0);
    assert_close(hit.position.y, STANDARD_GOAL_LINE_Y);
    assert_close(hit.position.z, 200.0);
}

#[test]
fn goal_target_hit_reports_back_wall_contact_outside_goal_mouth() {
    let initial = rigid_body(
        glam::Vec3::new(
            STANDARD_GOAL_MOUTH_HALF_WIDTH_X
                + STANDARD_GOAL_FRAME_RADIUS
                + STANDARD_BALL_RADIUS * 2.0
                + 10.0,
            STANDARD_GOAL_LINE_Y - 220.0,
            200.0,
        ),
        glam::Vec3::new(0.0, 1200.0, 0.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };
    let surfaces = standard_soccar_goal_target_prediction_surfaces();

    let hit = predict_ball_with_surface_bounces_goal_target_hit(
        &initial,
        BallGoalLineCrossingConfig::team_zero_attacking_goal(),
        trajectory_config,
        BallBounceConfig::STANDARD_SOCCAR,
        &surfaces,
    )
    .expect("wide shot should hit the attacking back wall");

    assert_eq!(hit.hit_kind, BallGoalTargetHitKind::BackWall);
    assert_close(hit.position.x, initial.location.x);
    assert_close(hit.position.y, STANDARD_GOAL_LINE_Y);
    assert_close(hit.position.z, initial.location.z);
}

#[test]
fn goal_target_hit_reports_goal_frame_contact() {
    let initial = rigid_body(
        glam::Vec3::new(
            STANDARD_GOAL_MOUTH_HALF_WIDTH_X + STANDARD_GOAL_FRAME_RADIUS + STANDARD_BALL_RADIUS
                - 5.0,
            STANDARD_GOAL_LINE_Y - 220.0,
            200.0,
        ),
        glam::Vec3::new(0.0, 1200.0, 0.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };
    let surfaces = standard_soccar_goal_target_prediction_surfaces();

    let hit = predict_ball_with_surface_bounces_goal_target_hit(
        &initial,
        BallGoalLineCrossingConfig::team_zero_attacking_goal(),
        trajectory_config,
        BallBounceConfig::STANDARD_SOCCAR,
        &surfaces,
    )
    .expect("near-post shot should hit the goal frame");

    assert_eq!(hit.hit_kind, BallGoalTargetHitKind::GoalFrame);
    assert!(hit.position.x > STANDARD_GOAL_MOUTH_HALF_WIDTH_X);
    assert!(hit.position.y <= STANDARD_GOAL_LINE_Y);
}

#[test]
fn surface_bounce_reflects_off_goal_post_cylinder() {
    let initial = rigid_body(
        glam::Vec3::new(
            STANDARD_GOAL_MOUTH_HALF_WIDTH_X + STANDARD_GOAL_FRAME_RADIUS + STANDARD_BALL_RADIUS
                - 5.0,
            STANDARD_GOAL_LINE_Y - 220.0,
            200.0,
        ),
        glam::Vec3::new(0.0, 1200.0, 0.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };
    let surfaces = standard_soccar_goal_frame_surfaces();

    let predicted = advance_ball_with_surface_bounces(
        &initial,
        0.25,
        trajectory_config,
        BallBounceConfig::STANDARD_SOCCAR,
        &surfaces,
    );

    assert!(predicted.linear_velocity.unwrap().y < 0.0);
    assert!(predicted.location.y < STANDARD_GOAL_LINE_Y);
}

#[test]
fn concave_cylinder_surface_can_model_low_side_wall_ramp_bounces() {
    let initial = rigid_body(
        glam::Vec3::new(
            STANDARD_ARENA_SIDE_WALL_X - 180.0,
            0.0,
            STANDARD_BALL_RADIUS,
        ),
        glam::Vec3::new(3200.0, 0.0, -50.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };
    let surfaces = [
        BallCollisionSurface::ConcaveCylinder(
            BallCollisionConcaveCylinder::standard_positive_x_wall_bottom_ramp(),
        ),
        BallCollisionSurface::Plane(BallCollisionPlane::standard_positive_x_wall_bounded()),
        BallCollisionSurface::Plane(BallCollisionPlane::standard_ground_bounded()),
    ];

    let predicted = advance_ball_with_surface_bounces(
        &initial,
        0.1,
        trajectory_config,
        BallBounceConfig::STANDARD_SOCCAR,
        &surfaces,
    );

    assert!(predicted.location.x < STANDARD_ARENA_SIDE_WALL_X - STANDARD_BALL_RADIUS);
    assert!(predicted.location.z > initial.location.z, "{predicted:?}");
    assert!(predicted.linear_velocity.unwrap().x < 0.0, "{predicted:?}");
    assert!(predicted.linear_velocity.unwrap().z > 0.0, "{predicted:?}");
}

#[test]
fn standard_prediction_planes_let_ball_pass_through_goal_mouth() {
    let initial = rigid_body(
        glam::Vec3::new(
            0.0,
            STANDARD_ARENA_BACK_WALL_Y - STANDARD_BALL_RADIUS - 1.0,
            200.0,
        ),
        glam::Vec3::new(0.0, 600.0, 0.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };
    let planes = standard_soccar_prediction_planes();

    let predicted = advance_ball_with_plane_bounces(
        &initial,
        0.25,
        trajectory_config,
        BallBounceConfig::STANDARD_SOCCAR,
        &planes,
    );

    assert!(predicted.location.y > STANDARD_ARENA_BACK_WALL_Y);
    assert!(predicted.linear_velocity.unwrap().y > 0.0);
}

#[test]
fn standard_prediction_surfaces_let_ball_pass_through_goal_mouth_bottom_ramp_gap() {
    let initial = rigid_body(
        glam::Vec3::new(
            0.0,
            STANDARD_ARENA_BACK_WALL_Y - STANDARD_BALL_RADIUS - 1.0,
            STANDARD_BALL_RADIUS,
        ),
        glam::Vec3::new(0.0, 600.0, 0.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };
    let surfaces = standard_soccar_prediction_surfaces();

    let predicted = advance_ball_with_surface_bounces(
        &initial,
        0.25,
        trajectory_config,
        BallBounceConfig::STANDARD_SOCCAR,
        &surfaces,
    );

    assert!(predicted.location.y > STANDARD_ARENA_BACK_WALL_Y);
    assert!(predicted.linear_velocity.unwrap().y > 0.0);
}

#[test]
fn standard_prediction_planes_include_back_wall_bounces_outside_goal_mouth() {
    let initial = rigid_body(
        glam::Vec3::new(
            STANDARD_GOAL_MOUTH_HALF_WIDTH_X + STANDARD_BALL_RADIUS + 5.0,
            STANDARD_ARENA_BACK_WALL_Y - STANDARD_BALL_RADIUS - 1.0,
            200.0,
        ),
        glam::Vec3::new(0.0, 600.0, 0.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };
    let planes = standard_soccar_prediction_planes();

    let predicted = advance_ball_with_plane_bounces(
        &initial,
        1.0 / ROCKET_LEAGUE_PHYSICS_TICK_RATE_HZ,
        trajectory_config,
        BallBounceConfig::STANDARD_SOCCAR,
        &planes,
    );

    assert!(predicted.location.y < initial.location.y);
    assert!(predicted.linear_velocity.unwrap().y < 0.0);
}

#[test]
fn standard_prediction_surfaces_do_not_reflect_ball_moving_out_of_positive_goal_mouth() {
    let initial = rigid_body(
        glam::Vec3::new(-186.0, 4736.0, 424.0),
        glam::Vec3::new(-897.0, -209.0, 139.0),
        glam::Vec3::ZERO,
    );
    let surfaces = standard_soccar_prediction_surfaces();

    let predicted = advance_ball_with_surface_bounces(
        &initial,
        0.201,
        BallTrajectoryConfig::STANDARD_SOCCAR,
        BallBounceConfig::STANDARD_SOCCAR,
        &surfaces,
    );

    assert!(predicted.location.y > 4500.0, "{predicted:?}");
    assert!(predicted.linear_velocity.unwrap().y < 0.0, "{predicted:?}");
}

#[test]
fn bounded_collision_plane_ignores_out_of_bounds_impacts() {
    let initial = rigid_body(
        glam::Vec3::new(0.0, 0.0, STANDARD_BALL_RADIUS + 0.5),
        glam::Vec3::new(0.0, 0.0, -120.0),
        glam::Vec3::ZERO,
    );
    let config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };
    let bounded_ground =
        BallCollisionPlane::standard_ground().with_bounds(BallCollisionPlaneBounds::new(
            glam::Vec3::new(1000.0, 1000.0, 0.0),
            glam::Vec3::new(2000.0, 2000.0, 1000.0),
        ));

    let predicted = advance_ball_with_plane_bounces(
        &initial,
        1.0 / ROCKET_LEAGUE_PHYSICS_TICK_RATE_HZ,
        config,
        BallBounceConfig::STANDARD_SOCCAR,
        &[bounded_ground],
    );

    assert!(predicted.location.z < STANDARD_BALL_RADIUS);
    assert!(predicted.linear_velocity.unwrap().z < 0.0);
}

#[test]
fn prediction_error_reports_position_and_velocity_residuals() {
    let initial = rigid_body(glam::Vec3::ZERO, glam::Vec3::ZERO, glam::Vec3::ZERO);
    let observed = [(
        1.0,
        rigid_body(
            glam::Vec3::new(3.0, 4.0, -327.70834),
            glam::Vec3::new(0.0, 12.0, STANDARD_BALL_GRAVITY_Z),
            glam::Vec3::ZERO,
        ),
    )];

    let error = ball_free_flight_prediction_error(
        &initial,
        &observed,
        BallTrajectoryConfig::STANDARD_SOCCAR,
    )
    .expect("observed samples should produce an error summary");

    assert_close(error.max_position_error, 5.0);
    assert_close(error.rms_position_error, 5.0);
    assert_close(error.max_velocity_error.unwrap(), 12.0);
    assert_close(error.rms_velocity_error.unwrap(), 12.0);
}

#[test]
fn plane_bounce_prediction_error_is_zero_for_matching_bounce_samples() {
    let initial = rigid_body(
        glam::Vec3::new(0.0, 0.0, STANDARD_BALL_RADIUS + 0.5),
        glam::Vec3::new(0.0, 0.0, -120.0),
        glam::Vec3::ZERO,
    );
    let trajectory_config = BallTrajectoryConfig {
        gravity: glam::Vec3::ZERO,
        ..BallTrajectoryConfig::STANDARD_SOCCAR
    };
    let samples = predict_ball_with_plane_bounces_trajectory(
        &initial,
        2.0 / ROCKET_LEAGUE_PHYSICS_TICK_RATE_HZ,
        1.0 / ROCKET_LEAGUE_PHYSICS_TICK_RATE_HZ,
        trajectory_config,
        BallBounceConfig::STANDARD_SOCCAR,
        &[BallCollisionPlane::standard_ground()],
    );
    let observed = samples
        .into_iter()
        .skip(1)
        .map(|sample| (sample.time, sample.rigid_body))
        .collect::<Vec<_>>();

    let error = ball_plane_bounce_prediction_error(
        &initial,
        &observed,
        trajectory_config,
        BallBounceConfig::STANDARD_SOCCAR,
        &[BallCollisionPlane::standard_ground()],
    )
    .expect("observed samples should produce an error summary");

    assert_eq!(error.sample_count, observed.len());
    assert_close(error.max_position_error, 0.0);
    assert_close(error.rms_position_error, 0.0);
    assert_close(error.max_velocity_error.unwrap(), 0.0);
    assert_close(error.rms_velocity_error.unwrap(), 0.0);
}

#[test]
fn keeps_missing_linear_velocity_as_stationary_initial_velocity() {
    let mut initial = rigid_body(glam::Vec3::ZERO, glam::Vec3::ZERO, glam::Vec3::ZERO);
    initial.linear_velocity = None;

    let predicted = advance_ball_free_flight(&initial, 1.0, BallTrajectoryConfig::STANDARD_SOCCAR);

    let predicted_velocity = predicted.linear_velocity.unwrap();
    assert_close(predicted_velocity.x, 0.0);
    assert_close(predicted_velocity.y, 0.0);
    assert_close(predicted_velocity.z, STANDARD_BALL_GRAVITY_Z);
    assert_close(predicted.location.z, -327.70834);
}
