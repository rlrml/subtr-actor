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
            return Some(signed_distance < 0.0);
        }

        let signed_speed = ball.velocity().dot(plane_normal);
        if signed_speed.abs() >= FIFTY_FIFTY_MIN_EXIT_SPEED {
            return Some(signed_speed < 0.0);
        }

        None
    }
}

impl StatsReducer for FiftyFiftyReducer {
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
mod tests {
    use boxcars::RemoteId;

    use super::*;

    fn rigid_body(x: f32, y: f32, z: f32, vx: f32, vy: f32, vz: f32) -> boxcars::RigidBody {
        boxcars::RigidBody {
            sleeping: false,
            location: boxcars::Vector3f { x, y, z },
            rotation: boxcars::Quaternion {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            linear_velocity: Some(boxcars::Vector3f {
                x: vx,
                y: vy,
                z: vz,
            }),
            angular_velocity: Some(boxcars::Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
        }
    }

    fn sample(
        frame_number: usize,
        time: f32,
        ball_x: f32,
        ball_y: f32,
        ball_vx: f32,
        ball_vy: f32,
        touch_events: Vec<TouchEvent>,
        kickoff_countdown_time: Option<i32>,
        ball_has_been_hit: Option<bool>,
    ) -> StatsSample {
        StatsSample {
            frame_number,
            time,
            dt: 1.0 / 120.0,
            seconds_remaining: Some(100),
            game_state: Some(0),
            ball_has_been_hit,
            kickoff_countdown_time,
            team_zero_score: Some(0),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: Some([1, 1]),
            ball: Some(BallSample {
                rigid_body: rigid_body(ball_x, ball_y, 110.0, ball_vx, ball_vy, 0.0),
            }),
            players: vec![
                PlayerSample {
                    player_id: RemoteId::Steam(1),
                    is_team_0: true,
                    rigid_body: Some(rigid_body(-120.0, -40.0, 17.0, 0.0, 0.0, 0.0)),
                    boost_amount: None,
                    last_boost_amount: None,
                    boost_active: false,
                    powerslide_active: false,
                    match_goals: None,
                    match_assists: None,
                    match_saves: None,
                    match_shots: None,
                    match_score: None,
                },
                PlayerSample {
                    player_id: RemoteId::Steam(2),
                    is_team_0: false,
                    rigid_body: Some(rigid_body(120.0, 40.0, 17.0, 0.0, 0.0, 0.0)),
                    boost_amount: None,
                    last_boost_amount: None,
                    boost_active: false,
                    powerslide_active: false,
                    match_goals: None,
                    match_assists: None,
                    match_saves: None,
                    match_shots: None,
                    match_score: None,
                },
            ],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events,
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        }
    }

    #[test]
    fn contested_touch_builds_horizontal_plane() {
        let touch_events = vec![
            TouchEvent {
                time: 0.0,
                frame: 10,
                team_is_team_0: true,
                player: Some(RemoteId::Steam(1)),
                closest_approach_distance: Some(0.0),
            },
            TouchEvent {
                time: 0.0,
                frame: 10,
                team_is_team_0: false,
                player: Some(RemoteId::Steam(2)),
                closest_approach_distance: Some(0.0),
            },
        ];
        let sample = sample(
            10,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            touch_events.clone(),
            Some(0),
            Some(false),
        );
        let active = FiftyFiftyReducer::contested_touch(&sample, &touch_events, true).unwrap();

        assert!(active.is_kickoff);
        assert_eq!(active.team_zero_player, Some(RemoteId::Steam(1)));
        assert_eq!(active.team_one_player, Some(RemoteId::Steam(2)));
        assert!(active.plane_normal_vec().z.abs() <= f32::EPSILON);
        assert!(active.plane_normal_vec().length() > 0.99);
    }

    #[test]
    fn winning_team_uses_ball_side_and_velocity() {
        let touch_events = vec![
            TouchEvent {
                time: 0.0,
                frame: 10,
                team_is_team_0: true,
                player: Some(RemoteId::Steam(1)),
                closest_approach_distance: Some(0.0),
            },
            TouchEvent {
                time: 0.0,
                frame: 10,
                team_is_team_0: false,
                player: Some(RemoteId::Steam(2)),
                closest_approach_distance: Some(0.0),
            },
        ];
        let start = sample(
            10,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            touch_events.clone(),
            None,
            Some(true),
        );
        let active = FiftyFiftyReducer::contested_touch(&start, &touch_events, false).unwrap();

        let blue_side = sample(
            11,
            0.4,
            -220.0,
            0.0,
            -300.0,
            0.0,
            Vec::new(),
            None,
            Some(true),
        );
        let orange_side = sample(
            12,
            0.4,
            220.0,
            0.0,
            300.0,
            0.0,
            Vec::new(),
            None,
            Some(true),
        );

        assert_eq!(
            FiftyFiftyReducer::winning_team_from_ball(&active, &blue_side),
            Some(true)
        );
        assert_eq!(
            FiftyFiftyReducer::winning_team_from_ball(&active, &orange_side),
            Some(false)
        );
    }

    #[test]
    fn reducer_tracks_kickoff_wins_and_possession_after() {
        let mut reducer = FiftyFiftyReducer::new();
        reducer.apply_event(&FiftyFiftyEvent {
            start_time: 0.1,
            start_frame: 10,
            resolve_time: 0.6,
            resolve_frame: 16,
            is_kickoff: true,
            team_zero_player: Some(RemoteId::Steam(1)),
            team_one_player: Some(RemoteId::Steam(2)),
            team_zero_position: [-120.0, -40.0, 17.0],
            team_one_position: [120.0, 40.0, 17.0],
            midpoint: [0.0, 0.0, 17.0],
            plane_normal: [0.95, 0.31, 0.0],
            winning_team_is_team_0: Some(false),
            possession_team_is_team_0: Some(false),
        });

        assert_eq!(reducer.stats().count, 1);
        assert_eq!(reducer.stats().kickoff_count, 1);
        assert_eq!(reducer.stats().team_one_wins, 1);
        assert_eq!(reducer.stats().kickoff_team_one_wins, 1);
        assert_eq!(reducer.stats().team_one_possession_after_count, 1);
        assert_eq!(reducer.stats().kickoff_team_one_possession_after_count, 1);

        let blue = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
        assert_eq!(blue.count, 1);
        assert_eq!(blue.losses, 1);
        assert_eq!(blue.kickoff_losses, 1);
        assert_eq!(blue.possession_after_count, 0);

        let orange = reducer.player_stats().get(&RemoteId::Steam(2)).unwrap();
        assert_eq!(orange.count, 1);
        assert_eq!(orange.wins, 1);
        assert_eq!(orange.kickoff_wins, 1);
        assert_eq!(orange.possession_after_count, 1);
        assert_eq!(orange.kickoff_possession_after_count, 1);
    }
}
