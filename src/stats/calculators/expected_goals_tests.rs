use super::*;

fn player_id(id: u64) -> PlayerId {
    boxcars::RemoteId::Steam(id)
}

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

fn ball(position: glam::Vec3, velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(position, velocity),
    })
}

fn player(
    id: u64,
    is_team_0: bool,
    position: glam::Vec3,
    boost_amount: Option<f32>,
) -> PlayerSample {
    PlayerSample {
        player_id: player_id(id),
        is_team_0,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(position, glam::Vec3::ZERO)),
        boost_amount,
        last_boost_amount: boost_amount,
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

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.1,
        seconds_remaining: None,
    }
}

fn live_play() -> LivePlayState {
    LivePlayState::active_play()
}

fn stoppage() -> LivePlayState {
    LivePlayState::new(GameplayPhase::PostGoal)
}

fn touch(frame_number: usize, time: f32, id: u64, is_team_0: bool) -> TouchEvent {
    TouchEvent {
        touch_id: None,
        time,
        frame: frame_number,
        team_is_team_0: is_team_0,
        player: Some(player_id(id)),
        player_position: None,
        closest_approach_distance: None,
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
        dodge_contact: false,
    }
}

fn touch_with_id(
    frame_number: usize,
    time: f32,
    id: u64,
    is_team_0: bool,
    touch_id: Option<u64>,
) -> TouchEvent {
    TouchEvent {
        touch_id,
        ..touch(frame_number, time, id, is_team_0)
    }
}

fn touch_state(touches: Vec<TouchEvent>) -> TouchState {
    TouchState {
        touch_events: touches,
        ..TouchState::default()
    }
}

fn goal_event(frame_number: usize, time: f32, scoring_team_is_team_0: bool) -> GoalEvent {
    GoalEvent {
        time,
        frame: frame_number,
        scoring_team_is_team_0,
        player: None,
        player_position: None,
        team_zero_score: None,
        team_one_score: None,
    }
}

fn no_demoed() -> HashSet<PlayerId> {
    HashSet::new()
}

/// Ball rolling in on an open team-zero net; the placeholder model rates this
/// far above the episode threshold.
fn dangerous_state() -> (BallFrameState, PlayerFrameState) {
    (
        ball(
            glam::Vec3::new(0.0, 4300.0, 93.0),
            glam::Vec3::new(0.0, 1400.0, 250.0),
        ),
        PlayerFrameState {
            players: vec![
                player(1, true, glam::Vec3::new(0.0, 4000.0, 17.0), Some(100.0)),
                player(2, false, glam::Vec3::new(2500.0, 1000.0, 17.0), Some(50.0)),
            ],
        },
    )
}

