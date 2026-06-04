use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PowerslideEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub active: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PowerslideCalculator {
    last_active: HashMap<PlayerId, bool>,
    events: EventStream<PowerslideEvent>,
}

impl PowerslideCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[PowerslideEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[PowerslideEvent] {
        self.events.new_events()
    }

    fn is_effective_powerslide(player: &PlayerSample) -> bool {
        player.powerslide_active
            && player
                .position()
                .map(|position| position.z <= POWERSLIDE_MAX_Z_THRESHOLD)
                .unwrap_or(false)
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        _live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        for player in &players.players {
            let effective_powerslide = Self::is_effective_powerslide(player);
            let previous_active = self
                .last_active
                .get(&player.player_id)
                .copied()
                .unwrap_or(false);

            if effective_powerslide != previous_active {
                self.events.push(PowerslideEvent {
                    time: frame.time,
                    frame: frame.frame_number,
                    player: player.player_id.clone(),
                    player_position: player.position().map(|position| position.to_array()),
                    is_team_0: player.is_team_0,
                    active: effective_powerslide,
                });
            }

            self.last_active
                .insert(player.player_id.clone(), effective_powerslide);
        }
        Ok(())
    }
}
