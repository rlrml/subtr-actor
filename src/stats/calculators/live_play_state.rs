use super::GameplayPhase;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePlayState {
    pub gameplay_phase: GameplayPhase,
    pub is_live_play: bool,
}

impl LivePlayState {
    pub fn counts_toward_player_motion(&self) -> bool {
        self.gameplay_phase.counts_toward_player_motion()
    }

    pub fn counts_toward_ball_position_stats(&self) -> bool {
        self.gameplay_phase.counts_toward_ball_position_stats()
    }
}
