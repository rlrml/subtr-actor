use super::*;

#[derive(Default)]
pub struct PlayerVerticalStateCalculator;

impl PlayerVerticalStateCalculator {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, players: &PlayerFrameState) -> PlayerVerticalState {
        let players = players
            .players
            .iter()
            .filter_map(|player| {
                let height = player.position()?.z;
                Some((
                    player.player_id.clone(),
                    PlayerVerticalSample::from_height(height),
                ))
            })
            .collect();

        PlayerVerticalState { players }
    }
}
