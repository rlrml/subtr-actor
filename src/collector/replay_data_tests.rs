use super::*;

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

fn shot_metadata(is_team_0: bool) -> ShotEventMetadata {
    let target_y = if is_team_0 { 1200.0 } else { -1200.0 };
    let ball = rigid_body(
        glam::Vec3::new(10.0, target_y, 120.0),
        Some(glam::Vec3::new(0.0, 1800.0, 0.0)),
    );
    ShotEventMetadata::from_rigid_bodies(is_team_0, &ball, None)
}

fn player_stat_event(
    kind: PlayerStatEventKind,
    player: u64,
    is_team_0: bool,
    frame: usize,
) -> PlayerStatEvent {
    PlayerStatEvent {
        time: frame as f32 * 0.1,
        frame,
        player: boxcars::RemoteId::Steam(player),
        player_position: Some(glam_to_vec(&glam::Vec3::new(player as f32, 0.0, 17.0))),
        is_team_0,
        kind,
        shot: (kind == PlayerStatEventKind::Shot).then(|| shot_metadata(is_team_0)),
    }
}

fn touch_event(player: u64, is_team_0: bool, frame: usize, time: f32) -> TouchEvent {
    TouchEvent {
        touch_id: Some(frame as u64),
        time,
        frame,
        team_is_team_0: is_team_0,
        player: Some(boxcars::RemoteId::Steam(player)),
        player_position: None,
        closest_approach_distance: Some(0.0),
        dodge_contact: false,
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
    }
}

fn frame_data_with_ball_frames(ball_frames: Vec<(f32, boxcars::RigidBody)>) -> FrameData {
    let mut frame_data = FrameData::new();
    for (frame_index, (time, ball_body)) in ball_frames.into_iter().enumerate() {
        frame_data
            .ball_data
            .add_frame(frame_index, BallFrame::new_from_rigid_body(ball_body));
        frame_data
            .metadata_frames
            .push(MetadataFrame::new(time, 0, 0, 0));
    }
    frame_data
}

#[test]
fn missing_metadata_i32_defaults_to_zero() {
    let missing_seconds = SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
        property: SECONDS_REMAINING_KEY,
    });

    assert_eq!(metadata_i32_or_default(Err(missing_seconds)), 0);
}

#[test]
fn present_metadata_i32_is_preserved() {
    assert_eq!(metadata_i32_or_default(Ok(42)), 42);
}

#[test]
fn player_stat_events_with_shot_saves_links_opponent_save_to_latest_pending_shot() {
    let first_shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 10);
    let second_shot = player_stat_event(PlayerStatEventKind::Shot, 2, true, 11);
    let save = player_stat_event(PlayerStatEventKind::Save, 3, false, 12);

    let events = player_stat_events_with_shot_saves(&[first_shot, second_shot, save.clone()]);

    assert!(events[0].shot.as_ref().unwrap().resulting_save.is_none());
    let resulting_save = events[1]
        .shot
        .as_ref()
        .unwrap()
        .resulting_save
        .as_ref()
        .unwrap();
    assert_eq!(resulting_save.player, save.player);
    assert_eq!(resulting_save.frame, save.frame);
    assert_eq!(resulting_save.is_team_0, save.is_team_0);
    assert!(
        events[1]
            .shot
            .as_ref()
            .unwrap()
            .projected_goal_line_crossing
            .is_some()
    );
    assert!(events[2].shot.is_none());
}

#[test]
fn player_stat_events_with_shot_saves_ignores_same_team_save() {
    let shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 10);
    let same_team_save = player_stat_event(PlayerStatEventKind::Save, 2, true, 11);

    let events = player_stat_events_with_shot_saves(&[shot, same_team_save]);

    assert!(events[0].shot.as_ref().unwrap().resulting_save.is_none());
}

#[test]
fn player_stat_events_with_shot_saves_preserves_goal_bound_crossing_without_save() {
    let mut goal_shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 10);
    goal_shot.shot = Some(ShotEventMetadata::from_rigid_bodies(
        true,
        &rigid_body(
            glam::Vec3::new(0.0, STANDARD_GOAL_LINE_Y - 300.0, 300.0),
            Some(glam::Vec3::new(0.0, 2400.0, -200.0)),
        ),
        None,
    ));

    let events = player_stat_events_with_shot_saves(&[goal_shot]);
    let shot = events[0].shot.as_ref().unwrap();
    let crossing = shot
        .projected_goal_line_crossing
        .as_ref()
        .expect("goal-bound shot should keep its goal-line crossing projection");

    assert!(shot.resulting_save.is_none());
    assert_eq!(crossing.position.y, STANDARD_GOAL_LINE_Y);
    assert!(crossing.inside_goal_mouth);
    assert!(crossing.time_after_shot > 0.0);
}

