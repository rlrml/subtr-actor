use std::collections::HashSet;

use serde::Serialize;

use super::*;

const RUSH_MAX_START_Y: f32 = BOOST_PAD_MIDFIELD_TOLERANCE_Y;
const RUSH_ATTACK_SUPPORT_DISTANCE_Y: f32 = 900.0;
const RUSH_DEFENDER_DISTANCE_Y: f32 = 150.0;

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
    stats: RushStats,
    live_play_tracker: LivePlayTracker,
}

impl RushReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &RushStats {
        &self.stats
    }

    fn rush_numbers(
        sample: &StatsSample,
        attacking_team_is_team_0: bool,
    ) -> Option<(usize, usize)> {
        let ball_position = sample.ball.as_ref()?.position();
        let normalized_ball_y = normalized_y(attacking_team_is_team_0, ball_position);
        if normalized_ball_y > RUSH_MAX_START_Y {
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
                    >= normalized_ball_y - RUSH_ATTACK_SUPPORT_DISTANCE_Y
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
                    >= normalized_ball_y + RUSH_DEFENDER_DISTANCE_Y
            })
            .count()
            .min(3);

        if attackers < 2 || defenders == 0 {
            return None;
        }

        Some((attackers, defenders))
    }

    fn record_possession_change(
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

        if let Some((attackers, defenders)) = Self::rush_numbers(sample, attacking_team_is_team_0) {
            self.stats
                .record(attacking_team_is_team_0, attackers, defenders);
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
            return Ok(());
        }

        let possession_state = ctx
            .get::<PossessionState>(POSSESSION_STATE_SIGNAL_ID)
            .cloned()
            .unwrap_or_default();
        self.record_possession_change(
            sample,
            possession_state.active_team_before_sample,
            possession_state.current_team_is_team_0,
        );

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

    fn sample(players: Vec<PlayerSample>) -> StatsSample {
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
                rigid_body: rigid_body(0.0, -200.0),
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

    #[test]
    fn classifies_two_v_one_from_turnover_shape() {
        let sample = sample(vec![
            player(1, true, 0.0, -500.0),
            player(2, true, 300.0, 250.0),
            player(3, true, -1500.0, -2600.0),
            player(4, false, 0.0, 1800.0),
            player(5, false, 800.0, -150.0),
            player(6, false, -900.0, -1800.0),
        ]);

        assert_eq!(RushReducer::rush_numbers(&sample, true), Some((2, 1)));
    }

    #[test]
    fn counts_rush_once_when_possession_changes() {
        let sample = sample(vec![
            player(1, true, 0.0, -500.0),
            player(2, true, 300.0, 250.0),
            player(3, true, -1500.0, -2600.0),
            player(4, false, 0.0, 1800.0),
            player(5, false, 800.0, -150.0),
            player(6, false, -900.0, -1800.0),
        ]);

        let mut reducer = RushReducer::new();
        reducer.record_possession_change(&sample, Some(false), Some(true));
        assert_eq!(reducer.stats().team_zero_count, 1);
        assert_eq!(reducer.stats().team_zero_two_v_one_count, 1);

        reducer.record_possession_change(&sample, Some(true), Some(true));
        assert_eq!(reducer.stats().team_zero_count, 1);
        assert_eq!(reducer.stats().team_zero_two_v_one_count, 1);
    }
}
