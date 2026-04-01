use super::*;

pub const FIFTY_FIFTY_STATE_SIGNAL_ID: DerivedSignalId = "fifty_fifty_state";

pub(crate) const FIFTY_FIFTY_CONTINUATION_TOUCH_WINDOW_SECONDS: f32 = 0.2;
pub(crate) const FIFTY_FIFTY_RESOLUTION_DELAY_SECONDS: f32 = 0.35;
pub(crate) const FIFTY_FIFTY_MAX_DURATION_SECONDS: f32 = 1.25;
pub(crate) const FIFTY_FIFTY_MIN_EXIT_DISTANCE: f32 = 180.0;
pub(crate) const FIFTY_FIFTY_MIN_EXIT_SPEED: f32 = 220.0;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FiftyFiftyState {
    pub active_event: Option<ActiveFiftyFifty>,
    pub resolved_events: Vec<FiftyFiftyEvent>,
    pub last_resolved_event: Option<FiftyFiftyEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveFiftyFifty {
    pub start_time: f32,
    pub start_frame: usize,
    pub last_touch_time: f32,
    pub last_touch_frame: usize,
    pub is_kickoff: bool,
    pub team_zero_player: Option<PlayerId>,
    pub team_one_player: Option<PlayerId>,
    pub team_zero_position: [f32; 3],
    pub team_one_position: [f32; 3],
    pub midpoint: [f32; 3],
    pub plane_normal: [f32; 3],
}

impl ActiveFiftyFifty {
    pub fn midpoint_vec(&self) -> glam::Vec3 {
        glam::Vec3::from_array(self.midpoint)
    }

    pub fn plane_normal_vec(&self) -> glam::Vec3 {
        glam::Vec3::from_array(self.plane_normal)
    }

    pub fn contains_team_touch(&self, touch_events: &[TouchEvent]) -> bool {
        touch_events.iter().any(|touch| {
            (touch.team_is_team_0 && self.team_zero_player.is_some())
                || (!touch.team_is_team_0 && self.team_one_player.is_some())
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FiftyFiftyEvent {
    pub start_time: f32,
    pub start_frame: usize,
    pub resolve_time: f32,
    pub resolve_frame: usize,
    pub is_kickoff: bool,
    pub team_zero_player: Option<PlayerId>,
    pub team_one_player: Option<PlayerId>,
    pub team_zero_position: [f32; 3],
    pub team_one_position: [f32; 3],
    pub midpoint: [f32; 3],
    pub plane_normal: [f32; 3],
    pub winning_team_is_team_0: Option<bool>,
    pub possession_team_is_team_0: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct FiftyFiftyStats {
    pub count: u32,
    pub team_zero_wins: u32,
    pub team_one_wins: u32,
    pub neutral_outcomes: u32,
    pub kickoff_count: u32,
    pub kickoff_team_zero_wins: u32,
    pub kickoff_team_one_wins: u32,
    pub kickoff_neutral_outcomes: u32,
    pub team_zero_possession_after_count: u32,
    pub team_one_possession_after_count: u32,
    pub neutral_possession_after_count: u32,
    pub kickoff_team_zero_possession_after_count: u32,
    pub kickoff_team_one_possession_after_count: u32,
    pub kickoff_neutral_possession_after_count: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct FiftyFiftyPlayerStats {
    pub count: u32,
    pub wins: u32,
    pub losses: u32,
    pub neutral_outcomes: u32,
    pub kickoff_count: u32,
    pub kickoff_wins: u32,
    pub kickoff_losses: u32,
    pub kickoff_neutral_outcomes: u32,
    pub possession_after_count: u32,
    pub kickoff_possession_after_count: u32,
}

impl FiftyFiftyStats {
    pub fn team_zero_win_pct(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.team_zero_wins as f32 * 100.0 / self.count as f32
        }
    }

    pub fn team_one_win_pct(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.team_one_wins as f32 * 100.0 / self.count as f32
        }
    }

    pub fn kickoff_team_zero_win_pct(&self) -> f32 {
        if self.kickoff_count == 0 {
            0.0
        } else {
            self.kickoff_team_zero_wins as f32 * 100.0 / self.kickoff_count as f32
        }
    }

    pub fn kickoff_team_one_win_pct(&self) -> f32 {
        if self.kickoff_count == 0 {
            0.0
        } else {
            self.kickoff_team_one_wins as f32 * 100.0 / self.kickoff_count as f32
        }
    }
}

impl FiftyFiftyPlayerStats {
    pub fn win_pct(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.wins as f32 * 100.0 / self.count as f32
        }
    }

    pub fn kickoff_win_pct(&self) -> f32 {
        if self.kickoff_count == 0 {
            0.0
        } else {
            self.kickoff_wins as f32 * 100.0 / self.kickoff_count as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FiftyFiftyReducer {
    stats: FiftyFiftyStats,
    player_stats: HashMap<PlayerId, FiftyFiftyPlayerStats>,
    events: Vec<FiftyFiftyEvent>,
}

impl FiftyFiftyReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &FiftyFiftyStats {
        &self.stats
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, FiftyFiftyPlayerStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[FiftyFiftyEvent] {
        &self.events
    }

    fn apply_team_outcome(
        stats: &mut FiftyFiftyStats,
        winning_team_is_team_0: Option<bool>,
        is_kickoff: bool,
    ) {
        match winning_team_is_team_0 {
            Some(true) => {
                stats.team_zero_wins += 1;
                if is_kickoff {
                    stats.kickoff_team_zero_wins += 1;
                }
            }
            Some(false) => {
                stats.team_one_wins += 1;
                if is_kickoff {
                    stats.kickoff_team_one_wins += 1;
                }
            }
            None => {
                stats.neutral_outcomes += 1;
                if is_kickoff {
                    stats.kickoff_neutral_outcomes += 1;
                }
            }
        }
    }

    fn apply_possession_outcome(
        stats: &mut FiftyFiftyStats,
        possession_team_is_team_0: Option<bool>,
        is_kickoff: bool,
    ) {
        match possession_team_is_team_0 {
            Some(true) => {
                stats.team_zero_possession_after_count += 1;
                if is_kickoff {
                    stats.kickoff_team_zero_possession_after_count += 1;
                }
            }
            Some(false) => {
                stats.team_one_possession_after_count += 1;
                if is_kickoff {
                    stats.kickoff_team_one_possession_after_count += 1;
                }
            }
            None => {
                stats.neutral_possession_after_count += 1;
                if is_kickoff {
                    stats.kickoff_neutral_possession_after_count += 1;
                }
            }
        }
    }

    fn apply_player_outcome(
        player_stats: &mut FiftyFiftyPlayerStats,
        player_team_is_team_0: bool,
        event: &FiftyFiftyEvent,
    ) {
        player_stats.count += 1;
        if event.is_kickoff {
            player_stats.kickoff_count += 1;
        }

        match event.winning_team_is_team_0 {
            Some(team_is_team_0) if team_is_team_0 == player_team_is_team_0 => {
                player_stats.wins += 1;
                if event.is_kickoff {
                    player_stats.kickoff_wins += 1;
                }
            }
            Some(_) => {
                player_stats.losses += 1;
                if event.is_kickoff {
                    player_stats.kickoff_losses += 1;
                }
            }
            None => {
                player_stats.neutral_outcomes += 1;
                if event.is_kickoff {
                    player_stats.kickoff_neutral_outcomes += 1;
                }
            }
        }

        if event.possession_team_is_team_0 == Some(player_team_is_team_0) {
            player_stats.possession_after_count += 1;
            if event.is_kickoff {
                player_stats.kickoff_possession_after_count += 1;
            }
        }
    }

    fn apply_event(&mut self, event: &FiftyFiftyEvent) {
        self.stats.count += 1;
        if event.is_kickoff {
            self.stats.kickoff_count += 1;
        }
        Self::apply_team_outcome(
            &mut self.stats,
            event.winning_team_is_team_0,
            event.is_kickoff,
        );
        Self::apply_possession_outcome(
            &mut self.stats,
            event.possession_team_is_team_0,
            event.is_kickoff,
        );

        if let Some(player_id) = event.team_zero_player.as_ref() {
            let stats = self.player_stats.entry(player_id.clone()).or_default();
            Self::apply_player_outcome(stats, true, event);
        }
        if let Some(player_id) = event.team_one_player.as_ref() {
            let stats = self.player_stats.entry(player_id.clone()).or_default();
            Self::apply_player_outcome(stats, false, event);
        }

        self.events.push(event.clone());
    }

    pub(crate) fn kickoff_phase_active(sample: &StatsSample) -> bool {
        sample.game_state == Some(GAME_STATE_KICKOFF_COUNTDOWN)
            || sample.kickoff_countdown_time.is_some_and(|time| time > 0)
            || sample.ball_has_been_hit == Some(false)
    }

    pub(crate) fn contested_touch(
        sample: &StatsSample,
        touch_events: &[TouchEvent],
        is_kickoff: bool,
    ) -> Option<ActiveFiftyFifty> {
        let team_zero_touch = touch_events.iter().find(|touch| touch.team_is_team_0)?;
        let team_one_touch = touch_events.iter().find(|touch| !touch.team_is_team_0)?;
        let team_zero_position = team_zero_touch.player.as_ref().and_then(|player_id| {
            sample
                .players
                .iter()
                .find(|player| &player.player_id == player_id)
                .and_then(PlayerSample::position)
        })?;
        let team_one_position = team_one_touch.player.as_ref().and_then(|player_id| {
            sample
                .players
                .iter()
                .find(|player| &player.player_id == player_id)
                .and_then(PlayerSample::position)
        })?;
        let midpoint = (team_zero_position + team_one_position) * 0.5;
        let mut plane_normal = team_one_position - team_zero_position;
        plane_normal.z = 0.0;
        if plane_normal.length_squared() <= f32::EPSILON {
            plane_normal = glam::Vec3::Y;
        } else {
            plane_normal = plane_normal.normalize();
        }

        Some(ActiveFiftyFifty {
            start_time: sample.time,
            start_frame: sample.frame_number,
            last_touch_time: sample.time,
            last_touch_frame: sample.frame_number,
            is_kickoff,
            team_zero_player: team_zero_touch.player.clone(),
            team_one_player: team_one_touch.player.clone(),
            team_zero_position: team_zero_position.to_array(),
            team_one_position: team_one_position.to_array(),
            midpoint: midpoint.to_array(),
            plane_normal: plane_normal.to_array(),
        })
    }

    pub(crate) fn winning_team_from_ball(
        active: &ActiveFiftyFifty,
        sample: &StatsSample,
    ) -> Option<bool> {
        let ball = sample.ball.as_ref()?;
        let midpoint = active.midpoint_vec();
        let plane_normal = active.plane_normal_vec();
        let displacement = ball.position() - midpoint;
        let signed_distance = displacement.dot(plane_normal);
        if signed_distance.abs() >= FIFTY_FIFTY_MIN_EXIT_DISTANCE {
            return Some(signed_distance > 0.0);
        }

        let signed_speed = ball.velocity().dot(plane_normal);
        if signed_speed.abs() >= FIFTY_FIFTY_MIN_EXIT_SPEED {
            return Some(signed_speed > 0.0);
        }

        None
    }
}

impl StatsReducer for FiftyFiftyReducer {
    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        vec![FIFTY_FIFTY_STATE_SIGNAL_ID]
    }

    fn on_sample_with_context(
        &mut self,
        _sample: &StatsSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        let fifty_fifty_state = ctx
            .get::<FiftyFiftyState>(FIFTY_FIFTY_STATE_SIGNAL_ID)
            .cloned()
            .unwrap_or_default();

        for event in &fifty_fifty_state.resolved_events {
            self.apply_event(event);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "fifty_fifty_test.rs"]
mod tests;
