use super::wall_aerial::{
    wall_aerial_normalize_score, wall_aerial_wall_for_position, WALL_AERIAL_HIGH_CONFIDENCE,
    WALL_AERIAL_MIN_TOUCH_BALL_Z, WALL_AERIAL_MIN_TOUCH_PLAYER_Z,
};
use super::*;

const WALL_AERIAL_SHOT_MAX_WALL_CONTACT_TO_TAKEOFF_SECONDS: f32 = 2.25;
const WALL_AERIAL_SHOT_MAX_TAKEOFF_TO_SHOT_SECONDS: f32 = 2.25;
const WALL_AERIAL_SHOT_GROUND_CONTACT_MAX_PLAYER_Z: f32 = 80.0;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct WallAerialShotEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub wall: WallAerialWall,
    pub wall_contact_time: f32,
    pub wall_contact_frame: usize,
    pub takeoff_time: f32,
    pub takeoff_frame: usize,
    pub time_since_takeoff: f32,
    pub wall_contact_position: [f32; 3],
    pub takeoff_position: [f32; 3],
    pub player_position: [f32; 3],
    pub ball_position: [f32; 3],
    pub ball_speed: Option<f32>,
    pub goal_alignment: Option<f32>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct WallAerialShotStats {
    pub count: u32,
    pub high_confidence_count: u32,
    pub is_last_wall_aerial_shot: bool,
    pub last_wall_aerial_shot_time: Option<f32>,
    pub last_wall_aerial_shot_frame: Option<usize>,
    pub time_since_last_wall_aerial_shot: Option<f32>,
    pub frames_since_last_wall_aerial_shot: Option<usize>,
    pub last_confidence: Option<f32>,
    pub best_confidence: f32,
    pub cumulative_confidence: f32,
    pub cumulative_takeoff_to_shot_time: f32,
    pub cumulative_shot_height: f32,
}

