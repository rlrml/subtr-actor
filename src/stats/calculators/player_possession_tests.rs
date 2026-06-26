use super::*;

const DT: f32 = 0.25;

fn player_id(id: u64) -> PlayerId {
    boxcars::RemoteId::Steam(id)
}

fn rigid_body(position: glam::Vec3) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: boxcars::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
        angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
    }
}

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: DT,
        seconds_remaining: None,
    }
}

fn ball_at(y: f32, z: f32) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, y, z)),
    })
}

fn player_sample(id: u64, is_team_0: bool, position: glam::Vec3) -> PlayerSample {
    PlayerSample {
        player_id: player_id(id),
        is_team_0,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(position)),
        boost_amount: None,
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

fn players_at(position: glam::Vec3) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![
            player_sample(1, true, position),
            player_sample(2, false, glam::Vec3::new(1000.0, -3000.0, 17.0)),
        ],
    }
}

fn players_with(team_zero: glam::Vec3, team_one: glam::Vec3) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![
            player_sample(1, true, team_zero),
            player_sample(2, false, team_one),
        ],
    }
}

fn possession(player: Option<u64>, team_is_team_0: Option<bool>) -> PossessionState {
    PossessionState {
        active_team_before_sample: team_is_team_0,
        current_team_is_team_0: team_is_team_0,
        active_player_before_sample: player.map(player_id),
        current_player: player.map(player_id),
        ..Default::default()
    }
}

fn touch(frame: usize, time: f32, id: u64, position: glam::Vec3) -> TouchEvent {
    TouchEvent {
        touch_id: None,
        time,
        frame,
        team_is_team_0: true,
        player: Some(player_id(id)),
        player_position: Some(glam_to_vec(&position)),
        closest_approach_distance: Some(0.0),
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
        dodge_contact: false,
    }
}

fn touch_state(events: Vec<TouchEvent>) -> TouchState {
    TouchState {
        touch_events: events,
        last_touch: None,
        last_touch_player: None,
        last_touch_team_is_team_0: None,
    }
}

fn update(
    calculator: &mut PlayerPossessionCalculator,
    frame_number: usize,
    time: f32,
    ball: &BallFrameState,
    players: &PlayerFrameState,
    state: &PossessionState,
    touches: Vec<TouchEvent>,
) {
    calculator
        .update(
            &frame(frame_number, time),
            ball,
            players,
            state,
            &touch_state(touches),
            &LivePlayState::active_play(),
        )
        .expect("update succeeds");
}

#[test]
fn emits_single_enriched_span_for_continuous_possession() {
    let mut calculator = PlayerPossessionCalculator::new();
    let players = players_at(glam::Vec3::new(0.0, 0.0, 17.0));
    let state = possession(Some(1), Some(true));

    // Ball advances toward the team-zero goal (+y) by 100 uu per frame, well
    // off the player so no carry sample registers.
    for step in 0..8 {
        let time = step as f32 * DT;
        let ball = ball_at(step as f32 * 100.0, 1000.0);
        // Two distinct touches so the span qualifies as possession; they are
        // grounded and well clear of the carry gap, so enrichment is unchanged.
        let touches = if step == 0 || step == 4 {
            vec![touch(step, time, 1, glam::Vec3::new(0.0, 0.0, 17.0))]
        } else {
            vec![]
        };
        update(
            &mut calculator,
            step,
            time,
            &ball,
            &players,
            &state,
            touches,
        );
    }
    calculator.finish();

    let events = calculator.events();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.player_id, player_id(1));
    assert!(event.is_team_0);
    assert_eq!(event.touch_count, 2);
    assert_eq!(event.aerial_touch_count, 0);
    assert_eq!(event.wall_touch_count, 0);
    assert!((event.duration - 8.0 * DT).abs() < 1e-4);
    // 7 inter-frame deltas of 100 uu each land inside the span.
    assert!((event.advance_distance - 700.0).abs() < 1.0);
    assert_eq!(event.retreat_distance, 0.0);
    assert_eq!(event.carry_time, 0.0);
}

