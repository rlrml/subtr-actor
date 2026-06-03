use super::*;

const WAVEDASH_MAX_DODGE_TO_LANDING_SECONDS: f32 = 0.35;
const WAVEDASH_MAX_CANDIDATE_SECONDS: f32 = 0.5;
const WAVEDASH_MIN_DODGE_START_Z: f32 = PLAYER_GROUND_Z_THRESHOLD + 8.0;
const WAVEDASH_MAX_DODGE_START_Z: f32 = 320.0;
const WAVEDASH_MIN_LANDING_UPRIGHTNESS: f32 = 0.15;
const WAVEDASH_MIN_CONFIDENCE: f32 = 0.45;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct WavedashEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub dodge_time: f32,
    pub dodge_frame: usize,
    pub time_since_dodge: f32,
    pub dodge_position: [f32; 3],
    pub landing_position: [f32; 3],
    pub start_speed: f32,
    pub landing_speed: f32,
    pub horizontal_speed_gain: f32,
    pub landing_uprightness: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveWavedashCandidate {
    is_team_0: bool,
    dodge_time: f32,
    dodge_frame: usize,
    dodge_position: [f32; 3],
    start_horizontal_speed: f32,
    start_height: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WavedashCalculator {
    stats: WavedashStatsAccumulator,
    events: EventStream<WavedashEvent>,
    active_candidates: HashMap<PlayerId, ActiveWavedashCandidate>,
    previous_dodge_active: HashMap<PlayerId, bool>,
}

impl WavedashCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, WavedashStats> {
        self.stats.player_stats()
    }

    pub fn events(&self) -> &[WavedashEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[WavedashEvent] {
        self.events.new_events()
    }

    fn horizontal_speed(player: &PlayerSample) -> f32 {
        player
            .velocity()
            .map(|velocity| velocity.truncate().length())
            .unwrap_or(0.0)
    }

    fn normalize_score(value: f32, min_value: f32, max_value: f32) -> f32 {
        if max_value <= min_value {
            return 0.0;
        }

        ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
    }

    fn landing_uprightness(player: &PlayerSample) -> Option<f32> {
        let rigid_body = player.rigid_body.as_ref()?;
        Some((quat_to_glam(&rigid_body.rotation) * glam::Vec3::Z).dot(glam::Vec3::Z))
    }

    fn maybe_start_candidate(&mut self, frame: &FrameInfo, player: &PlayerSample) {
        let was_dodge_active = self
            .previous_dodge_active
            .insert(player.player_id.clone(), player.dodge_active)
            .unwrap_or(false);
        if !player.dodge_active || was_dodge_active {
            return;
        }

        let Some(position) = player.position() else {
            return;
        };
        if !(WAVEDASH_MIN_DODGE_START_Z..=WAVEDASH_MAX_DODGE_START_Z).contains(&position.z) {
            return;
        }

        self.active_candidates.insert(
            player.player_id.clone(),
            ActiveWavedashCandidate {
                is_team_0: player.is_team_0,
                dodge_time: frame.time,
                dodge_frame: frame.frame_number,
                dodge_position: position.to_array(),
                start_horizontal_speed: Self::horizontal_speed(player),
                start_height: position.z,
            },
        );
    }

    fn candidate_event(
        player_id: &PlayerId,
        candidate: ActiveWavedashCandidate,
        frame: &FrameInfo,
        player: &PlayerSample,
    ) -> Option<WavedashEvent> {
        let landing_position = player.position()?;
        if landing_position.z > PLAYER_GROUND_Z_THRESHOLD {
            return None;
        }

        let time_since_dodge = frame.time - candidate.dodge_time;
        if !(0.0..=WAVEDASH_MAX_DODGE_TO_LANDING_SECONDS).contains(&time_since_dodge) {
            return None;
        }

        let landing_uprightness = Self::landing_uprightness(player)?;
        if landing_uprightness < WAVEDASH_MIN_LANDING_UPRIGHTNESS {
            return None;
        }

        let landing_speed = Self::horizontal_speed(player);
        let horizontal_speed_gain = landing_speed - candidate.start_horizontal_speed;
        let timing_score = 1.0
            - Self::normalize_score(
                time_since_dodge,
                0.08,
                WAVEDASH_MAX_DODGE_TO_LANDING_SECONDS,
            );
        let height_score =
            1.0 - Self::normalize_score(candidate.start_height, WAVEDASH_MIN_DODGE_START_Z, 220.0);
        let speed_score = Self::normalize_score(horizontal_speed_gain, 80.0, 550.0)
            .max(Self::normalize_score(landing_speed, 900.0, 1800.0) * 0.8);
        let upright_score = Self::normalize_score(landing_uprightness, 0.3, 0.95);
        let confidence =
            0.35 * timing_score + 0.25 * height_score + 0.25 * speed_score + 0.15 * upright_score;

        if confidence < WAVEDASH_MIN_CONFIDENCE {
            return None;
        }

        Some(WavedashEvent {
            time: frame.time,
            frame: frame.frame_number,
            player: player_id.clone(),
            is_team_0: candidate.is_team_0,
            dodge_time: candidate.dodge_time,
            dodge_frame: candidate.dodge_frame,
            time_since_dodge,
            dodge_position: candidate.dodge_position,
            landing_position: landing_position.to_array(),
            start_speed: candidate.start_horizontal_speed,
            landing_speed,
            horizontal_speed_gain,
            landing_uprightness,
            confidence,
        })
    }

    fn apply_event(&mut self, event: WavedashEvent) {
        self.stats.apply_event(&event);
        self.events.push(event);
    }

    fn update_active_candidates(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        let mut finished = Vec::new();
        let mut visible_players = HashSet::new();

        for player in &players.players {
            visible_players.insert(player.player_id.clone());
            self.maybe_start_candidate(frame, player);

            let Some(candidate) = self.active_candidates.get(&player.player_id).cloned() else {
                continue;
            };
            if frame.time - candidate.dodge_time > WAVEDASH_MAX_CANDIDATE_SECONDS {
                finished.push((player.player_id.clone(), None));
                continue;
            }
            if let Some(event) = Self::candidate_event(&player.player_id, candidate, frame, player)
            {
                finished.push((player.player_id.clone(), Some(event)));
            }
        }

        for (player_id, event) in finished {
            self.active_candidates.remove(&player_id);
            if let Some(event) = event {
                self.apply_event(event);
            }
        }

        self.active_candidates
            .retain(|player_id, _| visible_players.contains(player_id));
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if !live_play {
            self.active_candidates.clear();
            self.stats.reset_current_last_event_marker();
            return Ok(());
        }

        self.stats.begin_sample(frame);
        self.update_active_candidates(frame, players);

        Ok(())
    }
}

#[cfg(test)]
#[path = "wavedash_tests.rs"]
mod tests;