/// Slow midfield ball with the defense set; rated well under the threshold.
fn neutral_state() -> (BallFrameState, PlayerFrameState) {
    (
        ball(glam::Vec3::new(0.0, 0.0, 93.0), glam::Vec3::ZERO),
        PlayerFrameState {
            players: vec![
                player(1, true, glam::Vec3::new(0.0, -1000.0, 17.0), Some(100.0)),
                player(2, false, glam::Vec3::new(0.0, 2000.0, 17.0), Some(50.0)),
            ],
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn update(
    calculator: &mut ExpectedGoalsCalculator,
    frame_number: usize,
    time: f32,
    ball: &BallFrameState,
    players: &PlayerFrameState,
    events: FrameEventsState,
    touches: Vec<TouchEvent>,
    live: LivePlayState,
) {
    calculator
        .update_parts(
            &frame(frame_number, time),
            &GameplayState::default(),
            ball,
            players,
            &events,
            &touch_state(touches),
            &live,
        )
        .unwrap();
}

#[test]
fn feature_names_and_array_agree() {
    assert_eq!(ThreatFeatures::FEATURE_NAMES.len(), THREAT_FEATURE_COUNT);
    let (ball, players) = dangerous_state();
    let features = compute_threat_features(
        ball.position().unwrap(),
        ball.velocity().unwrap(),
        &players,
        &no_demoed(),
        true,
    );
    assert_eq!(
        features.to_array().len(),
        ThreatFeatures::FEATURE_NAMES.len()
    );
    let mut names: Vec<_> = ThreatFeatures::FEATURE_NAMES.to_vec();
    names.sort_unstable();
    names.dedup();
    assert_eq!(
        names.len(),
        THREAT_FEATURE_COUNT,
        "feature names must be unique"
    );
}

#[test]
fn features_are_bounded() {
    let (ball, players) = dangerous_state();
    let features = compute_threat_features(
        ball.position().unwrap(),
        ball.velocity().unwrap(),
        &players,
        &no_demoed(),
        true,
    );
    for (name, value) in ThreatFeatures::FEATURE_NAMES
        .iter()
        .zip(features.to_array())
    {
        assert!(
            (-1.0..=4.0).contains(&value),
            "feature {name} out of bounds: {value}"
        );
        if *name != "attacking_team_size" {
            assert!(
                (-1.0..=1.0).contains(&value),
                "normalized feature {name} out of [-1, 1]: {value}"
            );
        }
    }
}

/// A team-one attack that is the exact 180-degree rotation of a team-zero
/// attack must produce identical features: the attacking-frame normalization
/// is what makes one model serve both teams.
#[test]
fn mirrored_state_yields_identical_features_for_the_other_team() {
    let ball_position = glam::Vec3::new(800.0, 3900.0, 250.0);
    let ball_velocity = glam::Vec3::new(-300.0, 1200.0, 150.0);
    let mirror = |v: glam::Vec3| glam::Vec3::new(-v.x, -v.y, v.z);

    let players_team_zero_attacking = PlayerFrameState {
        players: vec![
            player(1, true, glam::Vec3::new(500.0, 3400.0, 17.0), Some(80.0)),
            player(2, true, glam::Vec3::new(-1200.0, 1500.0, 17.0), Some(20.0)),
            player(3, false, glam::Vec3::new(100.0, 4700.0, 17.0), Some(140.0)),
            player(4, false, glam::Vec3::new(900.0, 2500.0, 400.0), None),
        ],
    };
    let players_mirrored = PlayerFrameState {
        players: players_team_zero_attacking
            .players
            .iter()
            .map(|sample| {
                player(
                    match sample.player_id {
                        boxcars::RemoteId::Steam(id) => id,
                        _ => unreachable!(),
                    },
                    !sample.is_team_0,
                    mirror(sample.position().unwrap()),
                    sample.boost_amount,
                )
            })
            .collect(),
    };

    let features_team_zero = compute_threat_features(
        ball_position,
        ball_velocity,
        &players_team_zero_attacking,
        &no_demoed(),
        true,
    );
    let features_team_one = compute_threat_features(
        mirror(ball_position),
        mirror(ball_velocity),
        &players_mirrored,
        &no_demoed(),
        false,
    );

    assert_eq!(features_team_zero.to_array(), features_team_one.to_array());
}

#[test]
fn ballistic_on_target_hand_computed_fixture() {
    let empty_players = PlayerFrameState::default();
    // 820 uu from the goal line at 1400 uu/s: crosses at t = 0.5857s with
    // z = 100 + 250 t + 0.5 * (-650) t^2 = 100 + 146.4 - 111.5 = 134.9 -- in
    // the mouth, dead center.
    let on_target = compute_threat_features(
        glam::Vec3::new(0.0, 4300.0, 100.0),
        glam::Vec3::new(0.0, 1400.0, 250.0),
        &empty_players,
        &no_demoed(),
        true,
    );
    assert_eq!(on_target.on_target, 1.0);
    let expected_time = (STANDARD_GOAL_LINE_Y - 4300.0) / 1400.0;
    assert!((on_target.time_to_goal_line - 1.0 / (1.0 + expected_time)).abs() < 1e-6);

    // Same shot hit much higher: z at the line = 100 + 1200 * 0.5857 - 111.5
    // = 691.4 -- over the 642.775 crossbar.
    let over_the_bar = compute_threat_features(
        glam::Vec3::new(0.0, 4300.0, 100.0),
        glam::Vec3::new(0.0, 1400.0, 1200.0),
        &empty_players,
        &no_demoed(),
        true,
    );
    assert_eq!(over_the_bar.on_target, 0.0);
    assert!(over_the_bar.time_to_goal_line > 0.0);

    // Crossing x = 2000: wide of the 892.755 post.
    let wide = compute_threat_features(
        glam::Vec3::new(2000.0, 4300.0, 100.0),
        glam::Vec3::new(0.0, 1400.0, 250.0),
        &empty_players,
        &no_demoed(),
        true,
    );
    assert_eq!(wide.on_target, 0.0);

    // Moving away from the goal: no crossing, no time-to-goal-line.
    let away = compute_threat_features(
        glam::Vec3::new(0.0, 4300.0, 100.0),
        glam::Vec3::new(0.0, -1400.0, 250.0),
        &empty_players,
        &no_demoed(),
        true,
    );
    assert_eq!(away.on_target, 0.0);
    assert_eq!(away.time_to_goal_line, 0.0);
}

#[test]
fn touch_delta_event_carries_before_and_after_values() {
    let mut calculator = ExpectedGoalsCalculator::new();
    let (neutral_ball, neutral_players) = neutral_state();
    let (danger_ball, danger_players) = dangerous_state();

    update(
        &mut calculator,
        1,
        1.0,
        &neutral_ball,
        &neutral_players,
        FrameEventsState::default(),
        vec![],
        live_play(),
    );
    assert!(calculator.touch_events().is_empty());
    let neutral_value = calculator.current_values().unwrap()[0];

    update(
        &mut calculator,
        2,
        1.1,
        &danger_ball,
        &danger_players,
        FrameEventsState::default(),
        vec![touch(2, 1.1, 1, true)],
        live_play(),
    );

    let events = calculator.touch_events();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.player, Some(player_id(1)));
    assert!(event.team_is_team_0);
    assert!((event.value_before - neutral_value).abs() < 1e-6);
    assert_eq!(event.value_after, calculator.current_values().unwrap()[0]);
    assert!(
        event.delta() > 0.0,
        "turning a neutral ball into an on-target chance must be a positive delta"
    );
}

/// Two same-team touches on one frame share a single previous-frame ->
/// current-frame V transition; only the team's primary (latest,
/// best-evidence) touch may be credited or the accumulator double-counts it.
#[test]
fn simultaneous_same_team_touches_credit_one_event_for_the_frame_transition() {
    let mut calculator = ExpectedGoalsCalculator::new();
    let (neutral_ball, neutral_players) = neutral_state();
    let (danger_ball, danger_players) = dangerous_state();

    update(
        &mut calculator,
        1,
        1.0,
        &neutral_ball,
        &neutral_players,
        FrameEventsState::default(),
        vec![],
        live_play(),
    );
    let neutral_value = calculator.current_values().unwrap()[0];

    // Player 5's contact is backdated a frame; player 1's is the latest
    // contact and therefore the team's primary touch.
    update(
        &mut calculator,
        2,
        1.1,
        &danger_ball,
        &danger_players,
        FrameEventsState::default(),
        vec![touch(1, 1.05, 5, true), touch(2, 1.1, 1, true)],
        live_play(),
    );

    let events = calculator.touch_events();
    assert_eq!(
        events.len(),
        1,
        "one frame transition must be credited exactly once per team"
    );
    let event = &events[0];
    assert_eq!(event.player, Some(player_id(1)));
    assert!((event.value_before - neutral_value).abs() < 1e-6);
    assert_eq!(event.value_after, calculator.current_values().unwrap()[0]);
}

/// A cache-recovered touch is backdated: contact fields keep the touch's own
/// frame/time/id while the detection fields (which the ΔV brackets) carry the
/// processing frame.
#[test]
fn backdated_touch_keeps_contact_fields_and_detection_fields_separate() {
    let mut calculator = ExpectedGoalsCalculator::new();
    let (neutral_ball, neutral_players) = neutral_state();
    let (danger_ball, danger_players) = dangerous_state();

    update(
        &mut calculator,
        2,
        1.1,
        &neutral_ball,
        &neutral_players,
        FrameEventsState::default(),
        vec![],
        live_play(),
    );
    let neutral_value = calculator.current_values().unwrap()[0];

    update(
        &mut calculator,
        3,
        1.2,
        &danger_ball,
        &danger_players,
        FrameEventsState::default(),
        vec![touch_with_id(1, 0.95, 1, true, Some(42))],
        live_play(),
    );

    let events = calculator.touch_events();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.frame, 1, "contact frame comes from the touch");
    assert!(
        (event.time - 0.95).abs() < 1e-6,
        "contact time comes from the touch"
    );
    assert_eq!(event.touch_id, Some(42));
    assert_eq!(
        event.detection_frame, 3,
        "detection frame is the processing frame"
    );
    assert!((event.detection_time - 1.2).abs() < 1e-6);
    assert!(
        (event.value_before - neutral_value).abs() < 1e-6,
        "value_before brackets the detection frame, not the contact frame"
    );
    assert_eq!(event.value_after, calculator.current_values().unwrap()[0]);
}

/// `defenders_goalside` normalizes by the DEFENDING team's eligible roster:
/// on a 2v1 the lone goalside defender is 1/1, not 1/2.
#[test]
fn defenders_goalside_normalizes_by_defending_team_size() {
    let players = PlayerFrameState {
        players: vec![
            player(1, true, glam::Vec3::new(0.0, 2000.0, 17.0), Some(100.0)),
            player(2, true, glam::Vec3::new(500.0, 1500.0, 17.0), Some(100.0)),
            player(3, false, glam::Vec3::new(0.0, 4000.0, 17.0), Some(50.0)),
        ],
    };
    let features = compute_threat_features(
        glam::Vec3::new(0.0, 3000.0, 93.0),
        glam::Vec3::ZERO,
        &players,
        &no_demoed(),
        true,
    );
    assert_eq!(features.attacking_team_size, 2.0);
    assert_eq!(
        features.defenders_goalside, 1.0,
        "one of one eligible defenders is goalside"
    );

    // The same frame with the defender demoed: no eligible defenders, and the
    // zero-roster guard keeps the feature finite.
    let demoed: HashSet<PlayerId> = [player_id(3)].into_iter().collect();
    let features_demoed = compute_threat_features(
        glam::Vec3::new(0.0, 3000.0, 93.0),
        glam::Vec3::ZERO,
        &players,
        &demoed,
        true,
    );
    assert_eq!(features_demoed.defenders_goalside, 0.0);
}

/// A same-team goal arriving after the pending episode's goal grace has
/// passed must NOT upgrade the stale episode: it closes as the stoppage it
/// already was.
#[test]
fn goal_after_pending_grace_expiry_does_not_upgrade_episode() {
    let mut calculator = ExpectedGoalsCalculator::new();
    let (danger_ball, danger_players) = dangerous_state();

    update(
        &mut calculator,
        1,
        1.0,
        &danger_ball,
        &danger_players,
        FrameEventsState::default(),
        vec![],
        live_play(),
    );
    update(
        &mut calculator,
        2,
        1.1,
        &danger_ball,
        &danger_players,
        FrameEventsState::default(),
        vec![],
        stoppage(),
    );
    assert!(calculator.episode_events().is_empty());

    // Goal detection runs before stale-pending expiry within a frame, so
    // this goal (well past closed_at + grace) sees the pending episode.
    update(
        &mut calculator,
        3,
        20.0,
        &danger_ball,
        &danger_players,
        FrameEventsState {
            goal_events: vec![goal_event(3, 20.0, true)],
            ..FrameEventsState::default()
        },
        vec![],
        stoppage(),
    );

    let episodes = calculator.episode_events();
    assert_eq!(episodes.len(), 1);
    assert!(!episodes[0].ended_in_goal);
    assert_eq!(episodes[0].end_reason, ThreatEpisodeEndReason::Stoppage);
    assert_eq!(calculator.goal_records().len(), 1);
}

#[test]
fn episode_opens_above_threshold_and_closes_on_value_drop() {
    let mut calculator = ExpectedGoalsCalculator::new();
    let (neutral_ball, neutral_players) = neutral_state();
    let (danger_ball, danger_players) = dangerous_state();

    update(
        &mut calculator,
        1,
        1.0,
        &neutral_ball,
        &neutral_players,
        FrameEventsState::default(),
        vec![],
        live_play(),
    );
    assert!(calculator.episode_events().is_empty());

    // The touch that creates the chance opens the episode on the same frame.
    update(
        &mut calculator,
        2,
        1.1,
        &danger_ball,
        &danger_players,
        FrameEventsState::default(),
        vec![touch(2, 1.1, 1, true)],
        live_play(),
    );
    update(
        &mut calculator,
        3,
        1.2,
        &danger_ball,
        &danger_players,
        FrameEventsState::default(),
        vec![],
        live_play(),
    );
    assert!(calculator.episode_events().is_empty());
    let peak = calculator.current_values().unwrap()[0];
    assert!(peak > THREAT_EPISODE_THRESHOLD);

    update(
        &mut calculator,
        4,
        1.3,
        &neutral_ball,
        &neutral_players,
        FrameEventsState::default(),
        vec![],
        live_play(),
    );
    let neutral_value = calculator.current_values().unwrap()[0];

    let episodes = calculator.episode_events();
    assert_eq!(episodes.len(), 1);
    let episode = &episodes[0];
    assert!(episode.team_is_team_0);
    assert_eq!(episode.start_frame, 2);
    assert_eq!(episode.end_frame, 4);
    assert_eq!(episode.end_reason, ThreatEpisodeEndReason::ValueDropped);
    assert!(!episode.ended_in_goal);
    assert!((episode.peak_value - peak).abs() < 1e-6);
    // xg is the time integral over the episode's evaluated frames: the two
    // danger frames plus the sub-threshold frame that closed it, each
    // contributing V * dt / tau with dt = 0.1.
    let expected_integral =
        (2.0 * peak + neutral_value) * 0.1 / expected_goals_model::THREAT_HORIZON_SECONDS;
    assert!((episode.xg - expected_integral).abs() < 1e-6);
    assert!(episode.xg < episode.peak_value);
    assert_eq!(episode.credited_player, Some(player_id(1)));
}

/// Player credit follows the toucher associated with the episode's peak, not
/// simply the last teammate to touch before the episode closes.
#[test]
fn later_lower_value_touch_does_not_steal_episode_credit_from_peak_toucher() {
    let mut calculator = ExpectedGoalsCalculator::new();
    calculator.team_states[0].last_toucher = Some(player_id(1));

    calculator.update_episodes(&frame(1, 1.0), [0.5, 0.0]);
    calculator.emit_touch_events(
        &frame(2, 1.1),
        &touch_state(vec![touch(2, 1.1, 2, true)]),
        [0.3, 0.0],
    );
    calculator.update_episodes(&frame(2, 1.1), [0.3, 0.0]);
    calculator.update_episodes(&frame(3, 1.2), [0.0, 0.0]);

    let episodes = calculator.episode_events();
    assert_eq!(episodes.len(), 1);
    assert_eq!(episodes[0].peak_value, 0.5);
    assert_eq!(episodes[0].credited_player, Some(player_id(1)));
}

#[test]
fn goal_closes_episode_with_goal_outcome() {
    let mut calculator = ExpectedGoalsCalculator::new();
    let (danger_ball, danger_players) = dangerous_state();

    update(
        &mut calculator,
        1,
        1.0,
        &danger_ball,
        &danger_players,
        FrameEventsState::default(),
        vec![touch(1, 1.0, 1, true)],
        live_play(),
    );
    assert!(calculator.episode_events().is_empty());

    // The goal frame: live play already over, goal event attributed.
    update(
        &mut calculator,
        2,
        1.1,
        &danger_ball,
        &danger_players,
        FrameEventsState {
            goal_events: vec![goal_event(2, 1.1, true)],
            ..FrameEventsState::default()
        },
        vec![],
        stoppage(),
    );

    let episodes = calculator.episode_events();
    assert_eq!(episodes.len(), 1);
    let episode = &episodes[0];
    assert!(episode.ended_in_goal);
    assert_eq!(episode.end_reason, ThreatEpisodeEndReason::Goal);
    assert_eq!(episode.credited_player, Some(player_id(1)));
    assert_eq!(calculator.goal_records().len(), 1);
}

/// A goal attributed a few frames *after* the stoppage that closed the
/// episode still upgrades it: stoppage-closed episodes wait pending for the
/// goal attribution.
#[test]
fn late_goal_attribution_resolves_pending_stoppage_episode_as_goal() {
    let mut calculator = ExpectedGoalsCalculator::new();
    let (danger_ball, danger_players) = dangerous_state();

    update(
        &mut calculator,
        1,
        1.0,
        &danger_ball,
        &danger_players,
        FrameEventsState::default(),
        vec![],
        live_play(),
    );
    update(
        &mut calculator,
        2,
        1.1,
        &danger_ball,
        &danger_players,
        FrameEventsState::default(),
        vec![],
        stoppage(),
    );
    assert!(
        calculator.episode_events().is_empty(),
        "stoppage-closed episode must stay pending until the goal resolves"
    );

    update(
        &mut calculator,
        3,
        1.5,
        &danger_ball,
        &danger_players,
        FrameEventsState {
            goal_events: vec![goal_event(3, 1.5, true)],
            ..FrameEventsState::default()
        },
        vec![],
        stoppage(),
    );

    let episodes = calculator.episode_events();
    assert_eq!(episodes.len(), 1);
    assert!(episodes[0].ended_in_goal);
    assert_eq!(episodes[0].end_reason, ThreatEpisodeEndReason::Goal);
}

/// A stoppage with no following goal emits the episode as a plain stoppage
/// once the goal grace expires.
#[test]
fn pending_stoppage_episode_without_goal_resolves_as_stoppage() {
    let mut calculator = ExpectedGoalsCalculator::new();
    let (danger_ball, danger_players) = dangerous_state();

    update(
        &mut calculator,
        1,
        1.0,
        &danger_ball,
        &danger_players,
        FrameEventsState::default(),
        vec![],
        live_play(),
    );
    update(
        &mut calculator,
        2,
        1.1,
        &danger_ball,
        &danger_players,
        FrameEventsState::default(),
        vec![],
        stoppage(),
    );
    update(
        &mut calculator,
        3,
        20.0,
        &danger_ball,
        &danger_players,
        FrameEventsState::default(),
        vec![],
        stoppage(),
    );

    let episodes = calculator.episode_events();
    assert_eq!(episodes.len(), 1);
    assert!(!episodes[0].ended_in_goal);
    assert_eq!(episodes[0].end_reason, ThreatEpisodeEndReason::Stoppage);
}

#[test]
fn finish_closes_active_episode_as_replay_end() {
    let mut calculator = ExpectedGoalsCalculator::new();
    let (danger_ball, danger_players) = dangerous_state();

    update(
        &mut calculator,
        1,
        1.0,
        &danger_ball,
        &danger_players,
        FrameEventsState::default(),
        vec![],
        live_play(),
    );
    calculator.finish_calculation().unwrap();

    let episodes = calculator.episode_events();
    assert_eq!(episodes.len(), 1);
    assert_eq!(episodes[0].end_reason, ThreatEpisodeEndReason::ReplayEnd);
}

#[test]
fn sampling_records_two_attacking_normalized_rows_per_sampled_frame() {
    let mut calculator = ExpectedGoalsCalculator::with_config(ExpectedGoalsCalculatorConfig {
        sample_interval_seconds: Some(0.25),
        ..ExpectedGoalsCalculatorConfig::default()
    });
    let (neutral_ball, neutral_players) = neutral_state();

    for step in 0..4 {
        update(
            &mut calculator,
            step + 1,
            1.0 + step as f32 * 0.1,
            &neutral_ball,
            &neutral_players,
            FrameEventsState::default(),
            vec![],
            live_play(),
        );
    }

    // Sampled at t=1.0 and t=1.3 (0.25s cadence over 0.1s frames).
    let samples = calculator.samples();
    assert_eq!(samples.len(), 4);
    assert!(samples[0].is_team_0);
    assert!(!samples[1].is_team_0);
    assert_eq!(samples[0].time, samples[1].time);
    for sample in samples {
        assert!(sample.value > 0.0 && sample.value < 1.0);
    }
}

/// Constant V over N evaluated frames of known dt integrates to exactly
/// N * V * dt / tau (replay-end close: no extra closing-frame contribution).
#[test]
fn episode_xg_is_the_time_integral_of_v_over_the_episode() {
    let mut calculator = ExpectedGoalsCalculator::new();
    let (danger_ball, danger_players) = dangerous_state();

    let frame_count = 3usize;
    for step in 0..frame_count {
        update(
            &mut calculator,
            step + 1,
            1.0 + step as f32 * 0.1,
            &danger_ball,
            &danger_players,
            FrameEventsState::default(),
            vec![],
            live_play(),
        );
    }
    let value = calculator.current_values().unwrap()[0];
    calculator.finish_calculation().unwrap();

    let episodes = calculator.episode_events();
    assert_eq!(episodes.len(), 1);
    let episode = &episodes[0];
    let expected = frame_count as f32 * value * 0.1 / expected_goals_model::THREAT_HORIZON_SECONDS;
    assert!(
        (episode.xg - expected).abs() < 1e-6,
        "episode xg {} != N * V * dt / tau = {}",
        episode.xg,
        expected
    );
    assert!((episode.peak_value - value).abs() < 1e-6);
}

/// The team's full-match integral accumulates on every evaluated live frame,
/// including sub-threshold ones where no episode ever opens.
#[test]
fn team_xg_integral_accumulates_sub_threshold_frames_without_episodes() {
    let mut calculator = ExpectedGoalsCalculator::new();
    let (neutral_ball, neutral_players) = neutral_state();

    let frame_count = 4usize;
    for step in 0..frame_count {
        update(
            &mut calculator,
            step + 1,
            1.0 + step as f32 * 0.1,
            &neutral_ball,
            &neutral_players,
            FrameEventsState::default(),
            vec![],
            live_play(),
        );
    }
    let value = calculator.current_values().unwrap()[0];
    assert!(value < THREAT_EPISODE_THRESHOLD);
    calculator.finish_calculation().unwrap();

    assert!(calculator.episode_events().is_empty());
    let integrals = calculator.team_xg_integrals();
    let expected =
        f64::from(frame_count as f32 * value * 0.1 / expected_goals_model::THREAT_HORIZON_SECONDS);
    assert!(integrals[0] > 0.0);
    assert!((integrals[0] - expected).abs() < 1e-6);
    assert!(integrals[1] > 0.0);

    // The accumulator's team xg is fed from exactly this state.
    let mut accumulator = ExpectedGoalsStatsAccumulator::new();
    accumulator.set_team_xg_integrals(integrals);
    assert!((f64::from(accumulator.team_stats(true).xg) - integrals[0]).abs() < 1e-6);
    assert!((f64::from(accumulator.team_stats(false).xg) - integrals[1]).abs() < 1e-6);
}

#[test]
fn accumulator_folds_touch_deltas_and_episode_xg() {
    let mut accumulator = ExpectedGoalsStatsAccumulator::new();

    accumulator.apply_touch_event(&ThreatTouchEvent {
        time: 1.0,
        frame: 1,
        touch_id: None,
        detection_frame: 1,
        detection_time: 1.0,
        team_is_team_0: true,
        player: Some(player_id(1)),
        value_before: 0.05,
        value_after: 0.30,
    });
    // Negative deltas do not subtract from threat added.
    accumulator.apply_touch_event(&ThreatTouchEvent {
        time: 2.0,
        frame: 2,
        touch_id: None,
        detection_frame: 2,
        detection_time: 2.0,
        team_is_team_0: true,
        player: Some(player_id(1)),
        value_before: 0.30,
        value_after: 0.10,
    });
    accumulator.apply_episode_event(&ThreatEpisodeEvent {
        start_time: 1.0,
        start_frame: 1,
        end_time: 2.0,
        end_frame: 2,
        team_is_team_0: true,
        xg: 0.4,
        peak_value: 0.6,
        credited_player: Some(player_id(1)),
        ended_in_goal: true,
        end_reason: ThreatEpisodeEndReason::Goal,
    });
    // Team-only credit still advances the team's episode counters.
    accumulator.apply_episode_event(&ThreatEpisodeEvent {
        start_time: 3.0,
        start_frame: 3,
        end_time: 4.0,
        end_frame: 4,
        team_is_team_0: true,
        xg: 0.2,
        peak_value: 0.3,
        credited_player: None,
        ended_in_goal: false,
        end_reason: ThreatEpisodeEndReason::ValueDropped,
    });
    // Team xG comes from the full-match integral, not the episode sum; the
    // gap (1.0 vs the 0.6 of episode xg) is the diffuse sub-threshold threat
    // that is never attributed to any player.
    accumulator.set_team_xg_integrals([1.0, 0.25]);

    let player_stats = accumulator.player_stats().get(&player_id(1)).unwrap();
    assert!((player_stats.threat_added - 0.25).abs() < 1e-6);
    assert!((player_stats.xg - 0.4).abs() < 1e-6);
    assert_eq!(player_stats.credited_episode_count, 1);
    assert_eq!(player_stats.credited_goal_episode_count, 1);

    let team = accumulator.team_stats(true);
    assert!((team.xg - 1.0).abs() < 1e-6);
    assert_eq!(team.episode_count, 2);
    assert_eq!(team.goal_episode_count, 1);
    let other_team = accumulator.team_stats(false);
    assert_eq!(other_team.episode_count, 0);
    assert!((other_team.xg - 0.25).abs() < 1e-6);
}
