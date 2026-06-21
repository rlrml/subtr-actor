use super::*;

fn player(id: u64) -> PlayerId {
    boxcars::RemoteId::Steam(id)
}

fn frame_at(time: f32) -> FrameInfo {
    FrameInfo {
        frame_number: (time * 10.0).round() as usize,
        time,
        dt: 0.1,
        seconds_remaining: None,
    }
}

fn touch_at(player_id: &PlayerId, time: f32) -> TouchEvent {
    TouchEvent {
        touch_id: None,
        time,
        frame: (time * 10.0).round() as usize,
        team_is_team_0: true,
        player: Some(player_id.clone()),
        player_position: None,
        closest_approach_distance: None,
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
        dodge_contact: false,
    }
}

fn stat_event(player_id: &PlayerId, time: f32, kind: PlayerStatEventKind) -> PlayerStatEvent {
    PlayerStatEvent {
        time,
        frame: (time * 10.0).round() as usize,
        player: player_id.clone(),
        player_position: None,
        is_team_0: true,
        kind,
        shot: None,
    }
}

fn neutral_ctx() -> TouchIntentionFrameContext<'static> {
    TouchIntentionFrameContext {
        ball_position: Some(glam::Vec3::new(0.0, 0.0, BALL_RADIUS_Z)),
        ball_velocity: Some(glam::Vec3::ZERO),
        previous_ball_position: None,
        previous_ball_velocity: None,
        teammate_positions: &[],
        contested: false,
    }
}

#[test]
fn replay_reported_save_outranks_contested_challenge() {
    let player_id = player(1);
    let mut classifier = TouchIntentionClassifier::default();
    classifier.begin_frame(
        &frame_at(10.0),
        &[stat_event(&player_id, 10.0, PlayerStatEventKind::Save)],
    );

    let resolution = classifier.classify(
        &touch_at(&player_id, 10.0),
        &player_id,
        &TouchIntentionFrameContext {
            contested: true,
            ..neutral_ctx()
        },
    );

    assert_eq!(resolution.intention, TouchIntention::Save);
    assert!(resolution.contested);
}

#[test]
fn stat_event_for_other_player_does_not_match() {
    let toucher = player(1);
    let other = player(2);
    let mut classifier = TouchIntentionClassifier::default();
    classifier.begin_frame(
        &frame_at(10.0),
        &[stat_event(&other, 10.0, PlayerStatEventKind::Shot)],
    );

    let resolution = classifier.classify(&touch_at(&toucher, 10.0), &toucher, &neutral_ctx());

    assert_eq!(resolution.intention, TouchIntention::Neutral);
}

#[test]
fn stat_event_outside_match_window_does_not_match() {
    let player_id = player(1);
    let mut classifier = TouchIntentionClassifier::default();
    classifier.begin_frame(
        &frame_at(10.0),
        &[stat_event(&player_id, 10.0, PlayerStatEventKind::Shot)],
    );

    let resolution = classifier.classify(&touch_at(&player_id, 11.0), &player_id, &neutral_ctx());

    assert_eq!(resolution.intention, TouchIntention::Neutral);
}

#[test]
fn contested_touch_outranks_geometric_shot() {
    let player_id = player(1);
    let mut classifier = TouchIntentionClassifier::default();

    let resolution = classifier.classify(
        &touch_at(&player_id, 1.0),
        &player_id,
        &TouchIntentionFrameContext {
            ball_position: Some(glam::Vec3::new(0.0, 4000.0, BALL_RADIUS_Z)),
            ball_velocity: Some(glam::Vec3::new(0.0, 2000.0, 0.0)),
            contested: true,
            ..neutral_ctx()
        },
    );

    assert_eq!(resolution.intention, TouchIntention::Challenge);
    assert!(resolution.contested);
}

#[test]
fn fast_touch_toward_goal_mouth_classifies_as_shot() {
    let player_id = player(1);
    let mut classifier = TouchIntentionClassifier::default();

    let resolution = classifier.classify(
        &touch_at(&player_id, 1.0),
        &player_id,
        &TouchIntentionFrameContext {
            ball_position: Some(glam::Vec3::new(0.0, 4000.0, BALL_RADIUS_Z)),
            ball_velocity: Some(glam::Vec3::new(0.0, 2000.0, 0.0)),
            ..neutral_ctx()
        },
    );

    assert_eq!(resolution.intention, TouchIntention::Shot);
}

