use boxcars;

use crate::*;

#[derive(Debug, Clone)]
pub struct BallSample {
    pub rigid_body: boxcars::RigidBody,
}

impl BallSample {
    pub fn position(&self) -> glam::Vec3 {
        vec_to_glam(&self.rigid_body.location)
    }

    pub fn velocity(&self) -> glam::Vec3 {
        self.rigid_body
            .linear_velocity
            .as_ref()
            .map(vec_to_glam)
            .unwrap_or(glam::Vec3::ZERO)
    }
}

#[derive(Debug, Clone)]
pub struct PlayerSample {
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub rigid_body: Option<boxcars::RigidBody>,
    pub boost_amount: Option<f32>,
    pub last_boost_amount: Option<f32>,
    pub boost_active: bool,
    pub dodge_active: bool,
    pub powerslide_active: bool,
    pub match_goals: Option<i32>,
    pub match_assists: Option<i32>,
    pub match_saves: Option<i32>,
    pub match_shots: Option<i32>,
    pub match_score: Option<i32>,
}

impl PlayerSample {
    pub fn position(&self) -> Option<glam::Vec3> {
        self.rigid_body.as_ref().map(|rb| vec_to_glam(&rb.location))
    }

    pub fn velocity(&self) -> Option<glam::Vec3> {
        self.rigid_body
            .as_ref()
            .and_then(|rb| rb.linear_velocity.as_ref().map(vec_to_glam))
    }

