use crate::*;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FlipResetEvent {
    pub time: f32,
    pub frame: usize,
    pub player: PlayerId,
    pub is_team_0: bool,
    /// Heuristic confidence in the range `[0.0, 1.0]`.
    pub confidence: f32,
    /// Ball position relative to the car in the car's local frame.
    pub local_ball_position: boxcars::Vector3f,
    /// Motion-aware closest approach distance used for touch attribution.
    pub closest_approach_distance: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PostWallDodgeEvent {
    pub time: f32,
    pub frame: usize,
    pub player: PlayerId,
    pub is_team_0: bool,
    pub wall_contact_time: f32,
    pub time_since_wall_contact: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FlipResetFollowupDodgeEvent {
    pub time: f32,
    pub frame: usize,
    pub player: PlayerId,
    pub is_team_0: bool,
    pub candidate_touch_time: f32,
    pub time_since_candidate_touch: f32,
    pub candidate_touch_confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FlipResetHeuristic {
    pub confidence: f32,
    pub local_ball_position: glam::Vec3,
}

/// Returns a conservative flip-reset heuristic for a touch, if the geometry
/// looks like an underside wheel contact on an airborne car.
pub(crate) fn flip_reset_candidate(
    ball_body: &boxcars::RigidBody,
    player_body: &boxcars::RigidBody,
    closest_approach_distance: f32,
) -> Option<FlipResetHeuristic> {
    const MIN_PLAYER_HEIGHT: f32 = 120.0;
    const MIN_BALL_HEIGHT: f32 = 90.0;
    const MIN_CENTER_DISTANCE: f32 = 60.0;
    const MAX_CENTER_DISTANCE: f32 = 450.0;
    const MAX_TOUCH_DISTANCE: f32 = 250.0;
    const MIN_UNDERSIDE_ALIGNMENT: f32 = 0.72;
    const MAX_LOCAL_FORWARD_OFFSET: f32 = 220.0;
    const MAX_LOCAL_LATERAL_OFFSET: f32 = 220.0;

    let raw_ball_position = vec_to_glam(&ball_body.location);
    let raw_player_position = vec_to_glam(&player_body.location);
    let scale_factor = if raw_ball_position
        .truncate()
        .abs()
        .max(raw_player_position.truncate().abs())
        .max_element()
        < 200.0
    {
        100.0
    } else {
        1.0
    };
    let closest_approach_distance = closest_approach_distance * scale_factor;

    if closest_approach_distance > MAX_TOUCH_DISTANCE {
        return None;
    }

    let ball_position = raw_ball_position * scale_factor;
    let player_position = raw_player_position * scale_factor;
    if player_position.z < MIN_PLAYER_HEIGHT || ball_position.z < MIN_BALL_HEIGHT {
        return None;
    }

    let relative_ball_position = ball_position - player_position;
    let center_distance = relative_ball_position.length();
    if !center_distance.is_finite()
        || !(MIN_CENTER_DISTANCE..=MAX_CENTER_DISTANCE).contains(&center_distance)
    {
        return None;
    }

    let player_rotation = quat_to_glam(&player_body.rotation);
    let local_ball_position = player_rotation.inverse() * relative_ball_position;
    if local_ball_position.x.abs() > MAX_LOCAL_FORWARD_OFFSET
        || local_ball_position.y.abs() > MAX_LOCAL_LATERAL_OFFSET
    {
        return None;
    }

    let car_up = (player_rotation * glam::Vec3::Z).normalize_or_zero();
    let underside_alignment = (-car_up).dot(relative_ball_position.normalize_or_zero());
    if underside_alignment < MIN_UNDERSIDE_ALIGNMENT {
        return None;
    }

    let height_score = ((player_position.z - MIN_PLAYER_HEIGHT) / 600.0).clamp(0.0, 1.0);
    let touch_score =
        (1.0 - ((closest_approach_distance - 30.0) / (MAX_TOUCH_DISTANCE - 30.0))).clamp(0.0, 1.0);
    let alignment_score = ((underside_alignment - MIN_UNDERSIDE_ALIGNMENT)
        / (1.0 - MIN_UNDERSIDE_ALIGNMENT))
        .clamp(0.0, 1.0);
    let confidence = 0.45 * alignment_score + 0.35 * touch_score + 0.20 * height_score;

    Some(FlipResetHeuristic {
        confidence,
        local_ball_position,
    })
}

/// Returns a looser underside-touch heuristic intended to be paired with a
/// later dodge event. This is less precise than [`flip_reset_candidate`] but
/// useful when the later dodge provides extra evidence.
pub(crate) fn flip_reset_followup_touch_candidate(
    ball_body: &boxcars::RigidBody,
    player_body: &boxcars::RigidBody,
    closest_approach_distance: f32,
) -> Option<FlipResetHeuristic> {
    let raw_ball_position = vec_to_glam(&ball_body.location);
    let raw_player_position = vec_to_glam(&player_body.location);
    let scale_factor = if raw_ball_position
        .truncate()
        .abs()
        .max(raw_player_position.truncate().abs())
        .max_element()
        < 200.0
    {
        100.0
    } else {
        1.0
    };
    let ball_position = raw_ball_position * scale_factor;
    let player_position = raw_player_position * scale_factor;
    let relative_ball_position = ball_position - player_position;
    let center_distance = relative_ball_position.length();
    if !center_distance.is_finite() || center_distance <= 30.0 || center_distance >= 550.0 {
        return None;
    }

    let player_rotation = quat_to_glam(&player_body.rotation);
    let local_ball_position = player_rotation.inverse() * relative_ball_position;
    let underside_alignment = (-(player_rotation * glam::Vec3::Z).normalize_or_zero())
        .dot(relative_ball_position.normalize_or_zero());
    let scaled_touch_distance = closest_approach_distance * scale_factor;
    let below_car_score = (-local_ball_position.z / 180.0).clamp(0.0, 1.0);
    let alignment_score = ((underside_alignment - 0.45) / 0.50).clamp(0.0, 1.0);
    let touch_score = (1.0 - ((scaled_touch_distance - 20.0) / 220.0)).clamp(0.0, 1.0);
    let height_score = ((player_position.z - 70.0) / 500.0).clamp(0.0, 1.0);
    let footprint_score = (1.0
        - (local_ball_position.x.abs() / 260.0).clamp(0.0, 1.0) * 0.5
        - (local_ball_position.y.abs() / 260.0).clamp(0.0, 1.0) * 0.5)
        .clamp(0.0, 1.0);
    let confidence = 0.28 * below_car_score
        + 0.26 * alignment_score
        + 0.20 * touch_score
        + 0.14 * height_score
        + 0.12 * footprint_score;

    if confidence < 0.45 || local_ball_position.z >= 20.0 || underside_alignment < 0.25 {
        return None;
    }

    Some(FlipResetHeuristic {
        confidence,
        local_ball_position,
    })
}

impl<'a> ReplayProcessor<'a> {
    fn build_flip_reset_event(
        &self,
        touch_event: &TouchEvent,
        frame_index: usize,
    ) -> Option<FlipResetEvent> {
        let player = touch_event.player.as_ref()?;
        let closest_approach_distance = touch_event.closest_approach_distance?;
        let ball_rigid_body = self.get_ball_rigid_body().ok()?;
        let player_rigid_body = self.get_player_rigid_body(player).ok()?;
        let heuristic = flip_reset_candidate(
            ball_rigid_body,
            player_rigid_body,
            closest_approach_distance,
        )?;

        Some(FlipResetEvent {
            time: touch_event.time,
            frame: frame_index,
            player: player.clone(),
            is_team_0: touch_event.team_is_team_0,
            confidence: heuristic.confidence,
            local_ball_position: glam_to_vec(&heuristic.local_ball_position),
            closest_approach_distance,
        })
    }

    fn build_flip_reset_followup_touch_candidate(
        &self,
        touch_event: &TouchEvent,
        frame_index: usize,
    ) -> Option<FlipResetEvent> {
        let player = touch_event.player.as_ref()?;
        let closest_approach_distance = touch_event.closest_approach_distance?;
        let ball_rigid_body = self.get_ball_rigid_body().ok()?;
        let player_rigid_body = self.get_player_rigid_body(player).ok()?;
        let heuristic = flip_reset_followup_touch_candidate(
            ball_rigid_body,
            player_rigid_body,
            closest_approach_distance,
        )?;

        Some(FlipResetEvent {
            time: touch_event.time,
            frame: frame_index,
            player: player.clone(),
            is_team_0: touch_event.team_is_team_0,
            confidence: heuristic.confidence,
            local_ball_position: glam_to_vec(&heuristic.local_ball_position),
            closest_approach_distance,
        })
    }

    pub(crate) fn update_flip_reset_events(
        &mut self,
        _frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.current_frame_flip_reset_events.clear();
        for touch_event in &self.current_frame_touch_events {
            let Some(event) = self.build_flip_reset_event(touch_event, frame_index) else {
                continue;
            };
            self.current_frame_flip_reset_events.push(event.clone());
            self.flip_reset_events.push(event);
        }
        Ok(())
    }

    pub(crate) fn update_dodge_rising_edges(&mut self) -> SubtrActorResult<()> {
        self.current_frame_dodge_rising_edges.clear();
        let player_ids: Vec<_> = self.iter_player_ids_in_order().cloned().collect();

        for player_id in player_ids {
            let dodge_active = self.get_dodge_active(&player_id).unwrap_or(0) % 2 == 1;
            let was_dodge_active = self
                .previous_dodge_active
                .insert(player_id.clone(), dodge_active)
                .unwrap_or(false);
            if dodge_active && !was_dodge_active {
                self.current_frame_dodge_rising_edges.push(player_id);
            }
        }

        Ok(())
    }

    fn wall_sequence_scale_factor(player_rigid_body: &boxcars::RigidBody) -> f32 {
        if player_rigid_body
            .location
            .x
            .abs()
            .max(player_rigid_body.location.y.abs())
            < 200.0
        {
            100.0
        } else {
            1.0
        }
    }

    fn player_is_grounded_for_wall_sequence(player_rigid_body: &boxcars::RigidBody) -> bool {
        player_rigid_body.location.z * Self::wall_sequence_scale_factor(player_rigid_body) <= 80.0
    }

    fn player_is_touching_wall(player_rigid_body: &boxcars::RigidBody) -> bool {
        let scale_factor = Self::wall_sequence_scale_factor(player_rigid_body);
        let location = &player_rigid_body.location;
        let x = location.x.abs() * scale_factor;
        let y = location.y.abs() * scale_factor;
        let z = location.z * scale_factor;
        z >= 120.0 && (x >= 3600.0 || y >= 5000.0)
    }

    pub(crate) fn update_post_wall_dodge_events(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        const MIN_DELAY_AFTER_WALL_SECONDS: f32 = 0.20;
        const MAX_DELAY_AFTER_WALL_SECONDS: f32 = 1.10;

        self.current_frame_post_wall_dodge_events.clear();
        let current_time = frame.time;
        let player_ids: Vec<_> = self.iter_player_ids_in_order().cloned().collect();

        for player_id in &player_ids {
            let Ok(player_rigid_body) = self.get_player_rigid_body(player_id) else {
                self.previous_dodge_active.remove(player_id);
                continue;
            };
            let player_rigid_body = *player_rigid_body;

            let is_grounded = Self::player_is_grounded_for_wall_sequence(&player_rigid_body);
            if is_grounded {
                self.recent_wall_contact_time.remove(player_id);
            } else if Self::player_is_touching_wall(&player_rigid_body) {
                self.recent_wall_contact_time
                    .insert(player_id.clone(), current_time);
            }
        }

        for player_id in &self.current_frame_dodge_rising_edges {
            let Ok(player_rigid_body) = self.get_player_rigid_body(player_id) else {
                continue;
            };
            let player_rigid_body = *player_rigid_body;
            let is_grounded = Self::player_is_grounded_for_wall_sequence(&player_rigid_body);
            if is_grounded {
                continue;
            }

            let Some(wall_contact_time) = self.recent_wall_contact_time.get(player_id).copied()
            else {
                continue;
            };
            let time_since_wall_contact = current_time - wall_contact_time;
            if !(MIN_DELAY_AFTER_WALL_SECONDS..=MAX_DELAY_AFTER_WALL_SECONDS)
                .contains(&time_since_wall_contact)
            {
                continue;
            }
            if Self::player_is_touching_wall(&player_rigid_body) {
                continue;
            }

            let event = PostWallDodgeEvent {
                time: current_time,
                frame: frame_index,
                player: player_id.clone(),
                is_team_0: self.get_player_is_team_0(player_id).unwrap_or(false),
                wall_contact_time,
                time_since_wall_contact,
            };
            self.current_frame_post_wall_dodge_events
                .push(event.clone());
            self.post_wall_dodge_events.push(event);
        }

        Ok(())
    }

    pub(crate) fn update_flip_reset_followup_dodge_events(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        const MIN_DELAY_AFTER_CANDIDATE_TOUCH_SECONDS: f32 = 0.05;
        const MAX_DELAY_AFTER_CANDIDATE_TOUCH_SECONDS: f32 = 1.75;

        self.current_frame_flip_reset_followup_dodge_events.clear();
        let current_time = frame.time;
        let player_ids: Vec<_> = self.iter_player_ids_in_order().cloned().collect();

        for player_id in &player_ids {
            let Ok(player_rigid_body) = self.get_player_rigid_body(player_id) else {
                self.recent_flip_reset_candidates.remove(player_id);
                continue;
            };
            if Self::player_is_grounded_for_wall_sequence(player_rigid_body) {
                self.recent_flip_reset_candidates.remove(player_id);
            }
        }

        for touch_event in &self.current_frame_touch_events {
            let Some(event) =
                self.build_flip_reset_followup_touch_candidate(touch_event, frame_index)
            else {
                continue;
            };
            self.recent_flip_reset_candidates
                .insert(event.player.clone(), event.clone());
        }

        for player_id in &self.current_frame_dodge_rising_edges {
            let Ok(player_rigid_body) = self.get_player_rigid_body(player_id) else {
                continue;
            };
            if Self::player_is_grounded_for_wall_sequence(player_rigid_body) {
                continue;
            }

            let Some(candidate_event) = self.recent_flip_reset_candidates.get(player_id).cloned()
            else {
                continue;
            };
            let time_since_candidate_touch = current_time - candidate_event.time;
            if !(MIN_DELAY_AFTER_CANDIDATE_TOUCH_SECONDS..=MAX_DELAY_AFTER_CANDIDATE_TOUCH_SECONDS)
                .contains(&time_since_candidate_touch)
            {
                continue;
            }

            let event = FlipResetFollowupDodgeEvent {
                time: current_time,
                frame: frame_index,
                player: player_id.clone(),
                is_team_0: self.get_player_is_team_0(player_id).unwrap_or(false),
                candidate_touch_time: candidate_event.time,
                time_since_candidate_touch,
                candidate_touch_confidence: candidate_event.confidence,
            };
            self.current_frame_flip_reset_followup_dodge_events
                .push(event.clone());
            self.flip_reset_followup_dodge_events.push(event);
            self.recent_flip_reset_candidates.remove(player_id);
        }

        Ok(())
    }

    pub fn current_frame_flip_reset_events(&self) -> &[FlipResetEvent] {
        &self.current_frame_flip_reset_events
    }

    pub fn current_frame_post_wall_dodge_events(&self) -> &[PostWallDodgeEvent] {
        &self.current_frame_post_wall_dodge_events
    }

    pub fn current_frame_flip_reset_followup_dodge_events(&self) -> &[FlipResetFollowupDodgeEvent] {
        &self.current_frame_flip_reset_followup_dodge_events
    }
}
