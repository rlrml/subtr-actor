use super::*;

/// Spawn position a player took for a kickoff.
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

/// The kickoff formation/type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum KickoffType {
    Diagonal,
    CenterOffset,
    Center,
    #[default]
    Unknown,
}

impl KickoffType {
    pub fn from_taker_spawns(
        team_zero_spawn: Option<KickoffSpawnPosition>,
        team_one_spawn: Option<KickoffSpawnPosition>,
    ) -> Self {
        match (team_zero_spawn, team_one_spawn) {
            (
                Some(KickoffSpawnPosition::DiagonalLeft),
                Some(KickoffSpawnPosition::DiagonalLeft),
            )
            | (
                Some(KickoffSpawnPosition::DiagonalRight),
                Some(KickoffSpawnPosition::DiagonalRight),
            ) => Self::Diagonal,
            (
                Some(KickoffSpawnPosition::OffCenterLeft),
                Some(KickoffSpawnPosition::OffCenterLeft),
            )
            | (
                Some(KickoffSpawnPosition::OffCenterRight),
                Some(KickoffSpawnPosition::OffCenterRight),
            ) => Self::CenterOffset,
            (Some(KickoffSpawnPosition::Center), Some(KickoffSpawnPosition::Center)) => {
                Self::Center
            }
            _ => Self::Unknown,
        }
    }

    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Diagonal => "diagonal",
            Self::CenterOffset => "center_offset",
            Self::Center => "center",
            Self::Unknown => "unknown",
        }
    }
}

/// Direction a player approached the kickoff from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum KickoffDirection {
    Left,
    Right,
    Center,
    #[default]
    Unknown,
}

impl KickoffDirection {
    pub fn from_taker_spawns(
        team_zero_spawn: Option<KickoffSpawnPosition>,
        team_one_spawn: Option<KickoffSpawnPosition>,
    ) -> Self {
        match (team_zero_spawn, team_one_spawn) {
            (
                Some(KickoffSpawnPosition::DiagonalLeft),
                Some(KickoffSpawnPosition::DiagonalLeft),
            )
            | (
                Some(KickoffSpawnPosition::OffCenterLeft),
                Some(KickoffSpawnPosition::OffCenterLeft),
            ) => Self::Left,
            (
                Some(KickoffSpawnPosition::DiagonalRight),
                Some(KickoffSpawnPosition::DiagonalRight),
            )
            | (
                Some(KickoffSpawnPosition::OffCenterRight),
                Some(KickoffSpawnPosition::OffCenterRight),
            ) => Self::Right,
            (Some(KickoffSpawnPosition::Center), Some(KickoffSpawnPosition::Center)) => {
                Self::Center
            }
            _ => Self::Unknown,
        }
    }

    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Center => "center",
            Self::Unknown => "unknown",
        }
    }
}

/// Outcome for the player who took the kickoff.
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

/// Overall outcome of a kickoff.
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

/// Banded strength of a kickoff win.
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

/// Possession outcome following a kickoff.
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

/// Who the kickoff was ultimately good for once play settled, independent of
/// who won the immediate touch battle. A team gains the advantage by the
/// first of: stringing uncontested touches together long enough to count as
/// real possession (even deep in its own half — losing the opening touch but
/// collecting the ball cleanly is the collector's advantage), pinning the
/// ball in the opponent's half with touch engagement and no clean opposing
/// possession, or a qualifying kickoff goal. A kickoff where neither team
/// achieves any of those within the window stays `NoAdvantage`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum KickoffAdvantage {
    TeamZeroPossession,
    TeamOnePossession,
    TeamZeroPressure,
    TeamOnePressure,
    TeamZeroGoal,
    TeamOneGoal,
    #[default]
    NoAdvantage,
}

