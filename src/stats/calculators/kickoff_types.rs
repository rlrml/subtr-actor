use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum KickoffSpawnPosition {
    Center,
    OffCenterLeft,
    OffCenterRight,
    DiagonalLeft,
    DiagonalRight,
    #[default]
    Unknown,
}

impl KickoffSpawnPosition {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Center => "center",
            Self::OffCenterLeft => "off_center_left",
            Self::OffCenterRight => "off_center_right",
            Self::DiagonalLeft => "diagonal_left",
            Self::DiagonalRight => "diagonal_right",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum KickoffTakerOutcome {
    Touched,
    Fake,
    Missed,
    #[default]
    Unknown,
}

impl KickoffTakerOutcome {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Touched => "touched",
            Self::Fake => "fake",
            Self::Missed => "missed",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum KickoffOutcome {
    TeamZeroWin,
    TeamOneWin,
    Neutral,
    #[default]
    Unknown,
}

impl KickoffOutcome {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::TeamZeroWin => "team_zero_win",
            Self::TeamOneWin => "team_one_win",
            Self::Neutral => "neutral",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum KickoffWinStrengthBand {
    Narrow,
    Clear,
    Strong,
    #[default]
    Unknown,
}

impl KickoffWinStrengthBand {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Narrow => "narrow",
            Self::Clear => "clear",
            Self::Strong => "strong",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum KickoffPossessionOutcome {
    TeamZeroPossession,
    TeamOnePossession,
    TeamZeroAdvantage,
    TeamOneAdvantage,
    #[default]
    Contested,
}

impl KickoffPossessionOutcome {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::TeamZeroPossession => "team_zero_possession",
            Self::TeamOnePossession => "team_one_possession",
            Self::TeamZeroAdvantage => "team_zero_advantage",
            Self::TeamOneAdvantage => "team_one_advantage",
            Self::Contested => "contested",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum KickoffApproach {
    SpeedFlip,
    BoostIntoBall,
    FakeGoForBoost,
    FrontFlip,
    DiagonalFlip,
    #[default]
    Other,
}

impl KickoffApproach {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::SpeedFlip => "speed_flip",
            Self::BoostIntoBall => "boost_into_ball",
            Self::FakeGoForBoost => "fake_go_for_boost",
            Self::FrontFlip => "front_flip",
            Self::DiagonalFlip => "diagonal_flip",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum KickoffSupportBehavior {
    GoForBoost,
    Cheat,
    Other,
    #[default]
    Unknown,
}

impl KickoffSupportBehavior {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::GoForBoost => "go_for_boost",
            Self::Cheat => "cheat",
            Self::Other => "other",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct KickoffTakerEvent {
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub start_position: [f32; 3],
    pub spawn_position: KickoffSpawnPosition,
    pub start_boost: Option<f32>,
    pub boost_after: Option<f32>,
    pub first_touch_time: Option<f32>,
    pub first_touch_frame: Option<usize>,
    pub outcome: KickoffTakerOutcome,
    pub approach: KickoffApproach,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct KickoffSupportEvent {
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub start_position: [f32; 3],
    pub spawn_position: KickoffSpawnPosition,
    pub start_boost: Option<f32>,
    pub boost_after: Option<f32>,
    pub first_touch_time: Option<f32>,
    pub first_touch_frame: Option<usize>,
    pub support_behavior: KickoffSupportBehavior,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct KickoffEvent {
    pub start_time: f32,
    pub start_frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub first_touch_time: Option<f32>,
    pub first_touch_frame: Option<usize>,
    pub first_touch_team_is_team_0: Option<bool>,
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub first_touch_player: Option<PlayerId>,
    pub team_zero_taker_touch_time: Option<f32>,
    pub team_zero_taker_touch_frame: Option<usize>,
    pub team_one_taker_touch_time: Option<f32>,
    pub team_one_taker_touch_frame: Option<usize>,
    pub taker_touch_delay_seconds: Option<f32>,
    pub exit_velocity: Option<[f32; 3]>,
    pub exit_speed: Option<f32>,
    pub exit_y_velocity: Option<f32>,
    pub first_follow_up_touch_time: Option<f32>,
    pub first_follow_up_touch_frame: Option<usize>,
    pub first_follow_up_touch_team_is_team_0: Option<bool>,
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub first_follow_up_touch_player: Option<PlayerId>,
    pub outcome: KickoffOutcome,
    pub winning_team_is_team_0: Option<bool>,
    pub win_strength: Option<f32>,
    pub win_strength_band: KickoffWinStrengthBand,
    pub kickoff_possession_outcome: KickoffPossessionOutcome,
    pub kickoff_possession_team_is_team_0: Option<bool>,
    pub kickoff_goal: bool,
    pub scoring_team_is_team_0: Option<bool>,
    pub time_to_goal: Option<f32>,
    pub team_zero_taker: Option<KickoffTakerEvent>,
    pub team_one_taker: Option<KickoffTakerEvent>,
    pub team_zero_non_takers: Vec<KickoffSupportEvent>,
    pub team_one_non_takers: Vec<KickoffSupportEvent>,
}
