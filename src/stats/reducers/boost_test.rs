use super::*;
use boxcars::RemoteId;

fn player(player_id: u64, boost_amount: f32) -> PlayerSample {
    PlayerSample {
        player_id: RemoteId::Steam(player_id),
        is_team_0: true,
        rigid_body: None,
        boost_amount: Some(boost_amount),
        last_boost_amount: None,
        boost_active: false,
        dodge_active: false,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

fn sample(
    frame_number: usize,
    time: f32,
    boost_amount: f32,
    game_state: Option<i32>,
    ball_has_been_hit: Option<bool>,
    kickoff_countdown_time: Option<i32>,
) -> StatsSample {
    StatsSample {
        frame_number,
        time,
        dt: 1.0,
        seconds_remaining: None,
        game_state,
        ball_has_been_hit,
        kickoff_countdown_time,
        team_zero_score: Some(0),
        team_one_score: Some(0),
        possession_team_is_team_0: None,
        scored_on_team_is_team_0: None,
        current_in_game_team_player_counts: Some([1, 0]),
        ball: None,
        players: vec![player(1, boost_amount)],
        active_demos: Vec::new(),
        demo_events: Vec::new(),
        boost_pad_events: Vec::new(),
        touch_events: Vec::new(),
        dodge_refreshed_events: Vec::new(),
        player_stat_events: Vec::new(),
        goal_events: Vec::new(),
    }
}

#[test]
fn boost_levels_skip_pre_kickoff_dead_time() {
    let mut reducer = BoostReducer::new();
    let player_id = RemoteId::Steam(1);
    reducer.initial_respawn_awarded.insert(player_id.clone());
    reducer.kickoff_respawn_awarded.insert(player_id.clone());

    reducer
        .on_sample(&sample(
            0,
            0.0,
            BOOST_MAX_AMOUNT,
            Some(GAME_STATE_KICKOFF_COUNTDOWN),
            Some(false),
            Some(1),
        ))
        .unwrap();
    reducer
        .on_sample(&sample(1, 1.0, 0.0, None, Some(true), None))
        .unwrap();
    reducer
        .on_sample(&sample(2, 2.0, 0.0, None, Some(true), None))
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.tracked_time, 2.0);
    assert_eq!(stats.average_boost_amount(), 0.0);
}

#[test]
fn boost_levels_still_track_when_replay_starts_live() {
    let mut reducer = BoostReducer::new();
    let player_id = RemoteId::Steam(1);
    reducer.initial_respawn_awarded.insert(player_id.clone());

    reducer
        .on_sample(&sample(0, 0.0, 42.0, None, Some(true), None))
        .unwrap();
    reducer
        .on_sample(&sample(1, 1.0, 42.0, None, Some(true), None))
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.tracked_time, 2.0);
    assert_eq!(stats.average_boost_amount(), 42.0);
}