    pub fn speed(&self) -> Option<f32> {
        self.velocity().map(|velocity| velocity.length())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemoEventSample {
    pub attacker: PlayerId,
    pub victim: PlayerId,
}

#[derive(Debug, Clone)]
pub struct CoreSample {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub ball_has_been_hit: Option<bool>,
    pub kickoff_countdown_time: Option<i32>,
    pub team_zero_score: Option<i32>,
    pub team_one_score: Option<i32>,
    pub possession_team_is_team_0: Option<bool>,
    pub scored_on_team_is_team_0: Option<bool>,
    pub current_in_game_team_player_counts: Option<[usize; 2]>,
    pub ball: Option<BallSample>,
    pub players: Vec<PlayerSample>,
    pub active_demos: Vec<DemoEventSample>,
    pub demo_events: Vec<DemolishInfo>,
    pub boost_pad_events: Vec<BoostPadEvent>,
    pub touch_events: Vec<TouchEvent>,
    pub dodge_refreshed_events: Vec<DodgeRefreshedEvent>,
    pub player_stat_events: Vec<PlayerStatEvent>,
    pub goal_events: Vec<GoalEvent>,
}

pub(crate) const GAME_STATE_KICKOFF_COUNTDOWN: i32 = 55;
pub(crate) const GAME_STATE_GOAL_SCORED_REPLAY: i32 = 86;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LivePlayTracker {
    post_goal_phase_active: bool,
    last_score: Option<(i32, i32)>,
}

impl LivePlayTracker {
    fn current_score(sample: &CoreSample) -> Option<(i32, i32)> {
        Some((sample.team_zero_score?, sample.team_one_score?))
    }

    fn kickoff_phase_active(sample: &CoreSample) -> bool {
        sample.game_state == Some(GAME_STATE_KICKOFF_COUNTDOWN)
            || sample.kickoff_countdown_time.is_some_and(|time| time > 0)
    }

    fn live_play_internal(&mut self, sample: &CoreSample) -> bool {
        let kickoff_phase_active = Self::kickoff_phase_active(sample);
        let score_changed = Self::current_score(sample)
            .zip(self.last_score)
            .is_some_and(
                |((team_zero_score, team_one_score), (last_team_zero, last_team_one))| {
                    team_zero_score > last_team_zero || team_one_score > last_team_one
                },
            );

        if !sample.goal_events.is_empty() || score_changed {
            self.post_goal_phase_active = true;
        }

        let live_play = sample.is_live_play() && !self.post_goal_phase_active;

        if kickoff_phase_active {
            self.post_goal_phase_active = false;
        }

        if let Some(score) = Self::current_score(sample) {
            self.last_score = Some(score);
        }

        live_play
    }

    pub fn is_live_play(&mut self, sample: &CoreSample) -> bool {
        self.live_play_internal(sample)
    }
}

impl CoreSample {
    pub(crate) fn from_processor(
        processor: &ReplayProcessor,
        frame_number: usize,
        current_time: f32,
        dt: f32,
    ) -> SubtrActorResult<Self> {
        let ball = processor
            .get_interpolated_ball_rigid_body(current_time, 0.0)
            .ok()
            .filter(|rigid_body| !rigid_body.sleeping)
            .map(|rigid_body| BallSample { rigid_body });

        let mut players = Vec::new();
        for player_id in processor.iter_player_ids_in_order() {
            let Ok(is_team_0) = processor.get_player_is_team_0(player_id) else {
                continue;
            };
            players.push(PlayerSample {
                player_id: player_id.clone(),
                is_team_0,
                rigid_body: processor
                    .get_interpolated_player_rigid_body(player_id, current_time, 0.0)
                    .ok()
                    .filter(|rigid_body| !rigid_body.sleeping),
                boost_amount: processor.get_player_boost_level(player_id).ok(),
                last_boost_amount: processor.get_player_last_boost_level(player_id).ok(),
                boost_active: processor.get_boost_active(player_id).unwrap_or(0) % 2 == 1,
                dodge_active: processor.get_dodge_active(player_id).unwrap_or(0) % 2 == 1,
                powerslide_active: processor.get_powerslide_active(player_id).unwrap_or(false),
                match_goals: processor.get_player_match_goals(player_id).ok(),
                match_assists: processor.get_player_match_assists(player_id).ok(),
                match_saves: processor.get_player_match_saves(player_id).ok(),
                match_shots: processor.get_player_match_shots(player_id).ok(),
                match_score: processor.get_player_match_score(player_id).ok(),
            });
        }

        let team_scores = processor.get_team_scores().ok();
        let possession_team_is_team_0 =
            processor
                .get_ball_hit_team_num()
                .ok()
                .and_then(|team_num| match team_num {
                    0 => Some(true),
                    1 => Some(false),
                    _ => None,
                });
        let scored_on_team_is_team_0 =
            processor
                .get_scored_on_team_num()
                .ok()
                .and_then(|team_num| match team_num {
                    0 => Some(true),
                    1 => Some(false),
                    _ => None,
                });
        let active_demos = if let Ok(demos) = processor.get_active_demos() {
            demos
                .filter_map(|demo| {
                    let attacker = processor
                        .get_player_id_from_car_id(&demo.attacker_actor_id())
                        .ok()?;
                    let victim = processor
                        .get_player_id_from_car_id(&demo.victim_actor_id())
                        .ok()?;
                    Some(DemoEventSample { attacker, victim })
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(Self {
            frame_number,
            time: current_time,
            dt,
            seconds_remaining: processor.get_seconds_remaining().ok(),
            game_state: processor.get_replicated_state_name().ok(),
            ball_has_been_hit: processor.get_ball_has_been_hit().ok(),
            kickoff_countdown_time: processor.get_replicated_game_state_time_remaining().ok(),
            team_zero_score: team_scores.map(|scores| scores.0),
            team_one_score: team_scores.map(|scores| scores.1),
            possession_team_is_team_0,
            scored_on_team_is_team_0,
            current_in_game_team_player_counts: Some(
                processor.current_in_game_team_player_counts(),
            ),
            ball,
            players,
            active_demos,
            demo_events: Vec::new(),
            boost_pad_events: processor.current_frame_boost_pad_events().to_vec(),
            touch_events: processor.current_frame_touch_events().to_vec(),
            dodge_refreshed_events: processor.current_frame_dodge_refreshed_events().to_vec(),
            player_stat_events: processor.current_frame_player_stat_events().to_vec(),
            goal_events: processor.current_frame_goal_events().to_vec(),
        })
    }

    pub fn is_live_play(&self) -> bool {
        if matches!(
            self.game_state,
            Some(GAME_STATE_KICKOFF_COUNTDOWN | GAME_STATE_GOAL_SCORED_REPLAY)
        ) {
            return false;
        }

        true
    }

    pub fn current_in_game_team_player_count(&self, is_team_0: bool) -> usize {
        self.current_in_game_team_player_counts
            .map(|counts| counts[usize::from(!is_team_0)])
            .unwrap_or_else(|| {
                self.players
                    .iter()
                    .filter(|player| player.is_team_0 == is_team_0)
                    .count()
            })
    }
}