#[test]
fn upward_arced_shot_into_net_classifies_as_shot() {
    // A shot hit hard with significant upward velocity: a straight-line
    // projection sails it well over the crossbar (z ~1500), but gravity arcs it
    // back down into the goal mouth (z ~160). This mirrors a dribble-flick that
    // scores along an obvious arc, which used to read as Neutral.
    let player_id = player(1);
    let mut classifier = TouchIntentionClassifier::default();

    let resolution = classifier.classify(
        &touch_at(&player_id, 1.0),
        &player_id,
        &TouchIntentionFrameContext {
            ball_position: Some(glam::Vec3::new(0.0, 0.0, BALL_RADIUS_Z)),
            ball_velocity: Some(glam::Vec3::new(0.0, 2500.0, 700.0)),
            ..neutral_ctx()
        },
    );

    assert_eq!(resolution.intention, TouchIntention::Shot);
}

#[test]
fn shot_sailing_over_the_crossbar_is_not_a_shot() {
    // Even with gravity, a ball rocketing upward is still well above the
    // crossbar when it reaches the goal line, so it must not read as a shot.
    let player_id = player(1);
    let mut classifier = TouchIntentionClassifier::default();

    let resolution = classifier.classify(
        &touch_at(&player_id, 1.0),
        &player_id,
        &TouchIntentionFrameContext {
            ball_position: Some(glam::Vec3::new(0.0, 4000.0, BALL_RADIUS_Z)),
            ball_velocity: Some(glam::Vec3::new(0.0, 2500.0, 2000.0)),
            ..neutral_ctx()
        },
    );

    assert_eq!(resolution.intention, TouchIntention::Neutral);
}

#[test]
fn touch_wide_of_goal_mouth_is_not_a_shot() {
    let player_id = player(1);
    let mut classifier = TouchIntentionClassifier::default();

    let resolution = classifier.classify(
        &touch_at(&player_id, 1.0),
        &player_id,
        &TouchIntentionFrameContext {
            ball_position: Some(glam::Vec3::new(0.0, 4000.0, BALL_RADIUS_Z)),
            ball_velocity: Some(glam::Vec3::new(3000.0, 2000.0, 0.0)),
            ..neutral_ctx()
        },
    );

    assert_eq!(resolution.intention, TouchIntention::Neutral);
}

#[test]
fn slow_roll_toward_goal_is_not_a_shot() {
    let player_id = player(1);
    let mut classifier = TouchIntentionClassifier::default();

    let resolution = classifier.classify(
        &touch_at(&player_id, 1.0),
        &player_id,
        &TouchIntentionFrameContext {
            ball_position: Some(glam::Vec3::new(0.0, 4000.0, BALL_RADIUS_Z)),
            ball_velocity: Some(glam::Vec3::new(0.0, 900.0, 0.0)),
            ..neutral_ctx()
        },
    );

    assert_eq!(resolution.intention, TouchIntention::Neutral);
}

#[test]
fn redirect_of_ball_headed_into_own_goal_is_a_save() {
    let player_id = player(1);
    let mut classifier = TouchIntentionClassifier::default();

    let resolution = classifier.classify(
        &touch_at(&player_id, 1.0),
        &player_id,
        &TouchIntentionFrameContext {
            ball_position: Some(glam::Vec3::new(0.0, -4200.0, BALL_RADIUS_Z)),
            ball_velocity: Some(glam::Vec3::new(800.0, 0.0, 0.0)),
            previous_ball_position: Some(glam::Vec3::new(0.0, -4000.0, BALL_RADIUS_Z)),
            previous_ball_velocity: Some(glam::Vec3::new(0.0, -2000.0, 0.0)),
            ..neutral_ctx()
        },
    );

    assert_eq!(resolution.intention, TouchIntention::Save);
}