#[test]
fn player_stat_events_with_shot_saves_ignores_stale_save() {
    let shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 10);
    let stale_save = player_stat_event(PlayerStatEventKind::Save, 2, false, 50);

    let events = player_stat_events_with_shot_saves(&[shot, stale_save]);

    assert!(events[0].shot.as_ref().unwrap().resulting_save.is_none());
}

#[test]
fn player_stat_events_with_shot_saves_ignores_save_after_projected_crossing() {
    let mut shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 10);
    let ball = rigid_body(
        glam::Vec3::new(0.0, 0.0, 1000.0),
        Some(glam::Vec3::new(0.0, 6000.0, 0.0)),
    );
    shot.shot = Some(ShotEventMetadata::from_rigid_bodies(true, &ball, None));
    let late_save = player_stat_event(PlayerStatEventKind::Save, 2, false, 20);

    let events = player_stat_events_with_shot_saves(&[shot, late_save]);

    assert!(events[0].shot.as_ref().unwrap().resulting_save.is_none());
}

#[test]
fn player_stat_events_with_shot_saves_links_when_save_touch_precedes_projected_crossing() {
    let mut shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 10);
    shot.time = 1.0;
    shot.frame = 10;
    let ball = rigid_body(
        glam::Vec3::new(0.0, 0.0, 1000.0),
        Some(glam::Vec3::new(0.0, 6000.0, 0.0)),
    );
    shot.shot = Some(ShotEventMetadata::from_rigid_bodies(true, &ball, None));

    let mut late_save_stat = player_stat_event(PlayerStatEventKind::Save, 2, false, 20);
    late_save_stat.time = 2.0;
    late_save_stat.frame = 20;
    let touches = [touch_event(2, false, 12, 1.2)];

    let events = player_stat_events_with_shot_saves_and_frame_data(
        &[shot, late_save_stat],
        None,
        Some(&touches),
    );

    assert!(events[0].shot.as_ref().unwrap().resulting_save.is_some());
}

#[test]
fn player_stat_events_with_shot_saves_rejects_late_save_stat_without_touch_evidence() {
    let mut shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 10);
    shot.time = 1.0;
    shot.frame = 10;
    let ball = rigid_body(
        glam::Vec3::new(0.0, 0.0, 1000.0),
        Some(glam::Vec3::new(0.0, 6000.0, 0.0)),
    );
    shot.shot = Some(ShotEventMetadata::from_rigid_bodies(true, &ball, None));

    let mut late_save_stat = player_stat_event(PlayerStatEventKind::Save, 2, false, 20);
    late_save_stat.time = 2.0;
    late_save_stat.frame = 20;

    let events =
        player_stat_events_with_shot_saves_and_frame_data(&[shot, late_save_stat], None, None);

    assert!(events[0].shot.as_ref().unwrap().resulting_save.is_none());
}

