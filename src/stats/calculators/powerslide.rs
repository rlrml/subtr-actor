use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PowerslideEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub active: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PowerslideCalculator {
    stats: PowerslideStatsAccumulator,
    last_active: HashMap<PlayerId, bool>,
    events: EventStream<PowerslideEvent>,
}

impl PowerslideCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, PowerslideStats> {
        self.stats.player_stats()
    }

    pub fn team_zero_stats(&self) -> &PowerslideStats {
        self.stats.team_zero_stats()
    }

    pub fn team_one_stats(&self) -> &PowerslideStats {
        self.stats.team_one_stats()
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
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        for player in &players.players {
            let effective_powerslide = Self::is_effective_powerslide(player);
            let previous_active = self
                .last_active
                .get(&player.player_id)
                .copied()
                .unwrap_or(false);
            self.stats.apply_sample(
                &player.player_id,
                player.is_team_0,
                effective_powerslide,
                previous_active,
                frame.dt,
                live_play,
            );

            if effective_powerslide != previous_active {
                self.events.push(PowerslideEvent {
                    time: frame.time,
                    frame: frame.frame_number,
                    player: player.player_id.clone(),
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
