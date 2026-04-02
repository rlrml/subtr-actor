use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::*;

// Require the turnover to occur at least slightly inside the new attacking
// team's defensive half rather than anywhere around midfield.
const DEFAULT_RUSH_MAX_START_Y: f32 = -BOOST_PAD_MIDFIELD_TOLERANCE_Y;
const DEFAULT_RUSH_ATTACK_SUPPORT_DISTANCE_Y: f32 = 900.0;
const DEFAULT_RUSH_DEFENDER_DISTANCE_Y: f32 = 150.0;
const DEFAULT_RUSH_MIN_POSSESSION_RETAINED_SECONDS: f32 = 0.75;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RushEvent {
    pub start_time: f32,
    pub start_frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub is_team_0: bool,
    pub attackers: usize,
    pub defenders: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveRush {
    start_time: f32,
    start_frame: usize,
    last_time: f32,
    last_frame: usize,
    is_team_0: bool,
    attackers: usize,
    defenders: usize,
    counted: bool,
}

impl ActiveRush {
    fn retained_possession_time(&self) -> f32 {
        (self.last_time - self.start_time).max(0.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RushCalculatorConfig {
    pub max_start_y: f32,
    pub attack_support_distance_y: f32,
    pub defender_distance_y: f32,
    pub min_possession_retained_seconds: f32,
}

impl Default for RushCalculatorConfig {
    fn default() -> Self {
        Self {
            max_start_y: DEFAULT_RUSH_MAX_START_Y,
            attack_support_distance_y: DEFAULT_RUSH_ATTACK_SUPPORT_DISTANCE_Y,
            defender_distance_y: DEFAULT_RUSH_DEFENDER_DISTANCE_Y,
            min_possession_retained_seconds: DEFAULT_RUSH_MIN_POSSESSION_RETAINED_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RushStats {
    pub team_zero_count: u32,
    pub team_zero_two_v_one_count: u32,
    pub team_zero_two_v_two_count: u32,
    pub team_zero_two_v_three_count: u32,
    pub team_zero_three_v_one_count: u32,
    pub team_zero_three_v_two_count: u32,
    pub team_zero_three_v_three_count: u32,
    pub team_one_count: u32,
    pub team_one_two_v_one_count: u32,
    pub team_one_two_v_two_count: u32,
    pub team_one_two_v_three_count: u32,
    pub team_one_three_v_one_count: u32,
    pub team_one_three_v_two_count: u32,
    pub team_one_three_v_three_count: u32,
}

impl RushStats {
    fn record(&mut self, attacking_team_is_team_0: bool, attackers: usize, defenders: usize) {
        if attacking_team_is_team_0 {
            self.team_zero_count += 1;
            match (attackers, defenders) {
                (2, 1) => self.team_zero_two_v_one_count += 1,
                (2, 2) => self.team_zero_two_v_two_count += 1,
                (2, 3) => self.team_zero_two_v_three_count += 1,
                (3, 1) => self.team_zero_three_v_one_count += 1,
                (3, 2) => self.team_zero_three_v_two_count += 1,
                (3, 3) => self.team_zero_three_v_three_count += 1,
                _ => {}
            }
        } else {
            self.team_one_count += 1;
            match (attackers, defenders) {
                (2, 1) => self.team_one_two_v_one_count += 1,
                (2, 2) => self.team_one_two_v_two_count += 1,
                (2, 3) => self.team_one_two_v_three_count += 1,
                (3, 1) => self.team_one_three_v_one_count += 1,
                (3, 2) => self.team_one_three_v_two_count += 1,
                (3, 3) => self.team_one_three_v_three_count += 1,
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RushCalculator {
    config: RushCalculatorConfig,
    stats: RushStats,
    events: Vec<RushEvent>,
    active_rush: Option<ActiveRush>,
}

impl RushCalculator {
    pub fn new() -> Self {
        Self::with_config(RushCalculatorConfig::default())
    }

    pub fn with_config(config: RushCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn config(&self) -> &RushCalculatorConfig {
        &self.config
    }

    pub fn stats(&self) -> &RushStats {
        &self.stats
    }

    pub fn events(&self) -> &[RushEvent] {
        &self.events
    }

    fn record_active_rush(&mut self, active_rush: &mut ActiveRush) {
        if active_rush.counted {
            return;
        }
        if active_rush.retained_possession_time() < self.config.min_possession_retained_seconds {
            return;
        }

        self.stats.record(
            active_rush.is_team_0,
            active_rush.attackers,
            active_rush.defenders,
        );
        active_rush.counted = true;
    }

    fn rush_numbers(
        &self,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        attacking_team_is_team_0: bool,
    ) -> Option<(usize, usize)> {
        let ball_position = ball.position()?;
        let normalized_ball_y = normalized_y(attacking_team_is_team_0, ball_position);
        if normalized_ball_y > self.config.max_start_y {
            return None;
        }

        let demoed_players: HashSet<_> = events
            .active_demos
            .iter()
            .map(|demo| demo.victim.clone())
            .collect();

        let attackers = players
            .players
            .iter()
            .filter(|player| player.is_team_0 == attacking_team_is_team_0)
            .filter(|player| !demoed_players.contains(&player.player_id))
            .filter_map(PlayerSample::position)
            .filter(|position| {
                normalized_y(attacking_team_is_team_0, *position)
                    >= normalized_ball_y - self.config.attack_support_distance_y
            })
            .count()
            .min(3);

        let defenders = players
            .players
            .iter()
            .filter(|player| player.is_team_0 != attacking_team_is_team_0)
            .filter(|player| !demoed_players.contains(&player.player_id))
            .filter_map(PlayerSample::position)
            .filter(|position| {
                normalized_y(attacking_team_is_team_0, *position)
                    >= normalized_ball_y + self.config.defender_distance_y
            })
            .count()
            .min(3);

        if attackers < 2 || defenders == 0 {
            return None;
        }

        Some((attackers, defenders))
    }

    fn finalize_active_rush(&mut self) {
        let Some(mut active_rush) = self.active_rush.take() else {
            return;
        };
        self.record_active_rush(&mut active_rush);
        if !active_rush.counted {
            return;
        }
        self.events.push(RushEvent {
            start_time: active_rush.start_time,
            start_frame: active_rush.start_frame,
            end_time: active_rush.last_time,
            end_frame: active_rush.last_frame,
            is_team_0: active_rush.is_team_0,
            attackers: active_rush.attackers,
            defenders: active_rush.defenders,
        });
    }

    fn update_active_rush(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        current_team_is_team_0: Option<bool>,
    ) {
        let Some(active_team_is_team_0) = self.active_rush.as_ref().map(|rush| rush.is_team_0)
        else {
            return;
        };

        let active_continues = current_team_is_team_0 == Some(active_team_is_team_0)
            && self
                .rush_numbers(ball, players, events, active_team_is_team_0)
                .is_some();
        if active_continues {
            if let Some(active_rush) = self.active_rush.as_mut() {
                active_rush.last_time = frame.time;
                active_rush.last_frame = frame.frame_number;
            }
            if let Some(mut active_rush) = self.active_rush.take() {
                self.record_active_rush(&mut active_rush);
                self.active_rush = Some(active_rush);
            }
            return;
        }

        self.finalize_active_rush();
    }

    fn maybe_start_rush(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        active_team_before_sample: Option<bool>,
        current_team_is_team_0: Option<bool>,
    ) {
        let Some(attacking_team_is_team_0) = current_team_is_team_0 else {
            return;
        };
        if active_team_before_sample == Some(attacking_team_is_team_0) {
            return;
        }

        if let Some((attackers, defenders)) =
            self.rush_numbers(ball, players, events, attacking_team_is_team_0)
        {
            self.active_rush = Some(ActiveRush {
                start_time: frame.time,
                start_frame: frame.frame_number,
                last_time: frame.time,
                last_frame: frame.frame_number,
                is_team_0: attacking_team_is_team_0,
                attackers,
                defenders,
                counted: false,
            });
        }
    }

    fn update_rush_state(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        active_team_before_sample: Option<bool>,
        current_team_is_team_0: Option<bool>,
    ) {
        self.update_active_rush(frame, ball, players, events, current_team_is_team_0);
        if self.active_rush.is_none() {
            self.maybe_start_rush(
                frame,
                ball,
                players,
                events,
                active_team_before_sample,
                current_team_is_team_0,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_parts(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play: bool,
        possession_state: &PossessionState,
    ) -> SubtrActorResult<()> {
        if !live_play || gameplay.kickoff_phase_active() {
            self.finalize_active_rush();
            return Ok(());
        }

        self.update_rush_state(
            frame,
            ball,
            players,
            events,
            possession_state.active_team_before_sample,
            possession_state.current_team_is_team_0,
        );

        Ok(())
    }
    pub fn finish_calculation(&mut self) -> SubtrActorResult<()> {
        self.finalize_active_rush();
        Ok(())
    }
}