#[test]
fn player_stat_events_with_shot_saves_estimates_missing_saved_crossing_from_pre_save_ball_frame() {
    let mut shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 2);
    let moving_away_ball = rigid_body(
        glam::Vec3::new(0.0, 5000.0, 120.0),
        Some(glam::Vec3::new(0.0, -1000.0, 0.0)),
    );
    shot.time = 0.2;
    shot.frame = 2;
    shot.shot = Some(ShotEventMetadata::from_rigid_bodies(
        true,
        &moving_away_ball,
        None,
    ));
    assert!(
        shot.shot
            .as_ref()
            .unwrap()
            .projected_goal_line_crossing
            .is_none()
    );

    let mut save = player_stat_event(PlayerStatEventKind::Save, 2, false, 3);
    save.time = 2.3;
    save.frame = 3;
    let frame_data = frame_data_with_ball_frames(vec![
        (
            0.0,
            rigid_body(
                glam::Vec3::new(0.0, 2000.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (
            0.1,
            rigid_body(
                glam::Vec3::new(0.0, 2500.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (0.2, moving_away_ball),
        (2.3, moving_away_ball),
    ]);

    let events =
        player_stat_events_with_shot_saves_and_frame_data(&[shot, save], Some(&frame_data), None);
    let crossing = events[0]
        .shot
        .as_ref()
        .unwrap()
        .projected_goal_line_crossing
        .as_ref()
        .expect("saved shot should use the latest inbound pre-save ball frame");

    assert_eq!(
        crossing.prediction_kind,
        ShotGoalLineCrossingPredictionKind::SavedShotPreSaveSurfaceBounces
    );
    let target_hit = events[0]
        .shot
        .as_ref()
        .unwrap()
        .projected_goal_target_hit
        .as_ref()
        .expect("saved-shot estimate should also populate the projected target hit");
    assert_eq!(target_hit.hit_kind, ShotGoalTargetHitKind::GoalLine);
    assert_eq!(target_hit.position.y, STANDARD_GOAL_LINE_Y);
    assert!(
        crossing.time_after_shot
            > events[0]
                .shot
                .as_ref()
                .unwrap()
                .resulting_save
                .as_ref()
                .unwrap()
                .time
                - events[0].time
    );
    assert_eq!(crossing.position.y, STANDARD_GOAL_LINE_Y);
}

#[test]
fn player_stat_events_with_shot_saves_replaces_outside_projection_with_goal_mouth_estimate() {
    let mut shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 2);
    let moving_away_ball = rigid_body(
        glam::Vec3::new(0.0, 5000.0, 120.0),
        Some(glam::Vec3::new(0.0, -1000.0, 0.0)),
    );
    shot.time = 0.2;
    shot.frame = 2;
    shot.shot = Some(ShotEventMetadata::from_rigid_bodies(
        true,
        &moving_away_ball,
        None,
    ));
    shot.shot.as_mut().unwrap().projected_goal_line_crossing = Some(ShotGoalLineCrossing {
        time_after_shot: 3.0,
        prediction_start_time: None,
        prediction_start_frame: None,
        position: glam_to_vec(&glam::Vec3::new(3000.0, STANDARD_GOAL_LINE_Y, 120.0)),
        velocity: Some(glam_to_vec(&glam::Vec3::new(0.0, 1000.0, 0.0))),
        inside_goal_mouth: false,
        prediction_kind: ShotGoalLineCrossingPredictionKind::SurfaceBounces,
    });

    let mut save = player_stat_event(PlayerStatEventKind::Save, 2, false, 3);
    save.time = 2.3;
    save.frame = 3;
    let frame_data = frame_data_with_ball_frames(vec![
        (
            0.0,
            rigid_body(
                glam::Vec3::new(0.0, 2000.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (
            0.1,
            rigid_body(
                glam::Vec3::new(0.0, 2500.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (0.2, moving_away_ball),
        (2.3, moving_away_ball),
    ]);

    let events =
        player_stat_events_with_shot_saves_and_frame_data(&[shot, save], Some(&frame_data), None);
    let crossing = events[0]
        .shot
        .as_ref()
        .unwrap()
        .projected_goal_line_crossing
        .as_ref()
        .expect("saved shot should still have a projection");

    assert_eq!(
        crossing.prediction_kind,
        ShotGoalLineCrossingPredictionKind::SavedShotPreSaveSurfaceBounces
    );
    assert!(crossing.inside_goal_mouth);
    assert_eq!(crossing.position.x, 0.0);
    assert_eq!(crossing.position.y, STANDARD_GOAL_LINE_Y);
}

#[test]
fn player_stat_events_with_shot_saves_clears_unphysical_free_flight_projection() {
    let mut shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 10);
    shot.shot.as_mut().unwrap().projected_goal_line_crossing = Some(ShotGoalLineCrossing {
        time_after_shot: 1.0,
        prediction_start_time: None,
        prediction_start_frame: None,
        position: glam_to_vec(&glam::Vec3::new(0.0, STANDARD_GOAL_LINE_Y, -100.0)),
        velocity: Some(glam_to_vec(&glam::Vec3::new(0.0, 1000.0, -500.0))),
        inside_goal_mouth: false,
        prediction_kind: ShotGoalLineCrossingPredictionKind::FreeFlight,
    });
    let save = player_stat_event(PlayerStatEventKind::Save, 2, false, 11);

    let events = player_stat_events_with_shot_saves(&[shot, save]);
    let shot = events[0].shot.as_ref().unwrap();

    assert!(shot.resulting_save.is_some());
    assert!(shot.projected_goal_line_crossing.is_none());
}

#[test]
fn player_stat_events_with_shot_saves_does_not_estimate_saved_crossing_before_shot() {
    let mut shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 2);
    let moving_away_ball = rigid_body(
        glam::Vec3::new(0.0, 5000.0, 120.0),
        Some(glam::Vec3::new(0.0, -1000.0, 0.0)),
    );
    shot.time = 0.2;
    shot.frame = 2;
    shot.shot = Some(ShotEventMetadata::from_rigid_bodies(
        true,
        &moving_away_ball,
        None,
    ));

    let mut save = player_stat_event(PlayerStatEventKind::Save, 2, false, 3);
    save.time = 0.5;
    save.frame = 3;
    let frame_data = frame_data_with_ball_frames(vec![
        (
            0.0,
            rigid_body(
                glam::Vec3::new(0.0, 5110.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (0.2, moving_away_ball),
        (0.3, moving_away_ball),
        (0.5, moving_away_ball),
    ]);

    let events =
        player_stat_events_with_shot_saves_and_frame_data(&[shot, save], Some(&frame_data), None);

    assert!(
        events[0]
            .shot
            .as_ref()
            .unwrap()
            .projected_goal_line_crossing
            .is_none()
    );
    assert!(events[0].shot.as_ref().unwrap().resulting_save.is_some());
    assert_eq!(
        events[0]
            .shot
            .as_ref()
            .unwrap()
            .projected_goal_line_crossing_unavailable_reason,
        Some(ShotGoalLineCrossingUnavailableReason::CrossingsBeforePredictionStart)
    );
}

#[test]
fn player_stat_events_with_shot_saves_allows_no_touch_save_stat_lag() {
    let mut shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 2);
    let moving_away_ball = rigid_body(
        glam::Vec3::new(0.0, 4800.0, 120.0),
        Some(glam::Vec3::new(0.0, -1000.0, 0.0)),
    );
    shot.time = 0.2;
    shot.frame = 2;
    shot.shot = Some(ShotEventMetadata::from_rigid_bodies(
        true,
        &moving_away_ball,
        None,
    ));

    let mut delayed_save_stat = player_stat_event(PlayerStatEventKind::Save, 2, false, 10);
    delayed_save_stat.time = 1.0;
    delayed_save_stat.frame = 10;
    let frame_data = frame_data_with_ball_frames(vec![
        (0.0, moving_away_ball),
        (0.1, moving_away_ball),
        (0.2, moving_away_ball),
        (
            0.3,
            rigid_body(
                glam::Vec3::new(0.0, 5000.0, 120.0),
                Some(glam::Vec3::new(0.0, 800.0, 0.0)),
            ),
        ),
        (1.0, moving_away_ball),
    ]);

    let events = player_stat_events_with_shot_saves_and_frame_data(
        &[shot, delayed_save_stat],
        Some(&frame_data),
        None,
    );
    let crossing = events[0]
        .shot
        .as_ref()
        .unwrap()
        .projected_goal_line_crossing
        .as_ref()
        .expect("no-touch save stat lag should not suppress a valid pre-save estimate");

    assert!(crossing.time_after_shot < 1.0 - events[0].time);
    assert!(crossing.inside_goal_mouth);
    assert_eq!(
        crossing.prediction_kind,
        ShotGoalLineCrossingPredictionKind::SavedShotPreSaveSurfaceBounces
    );
}

#[test]
fn player_stat_events_with_shot_saves_allows_small_no_touch_shot_stat_lag() {
    let mut shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 10);
    let moving_away_ball = rigid_body(
        glam::Vec3::new(0.0, 4800.0, 120.0),
        Some(glam::Vec3::new(0.0, -1000.0, 0.0)),
    );
    shot.time = 1.0;
    shot.frame = 10;
    shot.shot = Some(ShotEventMetadata::from_rigid_bodies(
        true,
        &moving_away_ball,
        None,
    ));

    let mut save = player_stat_event(PlayerStatEventKind::Save, 2, false, 13);
    save.time = 1.3;
    save.frame = 13;
    let frame_data = frame_data_with_ball_frames(vec![
        (0.0, moving_away_ball),
        (
            0.9,
            rigid_body(
                glam::Vec3::new(0.0, 5080.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (1.0, moving_away_ball),
        (1.3, moving_away_ball),
    ]);

    let events =
        player_stat_events_with_shot_saves_and_frame_data(&[shot, save], Some(&frame_data), None);
    let crossing = events[0]
        .shot
        .as_ref()
        .unwrap()
        .projected_goal_line_crossing
        .as_ref()
        .expect("small no-touch shot stat lag should still produce a crossing estimate");

    assert_eq!(crossing.prediction_start_time, Some(0.9));
    assert_eq!(crossing.prediction_start_frame, Some(1));
    assert!(crossing.inside_goal_mouth);
    assert_eq!(
        crossing.prediction_kind,
        ShotGoalLineCrossingPredictionKind::SavedShotPreSaveSurfaceBounces
    );
}

#[test]
fn player_stat_events_with_shot_saves_uses_recent_pre_shot_save_touch_for_stat_lag() {
    let mut shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 10);
    let moving_away_ball = rigid_body(
        glam::Vec3::new(0.0, 4800.0, 120.0),
        Some(glam::Vec3::new(0.0, -1000.0, 0.0)),
    );
    shot.time = 1.0;
    shot.frame = 10;
    shot.shot = Some(ShotEventMetadata::from_rigid_bodies(
        true,
        &moving_away_ball,
        None,
    ));

    let mut delayed_save_stat = player_stat_event(PlayerStatEventKind::Save, 2, false, 11);
    delayed_save_stat.time = 1.1;
    delayed_save_stat.frame = 11;
    let frame_data = frame_data_with_ball_frames(vec![
        (0.0, moving_away_ball),
        (
            0.8,
            rigid_body(
                glam::Vec3::new(0.0, 4200.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (0.9, moving_away_ball),
        (1.0, moving_away_ball),
        (1.1, moving_away_ball),
    ]);
    let touches = [touch_event(2, false, 2, 0.9)];

    let events = player_stat_events_with_shot_saves_and_frame_data(
        &[shot, delayed_save_stat],
        Some(&frame_data),
        Some(&touches),
    );
    let crossing = events[0]
        .shot
        .as_ref()
        .unwrap()
        .projected_goal_line_crossing
        .as_ref()
        .expect("pre-shot save touch stat lag should still use the pre-save path");

    assert!(crossing.time_after_shot > 0.1);
    assert!(crossing.inside_goal_mouth);
    assert_eq!(
        crossing.prediction_kind,
        ShotGoalLineCrossingPredictionKind::SavedShotPreSaveSurfaceBounces
    );
}

#[test]
fn player_stat_events_with_shot_saves_estimates_from_before_saving_touch() {
    let mut shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 2);
    let moving_away_ball = rigid_body(
        glam::Vec3::new(0.0, 5000.0, 120.0),
        Some(glam::Vec3::new(0.0, -1000.0, 0.0)),
    );
    shot.time = 0.2;
    shot.frame = 2;
    shot.shot = Some(ShotEventMetadata::from_rigid_bodies(
        true,
        &moving_away_ball,
        None,
    ));

    let mut save = player_stat_event(PlayerStatEventKind::Save, 2, false, 30);
    save.time = 3.0;
    save.frame = 30;
    let frame_data = frame_data_with_ball_frames(vec![
        (
            0.0,
            rigid_body(
                glam::Vec3::new(0.0, 2000.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (
            0.1,
            rigid_body(
                glam::Vec3::new(0.0, 2500.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (
            0.2,
            rigid_body(
                glam::Vec3::new(3000.0, 3000.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (3.0, moving_away_ball),
    ]);
    let touches = [touch_event(2, false, 2, 0.2)];

    let events = player_stat_events_with_shot_saves_and_frame_data(
        &[shot, save],
        Some(&frame_data),
        Some(&touches),
    );
    let crossing = events[0]
        .shot
        .as_ref()
        .unwrap()
        .projected_goal_line_crossing
        .as_ref()
        .expect("saved shot should use the latest inbound frame before the saving touch");

    assert_eq!(crossing.position.x, 0.0);
    assert_eq!(crossing.position.y, STANDARD_GOAL_LINE_Y);
}

#[test]
fn player_stat_events_with_shot_saves_uses_inferred_shot_touch_for_delayed_shot_stat() {
    let mut shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 2);
    let moving_away_ball = rigid_body(
        glam::Vec3::new(0.0, 5000.0, 120.0),
        Some(glam::Vec3::new(0.0, -1000.0, 0.0)),
    );
    shot.time = 0.2;
    shot.frame = 2;
    shot.shot = Some(ShotEventMetadata::from_rigid_bodies(
        true,
        &moving_away_ball,
        None,
    ));

    let mut save = player_stat_event(PlayerStatEventKind::Save, 2, false, 4);
    save.time = 0.4;
    save.frame = 4;
    let frame_data = frame_data_with_ball_frames(vec![
        (
            0.0,
            rigid_body(
                glam::Vec3::new(0.0, 2000.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (
            0.1,
            rigid_body(
                glam::Vec3::new(0.0, 2600.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (0.2, moving_away_ball),
        (
            0.3,
            rigid_body(
                glam::Vec3::new(0.0, 3000.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (0.4, moving_away_ball),
    ]);
    let touches = [touch_event(1, true, 1, 0.1), touch_event(2, false, 4, 0.4)];

    let events = player_stat_events_with_shot_saves_and_frame_data(
        &[shot, save],
        Some(&frame_data),
        Some(&touches),
    );
    let crossing = events[0]
        .shot
        .as_ref()
        .unwrap()
        .projected_goal_line_crossing
        .as_ref()
        .expect("delayed shot stat should use the inferred attacking touch");

    assert_eq!(crossing.prediction_start_time, Some(0.1));
    assert_eq!(crossing.prediction_start_frame, Some(1));
    assert!(crossing.time_after_shot > 0.0);
    assert_eq!(crossing.position.y, STANDARD_GOAL_LINE_Y);
}

#[test]
fn player_stat_events_with_shot_saves_skips_unprojectable_inbound_frames() {
    let mut shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 1);
    let moving_away_ball = rigid_body(
        glam::Vec3::new(0.0, 4800.0, 120.0),
        Some(glam::Vec3::new(0.0, -1000.0, 0.0)),
    );
    shot.time = 0.1;
    shot.frame = 1;
    shot.shot = Some(ShotEventMetadata::from_rigid_bodies(
        true,
        &moving_away_ball,
        None,
    ));

    let mut save = player_stat_event(PlayerStatEventKind::Save, 2, false, 4);
    save.time = 0.4;
    save.frame = 4;
    let frame_data = frame_data_with_ball_frames(vec![
        (
            0.0,
            rigid_body(
                glam::Vec3::new(0.0, 2000.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (0.1, moving_away_ball),
        (
            0.2,
            rigid_body(
                glam::Vec3::new(4000.0, 3000.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (
            0.3,
            rigid_body(
                glam::Vec3::new(0.0, 2600.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (0.4, moving_away_ball),
    ]);

    let events =
        player_stat_events_with_shot_saves_and_frame_data(&[shot, save], Some(&frame_data), None);
    let crossing = events[0]
        .shot
        .as_ref()
        .unwrap()
        .projected_goal_line_crossing
        .as_ref()
        .expect("estimator should continue past the unprojectable inbound frame");

    assert_eq!(crossing.position.x, 0.0);
    assert_eq!(crossing.position.y, STANDARD_GOAL_LINE_Y);
}

#[test]
fn player_stat_events_with_shot_saves_rejects_crossing_before_saving_touch() {
    let mut shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 1);
    let moving_away_ball = rigid_body(
        glam::Vec3::new(0.0, 5000.0, 120.0),
        Some(glam::Vec3::new(0.0, -1000.0, 0.0)),
    );
    shot.time = 0.1;
    shot.frame = 1;
    shot.shot = Some(ShotEventMetadata::from_rigid_bodies(
        true,
        &moving_away_ball,
        None,
    ));

    let mut save = player_stat_event(PlayerStatEventKind::Save, 2, false, 4);
    save.time = 0.4;
    save.frame = 4;
    let frame_data = frame_data_with_ball_frames(vec![
        (
            0.0,
            rigid_body(
                glam::Vec3::new(0.0, STANDARD_GOAL_LINE_Y - 10.0, 120.0),
                Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            ),
        ),
        (0.1, moving_away_ball),
        (0.2, moving_away_ball),
        (0.3, moving_away_ball),
        (0.4, moving_away_ball),
    ]);
    let touches = [touch_event(1, true, 0, 0.0), touch_event(2, false, 4, 0.4)];

    let events = player_stat_events_with_shot_saves_and_frame_data(
        &[shot, save],
        Some(&frame_data),
        Some(&touches),
    );

    assert!(
        events[0]
            .shot
            .as_ref()
            .unwrap()
            .projected_goal_line_crossing
            .is_none()
    );
    assert!(events[0].shot.as_ref().unwrap().resulting_save.is_some());
    assert_eq!(
        events[0]
            .shot
            .as_ref()
            .unwrap()
            .projected_goal_line_crossing_unavailable_reason,
        Some(ShotGoalLineCrossingUnavailableReason::CrossingsBeforeSaveTouch)
    );
}