#[test]
fn fast_touch_out_of_defensive_third_is_a_clear() {
    let player_id = player(1);
    let mut classifier = TouchIntentionClassifier::default();

    let resolution = classifier.classify(
        &touch_at(&player_id, 1.0),
        &player_id,
        &TouchIntentionFrameContext {
            ball_position: Some(glam::Vec3::new(0.0, -4000.0, BALL_RADIUS_Z)),
            ball_velocity: Some(glam::Vec3::new(1400.0, 800.0, 0.0)),
            ..neutral_ctx()
        },
    );

    assert_eq!(resolution.intention, TouchIntention::Clear);
}

#[test]
fn slow_touch_in_defensive_third_is_not_a_clear() {
    let player_id = player(1);
    let mut classifier = TouchIntentionClassifier::default();

    let resolution = classifier.classify(
        &touch_at(&player_id, 1.0),
        &player_id,
        &TouchIntentionFrameContext {
            ball_position: Some(glam::Vec3::new(0.0, -4000.0, BALL_RADIUS_Z)),
            ball_velocity: Some(glam::Vec3::new(300.0, 200.0, 0.0)),
            ..neutral_ctx()
        },
    );

    assert_eq!(resolution.intention, TouchIntention::Neutral);
}

#[test]
fn touch_leading_a_teammate_is_a_pass() {
    let player_id = player(1);
    let mut classifier = TouchIntentionClassifier::default();
    let teammates = [glam::Vec3::new(0.0, 1000.0, 17.0)];

    let resolution = classifier.classify(
        &touch_at(&player_id, 1.0),
        &player_id,
        &TouchIntentionFrameContext {
            ball_velocity: Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            teammate_positions: &teammates,
            ..neutral_ctx()
        },
    );

    assert_eq!(resolution.intention, TouchIntention::Pass);
}

#[test]
fn touch_away_from_teammates_is_not_a_pass() {
    let player_id = player(1);
    let mut classifier = TouchIntentionClassifier::default();
    let teammates = [glam::Vec3::new(2000.0, -1000.0, 17.0)];

    let resolution = classifier.classify(
        &touch_at(&player_id, 1.0),
        &player_id,
        &TouchIntentionFrameContext {
            ball_velocity: Some(glam::Vec3::new(0.0, 1000.0, 0.0)),
            teammate_positions: &teammates,
            ..neutral_ctx()
        },
    );

    assert_eq!(resolution.intention, TouchIntention::Neutral);
}

#[test]
fn first_touch_tracking_follows_reception_changes() {
    let first_player = player(1);
    let second_player = player(2);
    let mut classifier = TouchIntentionClassifier::default();

    let opening = classifier.classify(&touch_at(&first_player, 1.0), &first_player, &neutral_ctx());
    assert!(opening.first_touch);

    let continuation =
        classifier.classify(&touch_at(&first_player, 1.5), &first_player, &neutral_ctx());
    assert!(!continuation.first_touch);

    let interception = classifier.classify(
        &touch_at(&second_player, 1.6),
        &second_player,
        &neutral_ctx(),
    );
    assert!(interception.first_touch);

    let after_gap = classifier.classify(
        &touch_at(&second_player, 4.5),
        &second_player,
        &neutral_ctx(),
    );
    assert!(after_gap.first_touch);
}

#[test]
fn contested_interruption_does_not_break_a_reception() {
    let dribbler = player(1);
    let challenger = player(2);
    let mut classifier = TouchIntentionClassifier::default();

    let opening = classifier.classify(&touch_at(&dribbler, 1.0), &dribbler, &neutral_ctx());
    assert!(opening.first_touch);

    let contested_ctx = TouchIntentionFrameContext {
        contested: true,
        ..neutral_ctx()
    };
    let challenge = classifier.classify(&touch_at(&challenger, 1.5), &challenger, &contested_ctx);
    assert!(challenge.first_touch);
    assert_eq!(challenge.intention, TouchIntention::Challenge);

    let recovery = classifier.classify(&touch_at(&dribbler, 2.0), &dribbler, &neutral_ctx());
    assert!(!recovery.first_touch);
}