#[test]
fn merges_spans_across_a_short_neutral_gap() {
    let mut calculator = PlayerPossessionCalculator::new();
    let players = players_at(glam::Vec3::new(0.0, 0.0, 17.0));
    let ball = ball_at(0.0, 1000.0);
    let owned = possession(Some(1), Some(true));
    let neutral = possession(None, None);

    // One touch before the gap and one after: the merged span has the two
    // distinct touches it needs to qualify as possession.
    for step in 0..4 {
        let touches = if step == 0 {
            vec![touch(
                step,
                step as f32 * DT,
                1,
                glam::Vec3::new(0.0, 0.0, 17.0),
            )]
        } else {
            vec![]
        };
        update(
            &mut calculator,
            step,
            step as f32 * DT,
            &ball,
            &players,
            &owned,
            touches,
        );
    }
    // Contested window shorter than the merge gap.
    for step in 4..8 {
        update(
            &mut calculator,
            step,
            step as f32 * DT,
            &ball,
            &players,
            &neutral,
            vec![],
        );
    }
    for step in 8..12 {
        let touches = if step == 8 {
            vec![touch(
                step,
                step as f32 * DT,
                1,
                glam::Vec3::new(0.0, 0.0, 17.0),
            )]
        } else {
            vec![]
        };
        update(
            &mut calculator,
            step,
            step as f32 * DT,
            &ball,
            &players,
            &owned,
            touches,
        );
    }
    calculator.finish();

    let events = calculator.events();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    // The contested gap is excluded from possessed duration.
    assert!((event.duration - 8.0 * DT).abs() < 1e-4);
    assert_eq!(event.start_frame, 0);
    assert_eq!(event.end_frame, 11);
}

#[test]
fn turnover_to_another_player_splits_spans() {
    let mut calculator = PlayerPossessionCalculator::new();
    // Both players sit next to the ball so the loose-distance bound never
    // demotes whoever currently holds it.
    let players = players_with(
        glam::Vec3::new(0.0, 0.0, 17.0),
        glam::Vec3::new(0.0, 0.0, 17.0),
    );
    let ball = ball_at(0.0, 1000.0);

    for step in 0..4 {
        let touches = if step == 0 || step == 2 {
            vec![touch(
                step,
                step as f32 * DT,
                1,
                glam::Vec3::new(0.0, 0.0, 17.0),
            )]
        } else {
            vec![]
        };
        update(
            &mut calculator,
            step,
            step as f32 * DT,
            &ball,
            &players,
            &possession(Some(1), Some(true)),
            touches,
        );
    }
    for step in 4..8 {
        let touches = if step == 4 || step == 6 {
            vec![touch(
                step,
                step as f32 * DT,
                2,
                glam::Vec3::new(0.0, 0.0, 17.0),
            )]
        } else {
            vec![]
        };
        update(
            &mut calculator,
            step,
            step as f32 * DT,
            &ball,
            &players,
            &possession(Some(2), Some(false)),
            touches,
        );
    }
    calculator.finish();

    let events = calculator.events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].player_id, player_id(1));
    assert!(events[0].is_team_0);
    assert_eq!(events[1].player_id, player_id(2));
    assert!(!events[1].is_team_0);
}

#[test]
fn expired_gap_finalizes_the_suspended_span() {
    let mut calculator = PlayerPossessionCalculator::new();
    let players = players_at(glam::Vec3::new(0.0, 0.0, 17.0));
    let ball = ball_at(0.0, 1000.0);
    let owned = possession(Some(1), Some(true));
    let neutral = possession(None, None);

    for step in 0..4 {
        let touches = if step == 0 || step == 2 {
            vec![touch(
                step,
                step as f32 * DT,
                1,
                glam::Vec3::new(0.0, 0.0, 17.0),
            )]
        } else {
            vec![]
        };
        update(
            &mut calculator,
            step,
            step as f32 * DT,
            &ball,
            &players,
            &owned,
            touches,
        );
    }
    // Neutral stretch much longer than the merge gap.
    for step in 4..20 {
        update(
            &mut calculator,
            step,
            step as f32 * DT,
            &ball,
            &players,
            &neutral,
            vec![],
        );
    }
    for step in 20..24 {
        let touches = if step == 20 || step == 22 {
            vec![touch(
                step,
                step as f32 * DT,
                1,
                glam::Vec3::new(0.0, 0.0, 17.0),
            )]
        } else {
            vec![]
        };
        update(
            &mut calculator,
            step,
            step as f32 * DT,
            &ball,
            &players,
            &owned,
            touches,
        );
    }
    calculator.finish();

    let events = calculator.events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].end_frame, 3);
    // Span starts are backdated one frame to cover the opening frame's dt.
    assert_eq!(events[1].start_frame, 19);
}

#[test]
fn accumulates_carry_time_when_ball_rides_the_player() {
    let mut calculator = PlayerPossessionCalculator::new();
    let player_position = glam::Vec3::new(0.0, 0.0, 17.0);
    let players = players_at(player_position);
    let state = possession(Some(1), Some(true));
    // Ball sitting just above the car: inside the grounded-carry gap bounds.
    let ball = ball_at(0.0, 120.0);

    for step in 0..8 {
        let touches = if step == 0 || step == 4 {
            vec![touch(
                step,
                step as f32 * DT,
                1,
                glam::Vec3::new(0.0, 0.0, 17.0),
            )]
        } else {
            vec![]
        };
        update(
            &mut calculator,
            step,
            step as f32 * DT,
            &ball,
            &players,
            &state,
            touches,
        );
    }
    calculator.finish();

    let events = calculator.events();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert!((event.carry_time - 8.0 * DT).abs() < 1e-4);
    assert_eq!(event.carry_count, 1);
    assert_eq!(event.air_dribble_time, 0.0);
}

