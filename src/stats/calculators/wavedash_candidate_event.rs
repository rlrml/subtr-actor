use super::*;

impl WavedashCalculator {
    pub(super) fn maybe_start_candidate(&mut self, frame: &FrameInfo, player: &PlayerSample) {
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
            ActiveWavedashCandidate::new(frame, player, position),
        );
    }

    pub(super) fn candidate_event(
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
        let confidence = candidate_confidence(
            time_since_dodge,
            candidate.start_height,
            landing_speed,
            horizontal_speed_gain,
            landing_uprightness,
        );
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
}

fn candidate_confidence(
    time_since_dodge: f32,
    start_height: f32,
    landing_speed: f32,
    horizontal_speed_gain: f32,
    landing_uprightness: f32,
) -> f32 {
    let timing_score = 1.0
        - WavedashCalculator::normalize_score(
            time_since_dodge,
            0.08,
            WAVEDASH_MAX_DODGE_TO_LANDING_SECONDS,
        );
    let height_score =
        1.0 - WavedashCalculator::normalize_score(start_height, WAVEDASH_MIN_DODGE_START_Z, 220.0);
    let speed_score = WavedashCalculator::normalize_score(horizontal_speed_gain, 80.0, 550.0)
        .max(WavedashCalculator::normalize_score(landing_speed, 900.0, 1800.0) * 0.8);
    let upright_score = WavedashCalculator::normalize_score(landing_uprightness, 0.3, 0.95);

    0.35 * timing_score + 0.25 * height_score + 0.25 * speed_score + 0.15 * upright_score
}
