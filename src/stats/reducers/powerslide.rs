use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
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
pub struct PowerslideReducer {
    player_stats: HashMap<PlayerId, PowerslideStats>,
    team_zero_stats: PowerslideStats,
    team_one_stats: PowerslideStats,
    last_active: HashMap<PlayerId, bool>,
    live_play_tracker: LivePlayTracker,
}

impl PowerslideReducer {
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

    fn is_effective_powerslide(player: &PlayerSample) -> bool {
        player.powerslide_active
            && player
                .position()
                .map(|position| position.z <= POWERSLIDE_MAX_Z_THRESHOLD)
                .unwrap_or(false)
    }
}

impl StatsReducer for PowerslideReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        let live_play = self.live_play_tracker.is_live_play(sample);
        for player in &sample.players {
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
                stats.total_duration += sample.dt;
                team_stats.total_duration += sample.dt;
            }

            if live_play && effective_powerslide && !previous_active {
                stats.press_count += 1;
                team_stats.press_count += 1;
            }

            self.last_active
                .insert(player.player_id.clone(), effective_powerslide);
        }
        Ok(())
    }
}
