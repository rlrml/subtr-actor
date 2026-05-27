use super::*;

#[test]
fn process_frame_uses_explicit_live_play_state_for_analysis_graph() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let frame = SaLiveFrame {
        frame_number: 7,
        time: 1.5,
        dt: 0.016,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        live_play: 0,
        has_live_play: 1,
        ..SaLiveFrame::default()
    };

    let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

    assert_eq!(status, 0);
    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let live_play = engine_ref
        .graph
        .state::<LivePlayState>()
        .expect("full analysis graph should expose live play state");
    assert_eq!(live_play.gameplay_phase, GameplayPhase::Unknown);
    assert!(!live_play.is_live_play);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn process_frame_derives_live_play_when_not_explicit() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let frame = SaLiveFrame {
        frame_number: 7,
        time: 1.5,
        dt: 0.016,
        ball_has_been_hit: 0,
        has_ball_has_been_hit: 1,
        live_play: 1,
        has_live_play: 0,
        ..SaLiveFrame::default()
    };

    let status = unsafe { subtr_actor_bakkesmod_process_frame(engine, &frame) };

    assert_eq!(status, 0);
    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let live_play = engine_ref
        .graph
        .state::<LivePlayState>()
        .expect("full analysis graph should expose live play state");
    assert_eq!(
        live_play.gameplay_phase,
        GameplayPhase::KickoffWaitingForTouch
    );
    assert!(!live_play.is_live_play);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn process_frame_treats_sampled_game_state_as_replay_phase_signal() {
    const GAME_STATE_KICKOFF_COUNTDOWN: i32 = 55;
    const GAME_STATE_GOAL_SCORED_REPLAY: i32 = 86;

    let engine = subtr_actor_bakkesmod_engine_create();
    let kickoff_frame = SaLiveFrame {
        frame_number: 7,
        time: 1.5,
        dt: 0.016,
        game_state: GAME_STATE_KICKOFF_COUNTDOWN,
        has_game_state: 1,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        ..SaLiveFrame::default()
    };

    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &kickoff_frame) },
        0
    );
    let engine_ref = unsafe { engine.as_ref().expect("engine should be valid") };
    let live_play = engine_ref
        .graph
        .state::<LivePlayState>()
        .expect("full analysis graph should expose live play state");
    assert_eq!(live_play.gameplay_phase, GameplayPhase::KickoffCountdown);
    assert!(!live_play.is_live_play);
    let gameplay = engine_ref
        .graph
        .state::<GameplayState>()
        .expect("full analysis graph should expose gameplay state");
    assert_eq!(gameplay.game_state, Some(GAME_STATE_KICKOFF_COUNTDOWN));

    let replay_frame = SaLiveFrame {
        frame_number: 8,
        time: 1.6,
        dt: 0.016,
        game_state: GAME_STATE_GOAL_SCORED_REPLAY,
        has_game_state: 1,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        ..SaLiveFrame::default()
    };
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_process_frame(engine, &replay_frame) },
        0
    );
    let live_play = engine_ref
        .graph
        .state::<LivePlayState>()
        .expect("full analysis graph should expose live play state");
    assert_eq!(live_play.gameplay_phase, GameplayPhase::PostGoal);
    assert!(!live_play.is_live_play);
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}