#[test]
fn clean_touch_after_winning_a_contest_starts_a_new_reception() {
    let dribbler = player(1);
    let challenger = player(2);
    let mut classifier = TouchIntentionClassifier::default();

    classifier.classify(&touch_at(&dribbler, 1.0), &dribbler, &neutral_ctx());

    let contested_ctx = TouchIntentionFrameContext {
        contested: true,
        ..neutral_ctx()
    };
    classifier.classify(&touch_at(&challenger, 1.5), &challenger, &contested_ctx);

    let takeover = classifier.classify(&touch_at(&challenger, 2.0), &challenger, &neutral_ctx());
    assert!(takeover.first_touch);

    let follow_up = classifier.classify(&touch_at(&challenger, 2.4), &challenger, &neutral_ctx());
    assert!(!follow_up.first_touch);
}

#[test]
fn reset_starts_a_new_reception() {
    let player_id = player(1);
    let mut classifier = TouchIntentionClassifier::default();

    classifier.classify(&touch_at(&player_id, 1.0), &player_id, &neutral_ctx());
    classifier.reset();
    let resolution = classifier.classify(&touch_at(&player_id, 1.2), &player_id, &neutral_ctx());

    assert!(resolution.first_touch);
}

#[test]
fn follow_up_touch_by_same_player_confirms_control() {
    let player_id = player(1);
    let mut tracker = ControlFollowTracker::default();
    tracker.open(0, &player_id, 1.0);

    let resolution = tracker.observe_touch(&player_id, 1.8).unwrap();

    assert_eq!(
        resolution,
        ControlResolution {
            touch_index: 0,
            control: true,
        }
    );
}

#[test]
fn late_follow_up_touch_does_not_confirm_control() {
    let player_id = player(1);
    let mut tracker = ControlFollowTracker::default();
    tracker.open(0, &player_id, 1.0);

    let resolution = tracker.observe_touch(&player_id, 3.0).unwrap();

    assert!(!resolution.control);
}

#[test]
fn staying_close_and_speed_matched_resolves_control_on_timeout() {
    let player_id = player(1);
    let mut tracker = ControlFollowTracker::default();
    tracker.open(0, &player_id, 0.0);

    for step in 1..=12 {
        let time = step as f32 * 0.1;
        let resolution = tracker.advance(
            &frame_at(time),
            Some(glam::Vec3::new(0.0, time * 300.0, BALL_RADIUS_Z)),
            Some(glam::Vec3::new(0.0, 300.0, 0.0)),
            Some(glam::Vec3::new(0.0, time * 300.0 - 100.0, 17.0)),
            Some(glam::Vec3::new(0.0, 300.0, 0.0)),
        );
        assert!(resolution.is_none());
    }

    let resolution = tracker
        .advance(&frame_at(1.3), None, None, None, None)
        .unwrap();
    assert_eq!(
        resolution,
        ControlResolution {
            touch_index: 0,
            control: true,
        }
    );
}

#[test]
fn ball_leaving_the_player_resolves_as_not_control() {
    let toucher = player(1);
    let stealer = player(2);
    let mut tracker = ControlFollowTracker::default();
    tracker.open(0, &toucher, 0.0);

    for step in 1..=5 {
        let time = step as f32 * 0.1;
        tracker.advance(
            &frame_at(time),
            Some(glam::Vec3::new(0.0, time * 2000.0, BALL_RADIUS_Z)),
            Some(glam::Vec3::new(0.0, 2000.0, 0.0)),
            Some(glam::Vec3::ZERO),
            Some(glam::Vec3::ZERO),
        );
    }

    let resolution = tracker.observe_touch(&stealer, 0.6).unwrap();
    assert_eq!(
        resolution,
        ControlResolution {
            touch_index: 0,
            control: false,
        }
    );
}

#[test]
fn window_cut_short_does_not_confirm_control() {
    let player_id = player(1);
    let mut tracker = ControlFollowTracker::default();
    tracker.open(0, &player_id, 0.0);

    tracker.advance(
        &frame_at(0.1),
        Some(glam::Vec3::new(0.0, 0.0, BALL_RADIUS_Z)),
        Some(glam::Vec3::ZERO),
        Some(glam::Vec3::new(0.0, -100.0, 17.0)),
        Some(glam::Vec3::ZERO),
    );

    let resolution = tracker.flush().unwrap();
    assert!(!resolution.control);
}
