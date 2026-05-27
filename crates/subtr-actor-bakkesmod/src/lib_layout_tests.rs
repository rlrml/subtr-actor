use super::*;

macro_rules! assert_layout {
    ($ty:ty, size = $size:expr, align = $align:expr) => {
        assert_eq!(
            std::mem::size_of::<$ty>(),
            $size,
            "size of {}",
            stringify!($ty)
        );
        assert_eq!(
            std::mem::align_of::<$ty>(),
            $align,
            "alignment of {}",
            stringify!($ty)
        );
    };
}

macro_rules! assert_offset {
    ($ty:ty, $field:tt, $offset:expr) => {
        assert_eq!(
            std::mem::offset_of!($ty, $field),
            $offset,
            "offset of {}.{}",
            stringify!($ty),
            stringify!($field)
        );
    };
}

#[test]
fn rust_event_abi_layout_matches_plugin_header_expectations() {
    assert_layout!(SaBoostPadEventKind, size = 4, align = 4);
    assert_layout!(SaPlayerStatEventKind, size = 4, align = 4);
    assert_layout!(SaMechanicKind, size = 4, align = 4);
    assert_layout!(SaTeamEventKind, size = 4, align = 4);
    assert_layout!(SaGoalBuildupKind, size = 4, align = 4);

    assert_layout!(SaVec3, size = 12, align = 4);
    assert_offset!(SaVec3, x, 0);
    assert_offset!(SaVec3, y, 4);
    assert_offset!(SaVec3, z, 8);

    assert_layout!(SaQuat, size = 16, align = 4);
    assert_offset!(SaQuat, x, 0);
    assert_offset!(SaQuat, y, 4);
    assert_offset!(SaQuat, z, 8);
    assert_offset!(SaQuat, w, 12);

    assert_layout!(SaRigidBody, size = 56, align = 4);
    assert_offset!(SaRigidBody, location, 0);
    assert_offset!(SaRigidBody, rotation, 12);
    assert_offset!(SaRigidBody, linear_velocity, 28);
    assert_offset!(SaRigidBody, angular_velocity, 40);
    assert_offset!(SaRigidBody, has_linear_velocity, 52);
    assert_offset!(SaRigidBody, has_angular_velocity, 53);
    assert_offset!(SaRigidBody, sleeping, 54);

    assert_layout!(SaPlayerFrame, size = 112, align = 8);
    assert_offset!(SaPlayerFrame, player_index, 0);
    assert_offset!(SaPlayerFrame, player_name, 8);
    assert_offset!(SaPlayerFrame, is_team_0, 16);
    assert_offset!(SaPlayerFrame, has_rigid_body, 17);
    assert_offset!(SaPlayerFrame, rigid_body, 20);
    assert_offset!(SaPlayerFrame, boost_amount, 76);
    assert_offset!(SaPlayerFrame, last_boost_amount, 80);
    assert_offset!(SaPlayerFrame, boost_active, 84);
    assert_offset!(SaPlayerFrame, jump_active, 85);
    assert_offset!(SaPlayerFrame, double_jump_active, 86);
    assert_offset!(SaPlayerFrame, dodge_active, 87);
    assert_offset!(SaPlayerFrame, powerslide_active, 88);
    assert_offset!(SaPlayerFrame, has_match_stats, 89);
    assert_offset!(SaPlayerFrame, match_goals, 92);
    assert_offset!(SaPlayerFrame, match_assists, 96);
    assert_offset!(SaPlayerFrame, match_saves, 100);
    assert_offset!(SaPlayerFrame, match_shots, 104);
    assert_offset!(SaPlayerFrame, match_score, 108);

    assert_layout!(SaEventTiming, size = 24, align = 8);
    assert_offset!(SaEventTiming, frame_number, 0);
    assert_offset!(SaEventTiming, time, 8);
    assert_offset!(SaEventTiming, seconds_remaining, 12);
    assert_offset!(SaEventTiming, has_timing, 16);
    assert_offset!(SaEventTiming, has_seconds_remaining, 17);

    assert_layout!(SaTouchEvent, size = 40, align = 8);
    assert_offset!(SaTouchEvent, timing, 0);
    assert_offset!(SaTouchEvent, player_index, 24);
    assert_offset!(SaTouchEvent, has_player, 28);
    assert_offset!(SaTouchEvent, is_team_0, 29);
    assert_offset!(SaTouchEvent, closest_approach_distance, 32);
    assert_offset!(SaTouchEvent, has_closest_approach_distance, 36);

    assert_layout!(SaDodgeRefreshedEvent, size = 40, align = 8);
    assert_offset!(SaDodgeRefreshedEvent, timing, 0);
    assert_offset!(SaDodgeRefreshedEvent, player_index, 24);
    assert_offset!(SaDodgeRefreshedEvent, is_team_0, 28);
    assert_offset!(SaDodgeRefreshedEvent, counter_value, 32);

    assert_layout!(SaBoostPadEvent, size = 48, align = 8);
    assert_offset!(SaBoostPadEvent, timing, 0);
    assert_offset!(SaBoostPadEvent, pad_id, 24);
    assert_offset!(SaBoostPadEvent, kind, 28);
    assert_offset!(SaBoostPadEvent, sequence, 32);
    assert_offset!(SaBoostPadEvent, player_index, 36);
    assert_offset!(SaBoostPadEvent, has_player, 40);

    assert_layout!(SaGoalEvent, size = 56, align = 8);
    assert_offset!(SaGoalEvent, timing, 0);
    assert_offset!(SaGoalEvent, scoring_team_is_team_0, 24);
    assert_offset!(SaGoalEvent, player_index, 28);
    assert_offset!(SaGoalEvent, has_player, 32);
    assert_offset!(SaGoalEvent, team_zero_score, 36);
    assert_offset!(SaGoalEvent, has_team_zero_score, 40);
    assert_offset!(SaGoalEvent, team_one_score, 44);
    assert_offset!(SaGoalEvent, has_team_one_score, 48);

    assert_layout!(SaPlayerStatEvent, size = 160, align = 8);
    assert_offset!(SaPlayerStatEvent, timing, 0);
    assert_offset!(SaPlayerStatEvent, player_index, 24);
    assert_offset!(SaPlayerStatEvent, is_team_0, 28);
    assert_offset!(SaPlayerStatEvent, kind, 32);
    assert_offset!(SaPlayerStatEvent, has_shot_ball, 36);
    assert_offset!(SaPlayerStatEvent, shot_ball, 40);
    assert_offset!(SaPlayerStatEvent, has_shot_player, 96);
    assert_offset!(SaPlayerStatEvent, shot_player, 100);

    assert_layout!(SaDemolishEvent, size = 72, align = 8);
    assert_offset!(SaDemolishEvent, timing, 0);
    assert_offset!(SaDemolishEvent, attacker_index, 24);
    assert_offset!(SaDemolishEvent, victim_index, 28);
    assert_offset!(SaDemolishEvent, attacker_velocity, 32);
    assert_offset!(SaDemolishEvent, victim_velocity, 44);
    assert_offset!(SaDemolishEvent, victim_location, 56);
    assert_offset!(SaDemolishEvent, active_duration_seconds, 68);

    assert_layout!(SaLiveFrame, size = 232, align = 8);
    assert_offset!(SaLiveFrame, frame_number, 0);
    assert_offset!(SaLiveFrame, time, 8);
    assert_offset!(SaLiveFrame, dt, 12);
    assert_offset!(SaLiveFrame, seconds_remaining, 16);
    assert_offset!(SaLiveFrame, has_seconds_remaining, 20);
    assert_offset!(SaLiveFrame, game_state, 24);
    assert_offset!(SaLiveFrame, has_game_state, 28);
    assert_offset!(SaLiveFrame, kickoff_countdown_time, 32);
    assert_offset!(SaLiveFrame, has_kickoff_countdown_time, 36);
    assert_offset!(SaLiveFrame, ball_has_been_hit, 37);
    assert_offset!(SaLiveFrame, has_ball_has_been_hit, 38);
    assert_offset!(SaLiveFrame, team_zero_score, 40);
    assert_offset!(SaLiveFrame, has_team_zero_score, 44);
    assert_offset!(SaLiveFrame, team_one_score, 48);
    assert_offset!(SaLiveFrame, has_team_one_score, 52);
    assert_offset!(SaLiveFrame, possession_team_is_team_0, 53);
    assert_offset!(SaLiveFrame, has_possession_team, 54);
    assert_offset!(SaLiveFrame, scored_on_team_is_team_0, 55);
    assert_offset!(SaLiveFrame, has_scored_on_team, 56);
    assert_offset!(SaLiveFrame, live_play, 57);
    assert_offset!(SaLiveFrame, has_live_play, 58);
    assert_offset!(SaLiveFrame, has_ball, 59);
    assert_offset!(SaLiveFrame, ball, 60);
    assert_offset!(SaLiveFrame, players, 120);
    assert_offset!(SaLiveFrame, player_count, 128);
    assert_offset!(SaLiveFrame, touches, 136);
    assert_offset!(SaLiveFrame, touch_count, 144);
    assert_offset!(SaLiveFrame, dodge_refreshes, 152);
    assert_offset!(SaLiveFrame, dodge_refresh_count, 160);
    assert_offset!(SaLiveFrame, boost_pad_events, 168);
    assert_offset!(SaLiveFrame, boost_pad_event_count, 176);
    assert_offset!(SaLiveFrame, goals, 184);
    assert_offset!(SaLiveFrame, goal_count, 192);
    assert_offset!(SaLiveFrame, player_stat_events, 200);
    assert_offset!(SaLiveFrame, player_stat_event_count, 208);
    assert_offset!(SaLiveFrame, demolishes, 216);
    assert_offset!(SaLiveFrame, demolish_count, 224);

    assert_layout!(SaMechanicEvent, size = 32, align = 8);
    assert_offset!(SaMechanicEvent, kind, 0);
    assert_offset!(SaMechanicEvent, player_index, 4);
    assert_offset!(SaMechanicEvent, is_team_0, 8);
    assert_offset!(SaMechanicEvent, frame_number, 16);
    assert_offset!(SaMechanicEvent, time, 24);
    assert_offset!(SaMechanicEvent, confidence, 28);

    assert_layout!(SaTeamEvent, size = 48, align = 8);
    assert_offset!(SaTeamEvent, kind, 0);
    assert_offset!(SaTeamEvent, is_team_0, 4);
    assert_offset!(SaTeamEvent, start_frame, 8);
    assert_offset!(SaTeamEvent, end_frame, 16);
    assert_offset!(SaTeamEvent, start_time, 24);
    assert_offset!(SaTeamEvent, end_time, 28);
    assert_offset!(SaTeamEvent, attackers, 32);
    assert_offset!(SaTeamEvent, defenders, 36);
    assert_offset!(SaTeamEvent, confidence, 40);

    assert_layout!(SaGoalContextEvent, size = 64, align = 8);
    assert_offset!(SaGoalContextEvent, frame_number, 0);
    assert_offset!(SaGoalContextEvent, time, 8);
    assert_offset!(SaGoalContextEvent, scoring_team_is_team_0, 12);
    assert_offset!(SaGoalContextEvent, has_scorer, 13);
    assert_offset!(SaGoalContextEvent, scorer_index, 16);
    assert_offset!(SaGoalContextEvent, has_scoring_team_most_back_player, 20);
    assert_offset!(SaGoalContextEvent, scoring_team_most_back_player_index, 24);
    assert_offset!(SaGoalContextEvent, has_defending_team_most_back_player, 28);
    assert_offset!(
        SaGoalContextEvent,
        defending_team_most_back_player_index,
        32
    );
    assert_offset!(SaGoalContextEvent, has_ball_position, 36);
    assert_offset!(SaGoalContextEvent, ball_position, 40);
    assert_offset!(SaGoalContextEvent, has_ball_air_time_before_goal, 52);
    assert_offset!(SaGoalContextEvent, ball_air_time_before_goal, 56);
    assert_offset!(SaGoalContextEvent, goal_buildup, 60);
}