impl WallAerialShotStats {
    fn average(&self, value: f32) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            value / self.count as f32
        }
    }

    pub fn average_confidence(&self) -> f32 {
        self.average(self.cumulative_confidence)
    }

    pub fn average_takeoff_to_shot_time(&self) -> f32 {
        self.average(self.cumulative_takeoff_to_shot_time)
    }

    pub fn average_shot_height(&self) -> f32 {
        self.average(self.cumulative_shot_height)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RecentWallContact {
    player: PlayerId,
    is_team_0: bool,
    wall: WallAerialWall,
    time: f32,
    frame: usize,
    position: glam::Vec3,
}

#[derive(Debug, Clone, PartialEq)]
struct ArmedWallAerialShot {
    player: PlayerId,
    is_team_0: bool,
    wall: WallAerialWall,
    wall_contact_time: f32,
    wall_contact_frame: usize,
    wall_contact_position: glam::Vec3,
    takeoff_time: f32,
    takeoff_frame: usize,
    takeoff_position: glam::Vec3,
}

#[derive(Debug, Clone, Default)]
pub struct WallAerialShotCalculator {
    player_stats: HashMap<PlayerId, WallAerialShotStats>,
    events: EventStream<WallAerialShotEvent>,
    recent_wall_contacts: HashMap<PlayerId, RecentWallContact>,
    armed_shots: HashMap<PlayerId, ArmedWallAerialShot>,
    current_last_wall_aerial_shot_player: Option<PlayerId>,
}

impl WallAerialShotCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, WallAerialShotStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[WallAerialShotEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[WallAerialShotEvent] {
        self.events.new_events()
    }

    fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_wall_aerial_shot = false;
            stats.time_since_last_wall_aerial_shot = stats
                .last_wall_aerial_shot_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_wall_aerial_shot = stats
                .last_wall_aerial_shot_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    fn update_wall_contacts_and_takeoffs(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        for player in &players.players {
            let Some(position) = player.position() else {
                continue;
            };
            if position.z <= WALL_AERIAL_SHOT_GROUND_CONTACT_MAX_PLAYER_Z {
                self.recent_wall_contacts.remove(&player.player_id);
                self.armed_shots.remove(&player.player_id);
                continue;
            }

            if let Some(wall) = wall_aerial_wall_for_position(position) {
                self.recent_wall_contacts.insert(
                    player.player_id.clone(),
                    RecentWallContact {
                        player: player.player_id.clone(),
                        is_team_0: player.is_team_0,
                        wall,
                        time: frame.time,
                        frame: frame.frame_number,
                        position,
                    },
                );
                continue;
            }

            if position.z < WALL_AERIAL_MIN_TOUCH_PLAYER_Z {
                self.armed_shots.remove(&player.player_id);
                continue;
            }

            if self.armed_shots.contains_key(&player.player_id) {
                continue;
            }

            let Some(contact) = self.recent_wall_contacts.remove(&player.player_id) else {
                continue;
            };
            if frame.time - contact.time > WALL_AERIAL_SHOT_MAX_WALL_CONTACT_TO_TAKEOFF_SECONDS {
                continue;
            }
            self.armed_shots.insert(
                player.player_id.clone(),
                ArmedWallAerialShot {
                    player: contact.player,
                    is_team_0: contact.is_team_0,
                    wall: contact.wall,
                    wall_contact_time: contact.time,
                    wall_contact_frame: contact.frame,
                    wall_contact_position: contact.position,
                    takeoff_time: frame.time,
                    takeoff_frame: frame.frame_number,
                    takeoff_position: position,
                },
            );
        }
    }

    fn prune_armed_shots(&mut self, current_time: f32) {
        self.armed_shots.retain(|_, armed| {
            current_time - armed.takeoff_time <= WALL_AERIAL_SHOT_MAX_TAKEOFF_TO_SHOT_SECONDS
        });
    }

    fn player_position(players: &PlayerFrameState, player_id: &PlayerId) -> Option<glam::Vec3> {
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .and_then(PlayerSample::position)
    }

    fn shot_event(
        &self,
        players: &PlayerFrameState,
        event: &PlayerStatEvent,
    ) -> Option<WallAerialShotEvent> {
        if event.kind != PlayerStatEventKind::Shot {
            return None;
        }
        let armed = self.armed_shots.get(&event.player)?;
        let time_since_takeoff = event.time - armed.takeoff_time;
        if !(0.0..=WALL_AERIAL_SHOT_MAX_TAKEOFF_TO_SHOT_SECONDS).contains(&time_since_takeoff) {
            return None;
        }

        let player_position = event
            .shot
            .as_ref()
            .and_then(|shot| shot.player_position.as_ref().map(vec_to_glam))
            .or_else(|| Self::player_position(players, &event.player))?;
        if player_is_on_wall(player_position) || player_position.z < WALL_AERIAL_MIN_TOUCH_PLAYER_Z
        {
            return None;
        }

        let shot = event.shot.as_ref()?;
        let ball_position = vec_to_glam(&shot.ball_position);
        if ball_position.z < WALL_AERIAL_MIN_TOUCH_BALL_Z {
            return None;
        }

        let ball_speed = shot.ball_speed;
        let goal_alignment = shot.ball_goal_alignment;
        let confidence = 0.42
            + 0.20
                * (1.0
                    - wall_aerial_normalize_score(
                        time_since_takeoff,
                        0.15,
                        WALL_AERIAL_SHOT_MAX_TAKEOFF_TO_SHOT_SECONDS,
                    ))
            + 0.16
                * wall_aerial_normalize_score(
                    player_position.z,
                    WALL_AERIAL_MIN_TOUCH_PLAYER_Z,
                    850.0,
                )
            + 0.12 * goal_alignment.unwrap_or(0.0).clamp(0.0, 1.0)
            + 0.10 * wall_aerial_normalize_score(ball_speed.unwrap_or(0.0), 600.0, 1800.0);

        Some(WallAerialShotEvent {
            time: event.time,
            frame: event.frame,
            player: event.player.clone(),
            is_team_0: event.is_team_0,
            wall: armed.wall,
            wall_contact_time: armed.wall_contact_time,
            wall_contact_frame: armed.wall_contact_frame,
            takeoff_time: armed.takeoff_time,
            takeoff_frame: armed.takeoff_frame,
            time_since_takeoff,
            wall_contact_position: armed.wall_contact_position.to_array(),
            takeoff_position: armed.takeoff_position.to_array(),
            player_position: player_position.to_array(),
            ball_position: ball_position.to_array(),
            ball_speed,
            goal_alignment,
            confidence: confidence.clamp(0.0, 1.0),
        })
    }

    fn record_event(&mut self, frame: &FrameInfo, event: WallAerialShotEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.count += 1;
        if event.confidence >= WALL_AERIAL_HIGH_CONFIDENCE {
            stats.high_confidence_count += 1;
        }
        stats.is_last_wall_aerial_shot = true;
        stats.last_wall_aerial_shot_time = Some(event.time);
        stats.last_wall_aerial_shot_frame = Some(event.frame);
        stats.time_since_last_wall_aerial_shot = Some((frame.time - event.time).max(0.0));
        stats.frames_since_last_wall_aerial_shot =
            Some(frame.frame_number.saturating_sub(event.frame));
        stats.last_confidence = Some(event.confidence);
        stats.best_confidence = stats.best_confidence.max(event.confidence);
        stats.cumulative_confidence += event.confidence;
        stats.cumulative_takeoff_to_shot_time += event.time_since_takeoff;
        stats.cumulative_shot_height += event.player_position[2];

        self.current_last_wall_aerial_shot_player = Some(event.player.clone());
        self.recent_wall_contacts.remove(&event.player);
        self.armed_shots.remove(&event.player);
        self.events.push(event);
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        frame_events: &FrameEventsState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        self.begin_sample(frame);
        if !live_play {
            self.recent_wall_contacts.clear();
            self.armed_shots.clear();
            self.current_last_wall_aerial_shot_player = None;
            return Ok(());
        }

        self.update_wall_contacts_and_takeoffs(frame, players);
        self.prune_armed_shots(frame.time);

        for stat_event in &frame_events.player_stat_events {
            if let Some(event) = self.shot_event(players, stat_event) {
                self.record_event(frame, event);
            }
        }

        if let Some(player_id) = self.current_last_wall_aerial_shot_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_wall_aerial_shot = true;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "wall_aerial_shot_tests.rs"]
mod tests;
