use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GameplayPhase {
    #[default]
    Unknown,
    KickoffCountdown,
    KickoffWaitingForTouch,
    ActivePlay,
    PostGoal,
}

impl GameplayPhase {
    pub fn is_live_play(self) -> bool {
        matches!(self, Self::ActivePlay)
    }

    pub fn counts_toward_player_motion(self) -> bool {
        matches!(self, Self::ActivePlay | Self::KickoffWaitingForTouch)
    }

    pub fn counts_toward_ball_position_stats(self) -> bool {
        self.is_live_play()
    }
}
