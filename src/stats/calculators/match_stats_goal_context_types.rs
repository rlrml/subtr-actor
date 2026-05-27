use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalContextPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<glam::Vec3> for GoalContextPosition {
    fn from(position: glam::Vec3) -> Self {
        Self {
            x: position.x,
            y: position.y,
            z: position.z,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalPlayerContext {
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub position: Option<GoalContextPosition>,
    pub boost_amount: Option<f32>,
    pub average_boost_in_leadup: Option<f32>,
    pub min_boost_in_leadup: Option<f32>,
    pub is_most_back: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalTouchContext {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub ball_position: Option<GoalContextPosition>,
    pub player_position: Option<GoalContextPosition>,
    pub players: Vec<GoalPlayerContext>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalContextEvent {
    pub time: f32,
    pub frame: usize,
    pub scoring_team_is_team_0: bool,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub scorer: Option<PlayerId>,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub scoring_team_most_back_player: Option<PlayerId>,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub defending_team_most_back_player: Option<PlayerId>,
    pub ball_position: Option<GoalContextPosition>,
    pub ball_air_time_before_goal: Option<f32>,
    #[serde(default)]
    pub goal_buildup: GoalBuildupKind,
    pub scorer_last_touch: Option<GoalTouchContext>,
    pub players: Vec<GoalPlayerContext>,
}