impl KickoffAdvantage {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::TeamZeroPossession => "team_zero_possession",
            Self::TeamOnePossession => "team_one_possession",
            Self::TeamZeroPressure => "team_zero_pressure",
            Self::TeamOnePressure => "team_one_pressure",
            Self::TeamZeroGoal => "team_zero_goal",
            Self::TeamOneGoal => "team_one_goal",
            Self::NoAdvantage => "no_advantage",
        }
    }

    pub fn team_is_team_0(self) -> Option<bool> {
        match self {
            Self::TeamZeroPossession | Self::TeamZeroPressure | Self::TeamZeroGoal => Some(true),
            Self::TeamOnePossession | Self::TeamOnePressure | Self::TeamOneGoal => Some(false),
            Self::NoAdvantage => None,
        }
    }
}

/// How a player approached the kickoff.
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

/// Local left/right direction of the dodge used during a kickoff approach.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum KickoffFlipDirection {
    Left,
    Right,
    #[default]
    NotApplicable,
}

impl KickoffFlipDirection {
    pub fn from_local_side_component(side_component: f32) -> Self {
        if side_component > f32::EPSILON {
            Self::Right
        } else if side_component < -f32::EPSILON {
            Self::Left
        } else {
            Self::NotApplicable
        }
    }

    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::NotApplicable => "not_applicable",
        }
    }
}

/// Behavior of a supporting (non-taking) player on a kickoff.
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

/// Direction the ball traveled off the kickoff.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum KickoffBallDirection {
    Left,
    Right,
    Center,
    #[default]
    Unknown,
}

impl KickoffBallDirection {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Center => "center",
            Self::Unknown => "unknown",
        }
    }
}

/// Kickoff event for the player who challenged the ball.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct KickoffTakerEvent {
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub start_position: [f32; 3],
    pub spawn_position: KickoffSpawnPosition,
    pub start_boost: Option<f32>,
    /// Boost remaining at the moment the taker contacts the ball (the
    /// counterpart to `boost_used`, which is spent reaching that contact).
    /// For takers that never touch the ball (fake / missed) this falls back to
    /// the end-of-kickoff sample. Invariant when the taker touches:
    /// `start_boost + boost_collected == boost_used + boost_after`.
    pub boost_after: Option<f32>,
    pub time_to_ball: Option<f32>,
    pub boost_collected: f32,
    pub boost_used: f32,
    pub ball_direction: KickoffBallDirection,
    pub first_touch_time: Option<f32>,
    pub first_touch_frame: Option<usize>,
    /// Car center position at the taker's first kickoff touch, when the touch
    /// could be matched to a player sample on that frame.
    pub contact_player_position: Option<[f32; 3]>,
    pub contact_player_velocity: Option<[f32; 3]>,
    pub contact_car_forward: Option<[f32; 3]>,
    /// Ball center in the taker's local hitbox coordinates at contact.
    pub contact_local_ball_position: Option<[f32; 3]>,
    /// Closest point on the taker's local hitbox to the ball center.
    pub contact_local_contact_point: Option<[f32; 3]>,
    /// Estimated ball-to-hitbox gap after subtracting the ball collision radius.
    pub contact_gap: Option<f32>,
    /// Team-normalized distance from car center to ball along the attack axis.
    /// Positive means the taker is behind the ball on their own-goal side.
    pub contact_behind_ball_depth: Option<f32>,
    /// Team-normalized left/right offset of car center from the ball.
    pub contact_lateral_offset: Option<f32>,
    pub contact_lateral_abs_offset: Option<f32>,
    /// Alignment of taker velocity with the opponent-goal direction (`-1..=1`).
    pub contact_velocity_attack_alignment: Option<f32>,
    /// Alignment of taker velocity with the car-to-ball direction (`-1..=1`).
    pub contact_velocity_ball_alignment: Option<f32>,
    /// Alignment of taker nose direction with the opponent-goal direction.
    pub contact_nose_attack_alignment: Option<f32>,
    /// Alignment of the ball's first-touch exit velocity with the taker's
    /// opponent-goal direction.
    pub contact_ball_exit_attack_alignment: Option<f32>,
    pub outcome: KickoffTakerOutcome,
    pub approach: KickoffApproach,
    #[serde(default)]
    pub approach_flip_direction: KickoffFlipDirection,
}

