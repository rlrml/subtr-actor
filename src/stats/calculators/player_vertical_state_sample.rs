use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerVerticalSample {
    pub height: f32,
    pub band: PlayerVerticalBand,
}

impl PlayerVerticalSample {
    pub fn from_height(height: f32) -> Self {
        Self {
            height,
            band: PlayerVerticalBand::from_height(height),
        }
    }
}
