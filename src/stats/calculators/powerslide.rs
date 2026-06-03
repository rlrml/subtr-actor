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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PowerslideStats {
    pub total_duration: f32,
    pub press_count: u32,
}

impl PowerslideStats {
    pub fn average_duration(&self) -> f32 {
        if self.press_count == 0 {
            0.0
        } else {
            self.total_duration / self.press_count as f32
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PowerslideCalculator {
    player_stats: HashMap<PlayerId, PowerslideStats>,
    team_zero_stats: PowerslideStats,
    team_one_stats: PowerslideStats,
    last_active: HashMap<PlayerId, bool>,
    events: EventStream<PowerslideEvent>,
}

impl PowerslideCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, PowerslideStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &PowerslideStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &PowerslideStats {
        &self.team_one_stats
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
            let stats = self
                .player_stats
                .entry(player.player_id.clone())
                .or_default();
            let team_stats = if player.is_team_0 {
                &mut self.team_zero_stats
            } else {
                &mut self.team_one_stats
            };

            if live_play && effective_powerslide {
                stats.total_duration += frame.dt;
                team_stats.total_duration += frame.dt;
            }

            if live_play && effective_powerslide && !previous_active {
                stats.press_count += 1;
                team_stats.press_count += 1;
            }

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
