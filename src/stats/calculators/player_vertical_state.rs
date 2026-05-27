use super::*;

#[path = "player_vertical_state_calculator.rs"]
mod calculator;
#[path = "player_vertical_state_sample.rs"]
mod sample;

pub use sample::*;
pub use self::calculator::PlayerVerticalStateCalculator;

pub const PLAYER_GROUND_Z_THRESHOLD: f32 = 20.0;
pub const PLAYER_HIGH_AIR_Z_THRESHOLD: f32 = 642.775 + BALL_RADIUS_Z;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerVerticalBand {
    Ground,
    LowAir,
    HighAir,
}

pub const ALL_PLAYER_VERTICAL_BANDS: [PlayerVerticalBand; 3] = [
    PlayerVerticalBand::Ground,
    PlayerVerticalBand::LowAir,
    PlayerVerticalBand::HighAir,
];

impl PlayerVerticalBand {
    pub fn from_height(height: f32) -> Self {
        if height <= PLAYER_GROUND_Z_THRESHOLD {
            Self::Ground
        } else if height >= PLAYER_HIGH_AIR_Z_THRESHOLD {
            Self::HighAir
        } else {
            Self::LowAir
        }
    }

    pub fn as_label(self) -> StatLabel {
        let value = match self {
            Self::Ground => "ground",
            Self::LowAir => "low_air",
            Self::HighAir => "high_air",
        };
        StatLabel::new("height_band", value)
    }

    pub fn is_grounded(self) -> bool {
        matches!(self, Self::Ground)
    }

    pub fn is_airborne(self) -> bool {
        !self.is_grounded()
    }

    pub fn is_high_air(self) -> bool {
        matches!(self, Self::HighAir)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlayerVerticalState {
    pub players: HashMap<PlayerId, PlayerVerticalSample>,
}

impl PlayerVerticalState {
    pub fn sample(&self, player_id: &PlayerId) -> Option<&PlayerVerticalSample> {
        self.players.get(player_id)
    }

    pub fn band_for_player(&self, player_id: &PlayerId) -> Option<PlayerVerticalBand> {
        self.sample(player_id).map(|sample| sample.band)
    }

    pub fn is_grounded(&self, player_id: &PlayerId) -> bool {
        self.band_for_player(player_id)
            .is_some_and(PlayerVerticalBand::is_grounded)
    }
}
