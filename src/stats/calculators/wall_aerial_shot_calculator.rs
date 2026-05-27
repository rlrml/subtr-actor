use super::*;

#[derive(Debug, Clone, Default)]
pub struct WallAerialShotCalculator {
    pub(super) player_stats: HashMap<PlayerId, WallAerialShotStats>,
    pub(super) events: Vec<WallAerialShotEvent>,
    pub(super) recent_wall_contacts: HashMap<PlayerId, RecentWallContact>,
    pub(super) armed_shots: HashMap<PlayerId, ArmedWallAerialShot>,
    pub(super) current_last_wall_aerial_shot_player: Option<PlayerId>,
}

impl WallAerialShotCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, WallAerialShotStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[WallAerialShotEvent] {
        &self.events
    }
}