#[test]
fn labels_sustained_control_using_controlled_play_criteria() {
    let mut calculator = PlayerPossessionCalculator::new();
    // Ball hovers 300uu over the player: inside the 700uu proximity radius,
    // so close time accrues every frame.
    let players = players_at(glam::Vec3::new(0.0, 0.0, 17.0));
    let ball = ball_at(0.0, 300.0);
    let state = possession(Some(1), Some(true));

    // Two distinct touches 1.5s apart over a 2s possession with constant
    // proximity: meets every controlled-play criterion.
    for step in 0..8 {
        let time = step as f32 * DT;
        let touches = if step == 0 || step == 6 {
            vec![touch(step, time, 1, glam::Vec3::new(0.0, 0.0, 17.0))]
        } else {
            vec![]
        };
        update(
            &mut calculator,
            step,
            time,
            &ball,
            &players,
            &state,
            touches,
        );
    }
    calculator.finish();

    let events = calculator.events();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert!(event.close_time > 0.0, "proximity time should accrue");
    assert!(
        event.sustained_control,
        "two spaced touches with sustained proximity qualify as controlled play"
    );
}

#[test]
fn single_touch_span_is_dropped() {
    // A lone touch the player never follows up on — the canonical kickoff poke
    // that stayed "possession" only because nobody else touched until the
    // opponent did — is not possession and must not be emitted at all.
    let mut calculator = PlayerPossessionCalculator::new();
    let players = players_at(glam::Vec3::new(0.0, 0.0, 17.0));
    let ball = ball_at(0.0, 1000.0);
    let state = possession(Some(1), Some(true));

    for step in 0..8 {
        let time = step as f32 * DT;
        let touches = if step == 0 {
            vec![touch(step, time, 1, glam::Vec3::new(0.0, 0.0, 17.0))]
        } else {
            vec![]
        };
        update(
            &mut calculator,
            step,
            time,
            &ball,
            &players,
            &state,
            touches,
        );
    }
    calculator.finish();

    assert!(
        calculator.events().is_empty(),
        "a single-touch span never qualifies as possession"
    );
}

#[test]
fn ball_drifting_far_ends_the_span_without_crediting_the_drift() {
    // The holder makes two real touches, then the ball rockets away and they
    // never play it again. The far frames must not be credited, and once the
    // drift outlasts the merge gap the span finalizes on its own. `current_player`
    // stays sticky on the holder throughout — only the loose-distance bound ends
    // the span.
    let mut calculator = PlayerPossessionCalculator::new();
    let players = players_at(glam::Vec3::new(0.0, 0.0, 17.0));
    let state = possession(Some(1), Some(true));

    for step in 0..4 {
        let time = step as f32 * DT;
        let ball = ball_at(0.0, 200.0);
        let touches = if step == 0 || step == 2 {
            vec![touch(step, time, 1, glam::Vec3::new(0.0, 0.0, 17.0))]
        } else {
            vec![]
        };
        update(
            &mut calculator,
            step,
            time,
            &ball,
            &players,
            &state,
            touches,
        );
    }
    // Ball well beyond the loose-distance bound; the player stays put.
    for step in 4..20 {
        let time = step as f32 * DT;
        let ball = ball_at(6000.0, 1000.0);
        update(&mut calculator, step, time, &ball, &players, &state, vec![]);
    }
    calculator.finish();

    let events = calculator.events();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.touch_count, 2);
    // Only the near frames (0..=3) are credited; the drift is excluded.
    assert_eq!(event.end_frame, 3);
    assert!((event.duration - 4.0 * DT).abs() < 1e-4);
}

#[test]
fn classifies_aerial_and_wall_touches() {
    let mut calculator = PlayerPossessionCalculator::new();
    let players = players_at(glam::Vec3::new(0.0, 0.0, 17.0));
    let ball = ball_at(0.0, 1000.0);
    let state = possession(Some(1), Some(true));

    let touches = [
        touch(0, 0.0, 1, glam::Vec3::new(0.0, 0.0, 17.0)),
        touch(1, 1.0, 1, glam::Vec3::new(0.0, 0.0, 800.0)),
        touch(2, 2.0, 1, glam::Vec3::new(4000.0, 0.0, 800.0)),
    ];
    for (step, touch) in touches.into_iter().enumerate() {
        update(
            &mut calculator,
            step,
            step as f32,
            &ball,
            &players,
            &state,
            vec![touch],
        );
    }
    calculator.finish();

    let events = calculator.events();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.touch_count, 3);
    assert_eq!(event.aerial_touch_count, 1);
    assert_eq!(event.wall_touch_count, 1);
}
