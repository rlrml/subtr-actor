use super::*;
use crate::stats::accumulators::WhiffTestProjection;

const FPS: f32 = 30.0;

fn rigid_body(position: glam::Vec3, velocity: glam::Vec3) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: boxcars::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(glam_to_vec(&velocity)),
        angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
    }
}

fn player_at(id: u64, is_team_0: bool, position: glam::Vec3, velocity: glam::Vec3) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(id),
        is_team_0,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(position, velocity)),
        boost_amount: Some(50.0),
        last_boost_amount: None,
        boost_active: false,
        dodge_active: false,
        dodge_torque: None,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

fn ball() -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, 0.0, 92.75), glam::Vec3::ZERO),
    })
}

fn touch(frame: usize, time: f32, player: u64, team_is_team_0: bool) -> TouchState {
    TouchState {
        touch_events: vec![TouchEvent {
            touch_id: None,
            time,
            frame,
            team_is_team_0,
            player: Some(boxcars::RemoteId::Steam(player)),
            player_position: None,
            closest_approach_distance: Some(0.0),
            contact_local_ball_position: None,
            contact_local_hitbox_point: None,
            contact_world_hitbox_point: None,
            dodge_contact: false,
        }],
        ..TouchState::default()
    }
}

fn clean(_: &WhiffFeatures) -> WhiffClassification {
    WhiffClassification {
        class: WhiffAttemptClass::CleanWhiff,
        probabilities: [0.9, 0.05, 0.03, 0.02],
    }
}

fn disrupted(_: &WhiffFeatures) -> WhiffClassification {
    WhiffClassification {
        class: WhiffAttemptClass::DisruptedAttempt,
        probabilities: [0.05, 0.9, 0.03, 0.02],
    }
}

fn contact(_: &WhiffFeatures) -> WhiffClassification {
    WhiffClassification {
        class: WhiffAttemptClass::SuccessfulContact,
        probabilities: [0.05, 0.05, 0.85, 0.05],
    }
}

/// Car 1 drives along x past a stationary ball (200 uu to the side), so its
/// closest approach is at x = 0, one second in.
fn drive_by_x(time: f32) -> f32 {
    -1500.0 + 1500.0 * time
}

struct Scenario {
    calculator: WhiffCalculator,
    frame: usize,
}

impl Scenario {
    fn new(classifier: WhiffClassifier) -> Self {
        let mut calculator = WhiffCalculator::with_classifier(classifier);
        calculator.enable_candidate_recording();
        Self {
            calculator,
            frame: 0,
        }
    }

    fn time(&self) -> f32 {
        self.frame as f32 / FPS
    }

    fn step_with(&mut self, x: f32, touches: &TouchState, live: bool) {
        let time = self.time();
        let mut players = vec![player_at(
            1,
            true,
            glam::Vec3::new(x, 200.0, 17.0),
            glam::Vec3::new(1500.0, 0.0, 0.0),
        )];
        players.push(player_at(
            2,
            false,
            glam::Vec3::new(0.0, -2000.0, 17.0),
            glam::Vec3::ZERO,
        ));
        let live = if live {
            LivePlayState::active_play()
        } else {
            LivePlayState::default()
        };
        self.calculator
            .update(
                &FrameInfo {
                    frame_number: self.frame,
                    time,
                    dt: 1.0 / FPS,
                    seconds_remaining: None,
                },
                &ball(),
                &PlayerFrameState { players },
                &PlayerControlState::default(),
                touches,
                &live,
            )
            .unwrap();
        self.frame += 1;
    }

    fn run_until(&mut self, end: f32, x: impl Fn(f32) -> f32) {
        while self.time() <= end {
            self.step_with(x(self.time()), &TouchState::default(), true);
        }
    }
}

#[test]
fn emits_closest_approach_after_bounded_resolution_delay() {
    let mut scenario = Scenario::new(clean);
    scenario.run_until(1.55, drive_by_x);
    assert!(
        scenario.calculator.events().is_empty(),
        "nothing is known before the resolution delay elapses"
    );
    scenario.run_until(2.5, drive_by_x);

    let events = scenario.calculator.events();
    let [event] = events else {
        panic!("expected one whiff, got {events:?}");
    };
    assert_eq!(event.kind, WhiffEventKind::Whiff);
    assert_eq!(event.player, boxcars::RemoteId::Steam(1));
    assert!(
        (event.time - 1.0).abs() < 0.05,
        "anchor at closest approach"
    );
    let delay = event.resolved_time - event.time;
    assert!((0.6 - 1e-4..=0.6 + 1.0 / FPS + 1e-4).contains(&delay));
    assert!(event.start_time < event.time);
    assert_eq!(
        event.resolution_reason,
        WhiffResolutionReason::AttemptClassified
    );
    assert_eq!(event.confidence, Some(0.9));
    let stats = &scenario.calculator.player_stats()[&boxcars::RemoteId::Steam(1)];
    assert_eq!((stats.whiff_count, stats.disrupted_attempt_count), (1, 0));
}

