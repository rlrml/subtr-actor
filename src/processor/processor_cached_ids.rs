use super::*;

#[derive(Clone, Copy, Default)]
pub(crate) struct CachedObjectIds {
    pub(crate) player_type: Option<boxcars::ObjectId>,
    pub(crate) car_type: Option<boxcars::ObjectId>,
    pub(crate) boost_type: Option<boxcars::ObjectId>,
    pub(crate) dodge_type: Option<boxcars::ObjectId>,
    pub(crate) jump_type: Option<boxcars::ObjectId>,
    pub(crate) double_jump_type: Option<boxcars::ObjectId>,
    pub(crate) unique_id: Option<boxcars::ObjectId>,
    pub(crate) team: Option<boxcars::ObjectId>,
    pub(crate) bot: Option<boxcars::ObjectId>,
    pub(crate) player_replication: Option<boxcars::ObjectId>,
    pub(crate) vehicle: Option<boxcars::ObjectId>,
    pub(crate) boost_replicated: Option<boxcars::ObjectId>,
    pub(crate) boost_amount: Option<boxcars::ObjectId>,
    pub(crate) component_active: Option<boxcars::ObjectId>,
    pub(crate) seconds_remaining: Option<boxcars::ObjectId>,
    pub(crate) replicated_state_name: Option<boxcars::ObjectId>,
    pub(crate) replicated_game_state_time_remaining: Option<boxcars::ObjectId>,
    pub(crate) ball_has_been_hit: Option<boxcars::ObjectId>,
    pub(crate) ball_hit_team_num: Option<boxcars::ObjectId>,
    pub(crate) dodges_refreshed_counter: Option<boxcars::ObjectId>,
}

impl CachedObjectIds {
    pub(crate) fn from_name_map(name_to_object_id: &HashMap<String, boxcars::ObjectId>) -> Self {
        let cached = |name| name_to_object_id.get(name).copied();
        Self {
            player_type: cached(PLAYER_TYPE),
            car_type: cached(CAR_TYPE),
            boost_type: cached(BOOST_TYPE),
            dodge_type: cached(DODGE_TYPE),
            jump_type: cached(JUMP_TYPE),
            double_jump_type: cached(DOUBLE_JUMP_TYPE),
            unique_id: cached(UNIQUE_ID_KEY),
            team: cached(TEAM_KEY),
            bot: cached(BOT_KEY),
            player_replication: cached(PLAYER_REPLICATION_KEY),
            vehicle: cached(VEHICLE_KEY),
            boost_replicated: cached(BOOST_REPLICATED_KEY),
            boost_amount: cached(BOOST_AMOUNT_KEY),
            component_active: cached(COMPONENT_ACTIVE_KEY),
            seconds_remaining: cached(SECONDS_REMAINING_KEY),
            replicated_state_name: cached(REPLICATED_STATE_NAME_KEY),
            replicated_game_state_time_remaining: cached(REPLICATED_GAME_STATE_TIME_REMAINING_KEY),
            ball_has_been_hit: cached(BALL_HAS_BEEN_HIT_KEY),
            ball_hit_team_num: cached(BALL_HIT_TEAM_NUM_KEY),
            dodges_refreshed_counter: cached(DODGES_REFRESHED_COUNTER_KEY),
        }
    }
}
