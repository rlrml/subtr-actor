use std::collections::HashSet;

use serde::Serialize;

use super::*;

// Require the turnover to occur at least slightly inside the new attacking
// team's defensive half rather than anywhere around midfield.
const DEFAULT_RUSH_MAX_START_Y: f32 = -BOOST_PAD_MIDFIELD_TOLERANCE_Y;
const DEFAULT_RUSH_ATTACK_SUPPORT_DISTANCE_Y: f32 = 900.0;
const DEFAULT_RUSH_DEFENDER_DISTANCE_Y: f32 = 150.0;
const DEFAULT_RUSH_MIN_POSSESSION_RETAINED_SECONDS: f32 = 0.75;

#[derive(Debug, Clone, PartialEq, Serialize)]
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
pub struct RushReducerConfig {
    pub max_start_y: f32,
    pub attack_support_distance_y: f32,
    pub defender_distance_y: f32,
    pub min_possession_retained_seconds: f32,
}

impl Default for RushReducerConfig {
    fn default() -> Self {
        Self {
            max_start_y: DEFAULT_RUSH_MAX_START_Y,
            attack_support_distance_y: DEFAULT_RUSH_ATTACK_SUPPORT_DISTANCE_Y,
            defender_distance_y: DEFAULT_RUSH_DEFENDER_DISTANCE_Y,
            min_possession_retained_seconds: DEFAULT_RUSH_MIN_POSSESSION_RETAINED_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
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
pub struct RushReducer {
    config: RushReducerConfig,
    stats: RushStats,
    events: Vec<RushEvent>,
    active_rush: Option<ActiveRush>,
    live_play_tracker: LivePlayTracker,
}

impl RushReducer {
    pub fn new() -> Self {
        Self::with_config(RushReducerConfig::default())
    }

    pub fn with_config(config: RushReducerConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn config(&self) -> &RushReducerConfig {
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
        sample: &StatsSample,
        attacking_team_is_team_0: bool,
    ) -> Option<(usize, usize)> {
        let ball_position = sample.ball.as_ref()?.position();
        let normalized_ball_y = normalized_y(attacking_team_is_team_0, ball_position);
        if normalized_ball_y > self.config.max_start_y {
            return None;
        }

        let demoed_players: HashSet<_> = sample
            .active_demos
            .iter()
            .map(|demo| demo.victim.clone())
            .collect();

        let attackers = sample
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

        let defenders = sample
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

    fn update_active_rush(&mut self, sample: &StatsSample, current_team_is_team_0: Option<bool>) {
        let Some(active_team_is_team_0) = self.active_rush.as_ref().map(|rush| rush.is_team_0)
        else {
            return;
        };

        let active_continues = current_team_is_team_0 == Some(active_team_is_team_0)
            && self.rush_numbers(sample, active_team_is_team_0).is_some();
        if active_continues {
            if let Some(active_rush) = self.active_rush.as_mut() {
                active_rush.last_time = sample.time;
                active_rush.last_frame = sample.frame_number;
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
        sample: &StatsSample,
        active_team_before_sample: Option<bool>,
        current_team_is_team_0: Option<bool>,
    ) {
        let Some(attacking_team_is_team_0) = current_team_is_team_0 else {
            return;
        };
        if active_team_before_sample == Some(attacking_team_is_team_0) {
            return;
        }

        if let Some((attackers, defenders)) = self.rush_numbers(sample, attacking_team_is_team_0) {
            self.active_rush = Some(ActiveRush {
                start_time: sample.time,
                start_frame: sample.frame_number,
                last_time: sample.time,
                last_frame: sample.frame_number,
                is_team_0: attacking_team_is_team_0,
                attackers,
                defenders,
                counted: false,
            });
        }
    }

    fn update_rush_state(
        &mut self,
        sample: &StatsSample,
        active_team_before_sample: Option<bool>,
        current_team_is_team_0: Option<bool>,
    ) {
        self.update_active_rush(sample, current_team_is_team_0);
        if self.active_rush.is_none() {
            self.maybe_start_rush(sample, active_team_before_sample, current_team_is_team_0);
        }
    }
}

impl StatsReducer for RushReducer {
    fn on_sample_with_context(
        &mut self,
        sample: &StatsSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        if !self.live_play_tracker.is_live_play(sample)
            || FiftyFiftyReducer::kickoff_phase_active(sample)
        {
            self.finalize_active_rush();
            return Ok(());
        }

        let possession_state = ctx
            .get::<PossessionState>(POSSESSION_STATE_SIGNAL_ID)
            .cloned()
            .unwrap_or_default();
        self.update_rush_state(
            sample,
            possession_state.active_team_before_sample,
            possession_state.current_team_is_team_0,
        );

        Ok(())
    }

    fn finish(&mut self) -> SubtrActorResult<()> {
        self.finalize_active_rush();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};

    use super::*;

    fn rigid_body(x: f32, y: f32) -> RigidBody {
        RigidBody {
            sleeping: false,
            location: Vector3f { x, y, z: 17.0 },
            rotation: Quaternion {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            linear_velocity: Some(Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
            angular_velocity: Some(Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
        }
    }

    fn player(player_id: u64, is_team_0: bool, x: f32, y: f32) -> PlayerSample {
        PlayerSample {
            player_id: RemoteId::Steam(player_id),
            is_team_0,
            rigid_body: Some(rigid_body(x, y)),
            boost_amount: Some(50.0),
            last_boost_amount: Some(50.0),
            boost_active: false,
            dodge_active: false,
            powerslide_active: false,
            match_goals: None,
            match_assists: None,
            match_saves: None,
            match_shots: None,
            match_score: None,
        }
    }

    fn sample_with_ball_y(players: Vec<PlayerSample>, ball_y: f32) -> StatsSample {
        StatsSample {
            frame_number: 10,
            time: 5.0,
            dt: 1.0 / 120.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: Some(0),
            team_one_score: Some(0),
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: Some([3, 3]),
            ball: Some(BallSample {
                rigid_body: rigid_body(0.0, ball_y),
            }),
            players,
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        }
    }

    fn sample(players: Vec<PlayerSample>) -> StatsSample {
        sample_with_ball_y(players, -200.0)
    }

    #[test]
    fn classifies_two_v_one_from_turnover_shape() {
        let reducer = RushReducer::new();
        let sample = sample(vec![
            player(1, true, 0.0, -500.0),
            player(2, true, 300.0, 250.0),
            player(3, true, -1500.0, -2600.0),
            player(4, false, 0.0, 1800.0),
            player(5, false, 800.0, -150.0),
            player(6, false, -900.0, -1800.0),
        ]);

        assert_eq!(reducer.rush_numbers(&sample, true), Some((2, 1)));
    }

    #[test]
    fn counts_rush_once_when_possession_changes() {
        let start_sample = sample(vec![
            player(1, true, 0.0, -500.0),
            player(2, true, 300.0, 250.0),
            player(3, true, -1500.0, -2600.0),
            player(4, false, 0.0, 1800.0),
            player(5, false, 800.0, -150.0),
            player(6, false, -900.0, -1800.0),
        ]);
        let continue_sample = StatsSample {
            frame_number: 11,
            time: 5.1,
            ..sample(vec![
                player(1, true, 0.0, -450.0),
                player(2, true, 300.0, 300.0),
                player(3, true, -1500.0, -2200.0),
                player(4, false, 0.0, 1700.0),
                player(5, false, 800.0, -100.0),
                player(6, false, -900.0, -1700.0),
            ])
        };

        let mut reducer = RushReducer::with_config(RushReducerConfig {
            min_possession_retained_seconds: 0.05,
            ..RushReducerConfig::default()
        });
        reducer.update_rush_state(&start_sample, Some(false), Some(true));
        assert_eq!(reducer.stats().team_zero_count, 0);
        assert_eq!(reducer.stats().team_zero_two_v_one_count, 0);
        assert_eq!(reducer.events().len(), 0);

        reducer.update_rush_state(&continue_sample, Some(true), Some(true));
        assert_eq!(reducer.stats().team_zero_count, 1);
        assert_eq!(reducer.stats().team_zero_two_v_one_count, 1);
        assert_eq!(reducer.events().len(), 0);

        reducer.update_rush_state(&continue_sample, Some(true), Some(true));
        assert_eq!(reducer.stats().team_zero_count, 1);
        assert_eq!(reducer.stats().team_zero_two_v_one_count, 1);
    }

    #[test]
    fn does_not_count_rush_when_turnover_starts_at_midfield() {
        let reducer = RushReducer::new();
        let sample = sample_with_ball_y(
            vec![
                player(1, true, 0.0, -500.0),
                player(2, true, 300.0, 250.0),
                player(3, true, -1500.0, -2600.0),
                player(4, false, 0.0, 1800.0),
                player(5, false, 800.0, -150.0),
                player(6, false, -900.0, -1800.0),
            ],
            0.0,
        );

        assert_eq!(reducer.rush_numbers(&sample, true), None);
    }

    #[test]
    fn records_rush_event_with_start_and_end_frames() {
        let mut reducer = RushReducer::with_config(RushReducerConfig {
            min_possession_retained_seconds: 0.05,
            ..RushReducerConfig::default()
        });
        let start_sample = sample(vec![
            player(1, true, 0.0, -500.0),
            player(2, true, 300.0, 250.0),
            player(3, true, -1500.0, -2600.0),
            player(4, false, 0.0, 1800.0),
            player(5, false, 800.0, -150.0),
            player(6, false, -900.0, -1800.0),
        ]);
        let continue_sample = StatsSample {
            frame_number: 11,
            time: 5.1,
            ..sample(vec![
                player(1, true, 0.0, -450.0),
                player(2, true, 300.0, 300.0),
                player(3, true, -1500.0, -2200.0),
                player(4, false, 0.0, 1700.0),
                player(5, false, 800.0, -100.0),
                player(6, false, -900.0, -1700.0),
            ])
        };
        let end_sample = StatsSample {
            frame_number: 12,
            time: 5.2,
            ..sample_with_ball_y(
                vec![
                    player(1, true, 0.0, -200.0),
                    player(2, true, 300.0, 700.0),
                    player(3, true, -1500.0, -1800.0),
                    player(4, false, 0.0, 1800.0),
                    player(5, false, 800.0, 100.0),
                    player(6, false, -900.0, -1500.0),
                ],
                300.0,
            )
        };

        reducer.update_rush_state(&start_sample, Some(false), Some(true));
        reducer.update_rush_state(&continue_sample, Some(true), Some(true));
        reducer.update_rush_state(&end_sample, Some(true), Some(true));

        assert_eq!(reducer.stats().team_zero_count, 1);
        assert_eq!(
            reducer.events(),
            &[RushEvent {
                start_time: 5.0,
                start_frame: 10,
                end_time: 5.1,
                end_frame: 11,
                is_team_0: true,
                attackers: 2,
                defenders: 1,
            }]
        );
    }

    #[test]
    fn does_not_count_short_lived_rush_before_retention_threshold() {
        let mut reducer = RushReducer::with_config(RushReducerConfig {
            min_possession_retained_seconds: 0.2,
            ..RushReducerConfig::default()
        });
        let start_sample = sample(vec![
            player(1, true, 0.0, -500.0),
            player(2, true, 300.0, 250.0),
            player(3, true, -1500.0, -2600.0),
            player(4, false, 0.0, 1800.0),
            player(5, false, 800.0, -150.0),
            player(6, false, -900.0, -1800.0),
        ]);
        let brief_continue_sample = StatsSample {
            frame_number: 11,
            time: 5.05,
            ..sample(vec![
                player(1, true, 0.0, -450.0),
                player(2, true, 300.0, 300.0),
                player(3, true, -1500.0, -2200.0),
                player(4, false, 0.0, 1700.0),
                player(5, false, 800.0, -100.0),
                player(6, false, -900.0, -1700.0),
            ])
        };
        let end_sample = StatsSample {
            frame_number: 12,
            time: 5.1,
            ..sample_with_ball_y(
                vec![
                    player(1, true, 0.0, -200.0),
                    player(2, true, 300.0, 700.0),
                    player(3, true, -1500.0, -1800.0),
                    player(4, false, 0.0, 1800.0),
                    player(5, false, 800.0, 100.0),
                    player(6, false, -900.0, -1500.0),
                ],
                300.0,
            )
        };

        reducer.update_rush_state(&start_sample, Some(false), Some(true));
        reducer.update_rush_state(&brief_continue_sample, Some(true), Some(true));
        reducer.update_rush_state(&end_sample, Some(true), Some(true));

        assert_eq!(reducer.stats().team_zero_count, 0);
        assert!(reducer.events().is_empty());
    }
}
