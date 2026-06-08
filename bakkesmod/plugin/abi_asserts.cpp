// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
namespace {

#if defined(__clang__) || defined(__GNUC__)
#define SA_OFFSETOF(type, member) __builtin_offsetof(type, member)
#else
#define SA_OFFSETOF(type, member) offsetof(type, member)
#endif

static_assert(sizeof(SaBoostPadEventKind) == 4);
static_assert(sizeof(SaPlayerStatEventKind) == 4);

static_assert(std::is_standard_layout_v<SaEventTiming>);
static_assert(sizeof(SaEventTiming) == 24);
static_assert(alignof(SaEventTiming) == 8);
static_assert(SA_OFFSETOF(SaEventTiming, frame_number) == 0);
static_assert(SA_OFFSETOF(SaEventTiming, time) == 8);
static_assert(SA_OFFSETOF(SaEventTiming, seconds_remaining) == 12);
static_assert(SA_OFFSETOF(SaEventTiming, has_timing) == 16);
static_assert(SA_OFFSETOF(SaEventTiming, has_seconds_remaining) == 17);
static_assert(sizeof(SaMechanicKind) == 4);
static_assert(sizeof(SaTeamEventKind) == 4);
static_assert(sizeof(SaGoalBuildupKind) == 4);

static_assert(std::is_standard_layout_v<SaVec3>);
static_assert(sizeof(SaVec3) == 12);
static_assert(alignof(SaVec3) == 4);
static_assert(SA_OFFSETOF(SaVec3, x) == 0);
static_assert(SA_OFFSETOF(SaVec3, y) == 4);
static_assert(SA_OFFSETOF(SaVec3, z) == 8);

static_assert(std::is_standard_layout_v<SaQuat>);
static_assert(sizeof(SaQuat) == 16);
static_assert(alignof(SaQuat) == 4);
static_assert(SA_OFFSETOF(SaQuat, x) == 0);
static_assert(SA_OFFSETOF(SaQuat, y) == 4);
static_assert(SA_OFFSETOF(SaQuat, z) == 8);
static_assert(SA_OFFSETOF(SaQuat, w) == 12);

static_assert(std::is_standard_layout_v<SaRigidBody>);
static_assert(sizeof(SaRigidBody) == 56);
static_assert(alignof(SaRigidBody) == 4);
static_assert(SA_OFFSETOF(SaRigidBody, location) == 0);
static_assert(SA_OFFSETOF(SaRigidBody, rotation) == 12);
static_assert(SA_OFFSETOF(SaRigidBody, linear_velocity) == 28);
static_assert(SA_OFFSETOF(SaRigidBody, angular_velocity) == 40);
static_assert(SA_OFFSETOF(SaRigidBody, has_linear_velocity) == 52);
static_assert(SA_OFFSETOF(SaRigidBody, has_angular_velocity) == 53);
static_assert(SA_OFFSETOF(SaRigidBody, sleeping) == 54);

static_assert(std::is_standard_layout_v<SaPlayerFrame>);
static_assert(sizeof(SaPlayerFrame) == 120);
static_assert(alignof(SaPlayerFrame) == 8);
static_assert(SA_OFFSETOF(SaPlayerFrame, player_index) == 0);
static_assert(SA_OFFSETOF(SaPlayerFrame, player_name) == 8);
static_assert(SA_OFFSETOF(SaPlayerFrame, is_team_0) == 16);
static_assert(SA_OFFSETOF(SaPlayerFrame, has_rigid_body) == 17);
static_assert(SA_OFFSETOF(SaPlayerFrame, rigid_body) == 20);
static_assert(SA_OFFSETOF(SaPlayerFrame, boost_amount) == 76);
static_assert(SA_OFFSETOF(SaPlayerFrame, last_boost_amount) == 80);
static_assert(SA_OFFSETOF(SaPlayerFrame, boost_active) == 84);
static_assert(SA_OFFSETOF(SaPlayerFrame, jump_active) == 85);
static_assert(SA_OFFSETOF(SaPlayerFrame, double_jump_active) == 86);
static_assert(SA_OFFSETOF(SaPlayerFrame, dodge_active) == 87);
static_assert(SA_OFFSETOF(SaPlayerFrame, powerslide_active) == 88);
static_assert(SA_OFFSETOF(SaPlayerFrame, car_body_id) == 92);
static_assert(SA_OFFSETOF(SaPlayerFrame, has_car_body_id) == 96);
static_assert(SA_OFFSETOF(SaPlayerFrame, has_match_stats) == 97);
static_assert(SA_OFFSETOF(SaPlayerFrame, match_goals) == 100);
static_assert(SA_OFFSETOF(SaPlayerFrame, match_assists) == 104);
static_assert(SA_OFFSETOF(SaPlayerFrame, match_saves) == 108);
static_assert(SA_OFFSETOF(SaPlayerFrame, match_shots) == 112);
static_assert(SA_OFFSETOF(SaPlayerFrame, match_score) == 116);

static_assert(std::is_standard_layout_v<SaTouchEvent>);
static_assert(sizeof(SaTouchEvent) == 40);
static_assert(alignof(SaTouchEvent) == 8);
static_assert(SA_OFFSETOF(SaTouchEvent, timing) == 0);
static_assert(SA_OFFSETOF(SaTouchEvent, player_index) == 24);
static_assert(SA_OFFSETOF(SaTouchEvent, has_player) == 28);
static_assert(SA_OFFSETOF(SaTouchEvent, is_team_0) == 29);
static_assert(SA_OFFSETOF(SaTouchEvent, closest_approach_distance) == 32);
static_assert(SA_OFFSETOF(SaTouchEvent, has_closest_approach_distance) == 36);

static_assert(std::is_standard_layout_v<SaDodgeRefreshedEvent>);
static_assert(sizeof(SaDodgeRefreshedEvent) == 40);
static_assert(alignof(SaDodgeRefreshedEvent) == 8);
static_assert(SA_OFFSETOF(SaDodgeRefreshedEvent, timing) == 0);
static_assert(SA_OFFSETOF(SaDodgeRefreshedEvent, player_index) == 24);
static_assert(SA_OFFSETOF(SaDodgeRefreshedEvent, is_team_0) == 28);
static_assert(SA_OFFSETOF(SaDodgeRefreshedEvent, counter_value) == 32);

static_assert(std::is_standard_layout_v<SaBoostPadEvent>);
static_assert(sizeof(SaBoostPadEvent) == 48);
static_assert(alignof(SaBoostPadEvent) == 8);
static_assert(SA_OFFSETOF(SaBoostPadEvent, timing) == 0);
static_assert(SA_OFFSETOF(SaBoostPadEvent, pad_id) == 24);
static_assert(SA_OFFSETOF(SaBoostPadEvent, kind) == 28);
static_assert(SA_OFFSETOF(SaBoostPadEvent, sequence) == 32);
static_assert(SA_OFFSETOF(SaBoostPadEvent, player_index) == 36);
static_assert(SA_OFFSETOF(SaBoostPadEvent, has_player) == 40);

static_assert(std::is_standard_layout_v<SaGoalEvent>);
static_assert(sizeof(SaGoalEvent) == 56);
static_assert(alignof(SaGoalEvent) == 8);
static_assert(SA_OFFSETOF(SaGoalEvent, timing) == 0);
static_assert(SA_OFFSETOF(SaGoalEvent, scoring_team_is_team_0) == 24);
static_assert(SA_OFFSETOF(SaGoalEvent, player_index) == 28);
static_assert(SA_OFFSETOF(SaGoalEvent, has_player) == 32);
static_assert(SA_OFFSETOF(SaGoalEvent, team_zero_score) == 36);
static_assert(SA_OFFSETOF(SaGoalEvent, has_team_zero_score) == 40);
static_assert(SA_OFFSETOF(SaGoalEvent, team_one_score) == 44);
static_assert(SA_OFFSETOF(SaGoalEvent, has_team_one_score) == 48);

static_assert(std::is_standard_layout_v<SaPlayerStatEvent>);
static_assert(sizeof(SaPlayerStatEvent) == 160);
static_assert(alignof(SaPlayerStatEvent) == 8);
static_assert(SA_OFFSETOF(SaPlayerStatEvent, timing) == 0);
static_assert(SA_OFFSETOF(SaPlayerStatEvent, player_index) == 24);
static_assert(SA_OFFSETOF(SaPlayerStatEvent, is_team_0) == 28);
static_assert(SA_OFFSETOF(SaPlayerStatEvent, kind) == 32);
static_assert(SA_OFFSETOF(SaPlayerStatEvent, has_shot_ball) == 36);
static_assert(SA_OFFSETOF(SaPlayerStatEvent, shot_ball) == 40);
static_assert(SA_OFFSETOF(SaPlayerStatEvent, has_shot_player) == 96);
static_assert(SA_OFFSETOF(SaPlayerStatEvent, shot_player) == 100);

static_assert(std::is_standard_layout_v<SaDemolishEvent>);
static_assert(sizeof(SaDemolishEvent) == 72);
static_assert(alignof(SaDemolishEvent) == 8);
static_assert(SA_OFFSETOF(SaDemolishEvent, timing) == 0);
static_assert(SA_OFFSETOF(SaDemolishEvent, attacker_index) == 24);
static_assert(SA_OFFSETOF(SaDemolishEvent, victim_index) == 28);
static_assert(SA_OFFSETOF(SaDemolishEvent, attacker_velocity) == 32);
static_assert(SA_OFFSETOF(SaDemolishEvent, victim_velocity) == 44);
static_assert(SA_OFFSETOF(SaDemolishEvent, victim_location) == 56);
static_assert(SA_OFFSETOF(SaDemolishEvent, active_duration_seconds) == 68);

static_assert(std::is_standard_layout_v<SaLiveFrame>);
static_assert(sizeof(SaLiveFrame) == 232);
static_assert(alignof(SaLiveFrame) == 8);
static_assert(SA_OFFSETOF(SaLiveFrame, frame_number) == 0);
static_assert(SA_OFFSETOF(SaLiveFrame, time) == 8);
static_assert(SA_OFFSETOF(SaLiveFrame, dt) == 12);
static_assert(SA_OFFSETOF(SaLiveFrame, seconds_remaining) == 16);
static_assert(SA_OFFSETOF(SaLiveFrame, has_seconds_remaining) == 20);
static_assert(SA_OFFSETOF(SaLiveFrame, game_state) == 24);
static_assert(SA_OFFSETOF(SaLiveFrame, has_game_state) == 28);
static_assert(SA_OFFSETOF(SaLiveFrame, kickoff_countdown_time) == 32);
static_assert(SA_OFFSETOF(SaLiveFrame, has_kickoff_countdown_time) == 36);
static_assert(SA_OFFSETOF(SaLiveFrame, ball_has_been_hit) == 37);
static_assert(SA_OFFSETOF(SaLiveFrame, has_ball_has_been_hit) == 38);
static_assert(SA_OFFSETOF(SaLiveFrame, team_zero_score) == 40);
static_assert(SA_OFFSETOF(SaLiveFrame, has_team_zero_score) == 44);
static_assert(SA_OFFSETOF(SaLiveFrame, team_one_score) == 48);
static_assert(SA_OFFSETOF(SaLiveFrame, has_team_one_score) == 52);
static_assert(SA_OFFSETOF(SaLiveFrame, possession_team_is_team_0) == 53);
static_assert(SA_OFFSETOF(SaLiveFrame, has_possession_team) == 54);
static_assert(SA_OFFSETOF(SaLiveFrame, scored_on_team_is_team_0) == 55);
static_assert(SA_OFFSETOF(SaLiveFrame, has_scored_on_team) == 56);
static_assert(SA_OFFSETOF(SaLiveFrame, live_play) == 57);
static_assert(SA_OFFSETOF(SaLiveFrame, has_live_play) == 58);
static_assert(SA_OFFSETOF(SaLiveFrame, has_ball) == 59);
static_assert(SA_OFFSETOF(SaLiveFrame, ball) == 60);
static_assert(SA_OFFSETOF(SaLiveFrame, players) == 120);
static_assert(SA_OFFSETOF(SaLiveFrame, player_count) == 128);
static_assert(SA_OFFSETOF(SaLiveFrame, touches) == 136);
static_assert(SA_OFFSETOF(SaLiveFrame, touch_count) == 144);
static_assert(SA_OFFSETOF(SaLiveFrame, dodge_refreshes) == 152);
static_assert(SA_OFFSETOF(SaLiveFrame, dodge_refresh_count) == 160);
static_assert(SA_OFFSETOF(SaLiveFrame, boost_pad_events) == 168);
static_assert(SA_OFFSETOF(SaLiveFrame, boost_pad_event_count) == 176);
static_assert(SA_OFFSETOF(SaLiveFrame, goals) == 184);
static_assert(SA_OFFSETOF(SaLiveFrame, goal_count) == 192);
static_assert(SA_OFFSETOF(SaLiveFrame, player_stat_events) == 200);
static_assert(SA_OFFSETOF(SaLiveFrame, player_stat_event_count) == 208);
static_assert(SA_OFFSETOF(SaLiveFrame, demolishes) == 216);
static_assert(SA_OFFSETOF(SaLiveFrame, demolish_count) == 224);

static_assert(std::is_standard_layout_v<SaReplayScore>);
static_assert(sizeof(SaReplayScore) == 16);
static_assert(alignof(SaReplayScore) == 4);
static_assert(SA_OFFSETOF(SaReplayScore, team_zero_score) == 0);
static_assert(SA_OFFSETOF(SaReplayScore, has_team_zero_score) == 4);
static_assert(SA_OFFSETOF(SaReplayScore, team_one_score) == 8);
static_assert(SA_OFFSETOF(SaReplayScore, has_team_one_score) == 12);

static_assert(std::is_standard_layout_v<SaMechanicEvent>);
static_assert(sizeof(SaMechanicEvent) == 32);
static_assert(alignof(SaMechanicEvent) == 8);
static_assert(SA_OFFSETOF(SaMechanicEvent, kind) == 0);
static_assert(SA_OFFSETOF(SaMechanicEvent, player_index) == 4);
static_assert(SA_OFFSETOF(SaMechanicEvent, is_team_0) == 8);
static_assert(SA_OFFSETOF(SaMechanicEvent, frame_number) == 16);
static_assert(SA_OFFSETOF(SaMechanicEvent, time) == 24);
static_assert(SA_OFFSETOF(SaMechanicEvent, confidence) == 28);

static_assert(std::is_standard_layout_v<SaReplayPlayerInfo>);
static_assert(sizeof(SaReplayPlayerInfo) == 16);
static_assert(alignof(SaReplayPlayerInfo) == 8);
static_assert(SA_OFFSETOF(SaReplayPlayerInfo, player_index) == 0);
static_assert(SA_OFFSETOF(SaReplayPlayerInfo, is_team_0) == 4);
static_assert(SA_OFFSETOF(SaReplayPlayerInfo, name) == 8);

static_assert(std::is_standard_layout_v<SaTeamEvent>);
static_assert(sizeof(SaTeamEvent) == 48);
static_assert(alignof(SaTeamEvent) == 8);
static_assert(SA_OFFSETOF(SaTeamEvent, kind) == 0);
static_assert(SA_OFFSETOF(SaTeamEvent, is_team_0) == 4);
static_assert(SA_OFFSETOF(SaTeamEvent, start_frame) == 8);
static_assert(SA_OFFSETOF(SaTeamEvent, end_frame) == 16);
static_assert(SA_OFFSETOF(SaTeamEvent, start_time) == 24);
static_assert(SA_OFFSETOF(SaTeamEvent, end_time) == 28);
static_assert(SA_OFFSETOF(SaTeamEvent, attackers) == 32);
static_assert(SA_OFFSETOF(SaTeamEvent, defenders) == 36);
static_assert(SA_OFFSETOF(SaTeamEvent, confidence) == 40);

static_assert(std::is_standard_layout_v<SaGoalContextEvent>);
static_assert(sizeof(SaGoalContextEvent) == 64);
static_assert(alignof(SaGoalContextEvent) == 8);
static_assert(SA_OFFSETOF(SaGoalContextEvent, frame_number) == 0);
static_assert(SA_OFFSETOF(SaGoalContextEvent, time) == 8);
static_assert(SA_OFFSETOF(SaGoalContextEvent, scoring_team_is_team_0) == 12);
static_assert(SA_OFFSETOF(SaGoalContextEvent, has_scorer) == 13);
static_assert(SA_OFFSETOF(SaGoalContextEvent, scorer_index) == 16);
static_assert(SA_OFFSETOF(SaGoalContextEvent, has_scoring_team_most_back_player) == 20);
static_assert(SA_OFFSETOF(SaGoalContextEvent, scoring_team_most_back_player_index) == 24);
static_assert(SA_OFFSETOF(SaGoalContextEvent, has_defending_team_most_back_player) == 28);
static_assert(SA_OFFSETOF(SaGoalContextEvent, defending_team_most_back_player_index) == 32);
static_assert(SA_OFFSETOF(SaGoalContextEvent, has_ball_position) == 36);
static_assert(SA_OFFSETOF(SaGoalContextEvent, ball_position) == 40);
static_assert(SA_OFFSETOF(SaGoalContextEvent, has_ball_air_time_before_goal) == 52);
static_assert(SA_OFFSETOF(SaGoalContextEvent, ball_air_time_before_goal) == 56);
static_assert(SA_OFFSETOF(SaGoalContextEvent, goal_buildup) == 60);


} // namespace

#undef SA_OFFSETOF