#[test]
fn disrupted_attempt_is_counted_separately_and_names_the_toucher() {
    let mut scenario = Scenario::new(disrupted);
    scenario.run_until(0.9, drive_by_x);
    let touches = touch(scenario.frame, scenario.time(), 2, false);
    scenario.step_with(drive_by_x(scenario.time()), &touches, true);
    scenario.run_until(2.5, drive_by_x);

    let events = scenario.calculator.events();
    let [event] = events else {
        panic!("expected one disrupted attempt, got {events:?}");
    };
    assert_eq!(event.kind, WhiffEventKind::DisruptedAttempt);
    assert_eq!(event.disrupted_by, Some(boxcars::RemoteId::Steam(2)));
    let stats = &scenario.calculator.player_stats()[&boxcars::RemoteId::Steam(1)];
    assert_eq!((stats.whiff_count, stats.disrupted_attempt_count), (0, 1));
}

#[test]
fn repeated_pass_is_a_new_attempt_not_suppressed_by_the_first() {
    // Out and back: closest approaches at 1.0 s and 2.0 s.
    let out_and_back = |t: f32| {
        if t <= 1.5 {
            drive_by_x(t)
        } else {
            drive_by_x(3.0 - t)
        }
    };
    let mut scenario = Scenario::new(clean);
    scenario.run_until(3.5, out_and_back);

    let times: Vec<f32> = scenario
        .calculator
        .events()
        .iter()
        .map(|e| e.time)
        .collect();
    assert_eq!(times.len(), 2, "{times:?}");
    assert!((times[0] - 1.0).abs() < 0.05 && (times[1] - 2.0).abs() < 0.05);
}

#[test]
fn contact_classification_records_candidate_without_event() {
    let mut scenario = Scenario::new(contact);
    scenario.run_until(2.5, drive_by_x);

    assert!(scenario.calculator.events().is_empty());
    let [candidate] = scenario.calculator.candidates() else {
        panic!("expected one candidate");
    };
    assert_eq!(
        candidate.classification.class,
        WhiffAttemptClass::SuccessfulContact
    );
}

#[test]
fn closest_approach_outside_live_play_is_not_a_candidate() {
    let mut scenario = Scenario::new(clean);
    while scenario.time() <= 2.5 {
        let x = drive_by_x(scenario.time());
        scenario.step_with(x, &TouchState::default(), false);
    }
    assert!(scenario.calculator.candidates().is_empty());
}

#[test]
fn finish_resolves_pending_candidate_with_observed_frames() {
    let mut scenario = Scenario::new(clean);
    scenario.run_until(1.4, drive_by_x);
    assert!(scenario.calculator.events().is_empty());
    scenario.calculator.finish();
    assert_eq!(scenario.calculator.events().len(), 1);
}

#[test]
fn whiff_stats_keep_clean_breakdown_and_disrupted_counts_separate() {
    let event = |kind, aerial, dodge_active, closest_approach_distance| WhiffEvent {
        kind,
        start_time: 0.8,
        start_frame: 8,
        time: 1.0,
        frame: 10,
        resolved_time: 1.6,
        resolved_frame: 16,
        resolution_reason: WhiffResolutionReason::AttemptClassified,
        player: boxcars::RemoteId::Steam(1),
        player_position: None,
        is_team_0: true,
        closest_approach_distance,
        forward_alignment: 0.9,
        approach_speed: 900.0,
        closing_speed_at_closest: Some(850.0),
        velocity_alignment_at_closest: Some(0.95),
        local_ball_position_at_closest: Some([100.0, 0.0, 50.0]),
        resolved_distance: Some(400.0),
        dodge_active,
        aerial,
        confidence: None,
        disrupted_by: None,
    };
    let mut accumulator = WhiffStatsAccumulator::default();
    let frame = FrameInfo {
        frame_number: 16,
        time: 1.6,
        dt: 1.0 / FPS,
        seconds_remaining: None,
    };
    for e in [
        event(WhiffEventKind::Whiff, false, false, 120.0),
        event(WhiffEventKind::Whiff, true, true, 90.0),
        event(WhiffEventKind::DisruptedAttempt, true, false, 60.0),
    ] {
        accumulator.apply_event(&e, &frame);
    }
    let stats = &accumulator.player_stats()[&boxcars::RemoteId::Steam(1)];
    assert_eq!(stats.whiff_count, 2);
    assert_eq!(
        (stats.grounded_whiff_count, stats.aerial_whiff_count),
        (1, 1)
    );
    assert_eq!(stats.dodge_whiff_count, 1);
    assert_eq!(stats.best_closest_approach_distance, Some(90.0));
    assert_eq!(
        (
            stats.disrupted_attempt_count,
            stats.aerial_disrupted_attempt_count
        ),
        (1, 1)
    );
    assert_eq!(
        stats.whiff_count_with_labels(&[StatLabel::new("vertical_state", "aerial")]),
        1
    );
}