/// Kickoff event for a supporting player.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct KickoffSupportEvent {
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub start_position: [f32; 3],
    /// Horizontal distance (uu) of the support player's spawn from field
    /// center — i.e. from the ball's kickoff position at the origin. Larger
    /// values mean the player started further back/wider; a `cheat` support
    /// closes this gap as the kickoff develops while a back-positioned support
    /// keeps it.
    pub start_distance_from_center: f32,
    pub spawn_position: KickoffSpawnPosition,
    pub start_boost: Option<f32>,
    pub boost_after: Option<f32>,
    pub first_touch_time: Option<f32>,
    pub first_touch_frame: Option<usize>,
    pub support_behavior: KickoffSupportBehavior,
}

/// A kickoff with its type, outcome, and participant context.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct KickoffEvent {
    pub start_time: f32,
    pub start_frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub live_action_start_time: Option<f32>,
    pub live_action_start_frame: Option<usize>,
    pub movement_start_time: f32,
    pub movement_start_frame: usize,
    pub kickoff_type: KickoffType,
    pub kickoff_direction: KickoffDirection,
    pub first_touch_time: Option<f32>,
    pub first_touch_frame: Option<usize>,
    pub first_touch_team_is_team_0: Option<bool>,
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub first_touch_player: Option<PlayerId>,
    /// Identity of the first kickoff [`TouchEvent`](crate::TouchEvent). Join on
    /// this instead of player + frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number")]
    pub first_touch_id: Option<u64>,
    /// Ball position (field coordinates) at the frame of the first kickoff touch.
    pub first_touch_ball_position: Option<[f32; 3]>,
    /// Lateral centrality of the first-touch contact: `abs(x)` of the ball
    /// position at first touch. `0.0` means the ball was struck dead center.
    pub first_touch_ball_abs_x: Option<f32>,
    /// Ball height (z) at first touch. On a standard kickoff the ball rests at
    /// center field, so values above the resting radius indicate the ball was
    /// popped or hit upward.
    pub first_touch_ball_height: Option<f32>,
    /// Ball velocity at the first-touch frame (immediately after contact).
    pub first_touch_ball_velocity: Option<[f32; 3]>,
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
    /// Projected depth of the ball into the losing half at kickoff resolution,
    /// as a fraction of the half-field length (`0.0..=1.0`). The projection
    /// adds half a second of the ball's y velocity so direction of travel
    /// counts toward (or against) the win.
    pub win_strength: Option<f32>,
    pub win_strength_band: KickoffWinStrengthBand,
    pub kickoff_possession_outcome: KickoffPossessionOutcome,
    pub kickoff_possession_team_is_team_0: Option<bool>,
    pub kickoff_goal: bool,
    pub scoring_team_is_team_0: Option<bool>,
    pub time_to_goal: Option<f32>,
    /// See [`KickoffAdvantage`]. Unlike `outcome` and
    /// `kickoff_possession_outcome`, which read the immediate exchange, this
    /// answers "who did the kickoff actually end up being good for" once play
    /// settled.
    pub advantage: KickoffAdvantage,
    pub advantage_team_is_team_0: Option<bool>,
    pub advantage_time: Option<f32>,
    pub advantage_frame: Option<usize>,
    pub advantage_seconds_after_first_touch: Option<f32>,
    /// For possession advantages, the player whose touch completed the
    /// possession run. Pressure and goal advantages are team-level.
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub advantage_player: Option<PlayerId>,
    pub team_zero_taker: Option<KickoffTakerEvent>,
    pub team_one_taker: Option<KickoffTakerEvent>,
    pub team_zero_non_takers: Vec<KickoffSupportEvent>,
    pub team_one_non_takers: Vec<KickoffSupportEvent>,
}
