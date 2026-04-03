use crate::*;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FlipResetEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    /// Heuristic confidence in the range `[0.0, 1.0]`.
    pub confidence: f32,
    /// Ball position relative to the car in the car's local frame.
    #[ts(as = "crate::ts_bindings::Vector3fTs")]
    pub local_ball_position: boxcars::Vector3f,
    /// Motion-aware closest approach distance used for touch attribution.
    pub closest_approach_distance: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DodgeRefreshedEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub counter_value: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PostWallDodgeEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub wall_contact_time: f32,
    pub time_since_wall_contact: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FlipResetFollowupDodgeEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct FlipResetTouchFeatures {
    player_position: glam::Vec3,
    ball_position: glam::Vec3,
    center_distance: f32,
    local_ball_position: glam::Vec3,
    scaled_touch_distance: f32,
    underside_alignment: f32,
}

fn scale_factor_for_positions(ball_position: glam::Vec3, player_position: glam::Vec3) -> f32 {
    if ball_position
        .truncate()
        .abs()
        .max(player_position.truncate().abs())
        .max_element()
        < 200.0
    {
        100.0
    } else {
        1.0
    }
}

fn build_touch_features(
    ball_body: &boxcars::RigidBody,
    player_body: &boxcars::RigidBody,
    closest_approach_distance: f32,
) -> Option<FlipResetTouchFeatures> {
    let raw_ball_position = vec_to_glam(&ball_body.location);
    let raw_player_position = vec_to_glam(&player_body.location);
    let scale_factor = scale_factor_for_positions(raw_ball_position, raw_player_position);

    let ball_position = raw_ball_position * scale_factor;
    let player_position = raw_player_position * scale_factor;
    let relative_ball_position = ball_position - player_position;
    let center_distance = relative_ball_position.length();
    if !center_distance.is_finite() || center_distance <= 30.0 || center_distance >= 550.0 {
        return None;
    }

    let player_rotation = quat_to_glam(&player_body.rotation);
    let local_ball_position = player_rotation.inverse() * relative_ball_position;
    let car_up = (player_rotation * glam::Vec3::Z).normalize_or_zero();
    let underside_alignment = (-car_up).dot(relative_ball_position.normalize_or_zero());
    let scaled_touch_distance = closest_approach_distance * scale_factor;

    Some(FlipResetTouchFeatures {
        player_position,
        ball_position,
        center_distance,
        local_ball_position,
        scaled_touch_distance,
        underside_alignment,
    })
}

fn flip_reset_confidence(features: &FlipResetTouchFeatures) -> f32 {
    let below_car_score = (-features.local_ball_position.z / 180.0).clamp(0.0, 1.0);
    let alignment_score = ((features.underside_alignment - 0.45) / 0.50).clamp(0.0, 1.0);
    let touch_score = (1.0 - ((features.scaled_touch_distance - 20.0) / 220.0)).clamp(0.0, 1.0);
    let height_score = ((features.player_position.z - 70.0) / 500.0).clamp(0.0, 1.0);
    let footprint_score = (1.0
        - (features.local_ball_position.x.abs() / 260.0).clamp(0.0, 1.0) * 0.5
        - (features.local_ball_position.y.abs() / 260.0).clamp(0.0, 1.0) * 0.5)
        .clamp(0.0, 1.0);
    0.28 * below_car_score
        + 0.26 * alignment_score
        + 0.20 * touch_score
        + 0.14 * height_score
        + 0.12 * footprint_score
}

/// Returns a conservative flip-reset heuristic for a touch, if the geometry
/// looks like an underside wheel contact on an airborne car.
pub(crate) fn flip_reset_candidate(
    ball_body: &boxcars::RigidBody,
    player_body: &boxcars::RigidBody,
    closest_approach_distance: f32,
) -> Option<FlipResetHeuristic> {
    const MIN_PLAYER_HEIGHT: f32 = 95.0;
    const MIN_BALL_HEIGHT: f32 = 80.0;
    const MAX_TOUCH_DISTANCE: f32 = 220.0;
    const MIN_UNDERSIDE_ALIGNMENT: f32 = 0.60;
    const MAX_LOCAL_FORWARD_OFFSET: f32 = 240.0;
    const MAX_LOCAL_LATERAL_OFFSET: f32 = 240.0;
    const MIN_CONFIDENCE: f32 = 0.55;

    let features = build_touch_features(ball_body, player_body, closest_approach_distance)?;
    if features.player_position.z < MIN_PLAYER_HEIGHT || features.ball_position.z < MIN_BALL_HEIGHT
    {
        return None;
    }
    if features.scaled_touch_distance > MAX_TOUCH_DISTANCE
        || features.underside_alignment < MIN_UNDERSIDE_ALIGNMENT
        || features.local_ball_position.x.abs() > MAX_LOCAL_FORWARD_OFFSET
        || features.local_ball_position.y.abs() > MAX_LOCAL_LATERAL_OFFSET
        || features.local_ball_position.z >= 10.0
    {
        return None;
    }

    let confidence = flip_reset_confidence(&features);
    if confidence < MIN_CONFIDENCE {
        return None;
    }

    Some(FlipResetHeuristic {
        confidence,
        local_ball_position: features.local_ball_position,
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
    let features = build_touch_features(ball_body, player_body, closest_approach_distance)?;
    let confidence = flip_reset_confidence(&features);

    if confidence < 0.45
        || features.local_ball_position.z >= 20.0
        || features.underside_alignment < 0.25
    {
        return None;
    }

    Some(FlipResetHeuristic {
        confidence,
        local_ball_position: features.local_ball_position,
    })
}

fn flip_reset_proximity_candidate(
    ball_body: &boxcars::RigidBody,
    player_body: &boxcars::RigidBody,
) -> Option<FlipResetHeuristic> {
    const MIN_PLAYER_HEIGHT: f32 = 95.0;
    const MIN_BALL_HEIGHT: f32 = 80.0;
    const MAX_CENTER_DISTANCE: f32 = 110.0;
    const MIN_UNDERSIDE_ALIGNMENT: f32 = 0.52;
    const MAX_LOCAL_FORWARD_OFFSET: f32 = 260.0;
    const MAX_LOCAL_LATERAL_OFFSET: f32 = 260.0;
    const MIN_CONFIDENCE: f32 = 0.52;

    let raw_ball_position = vec_to_glam(&ball_body.location);
    let raw_player_position = vec_to_glam(&player_body.location);
    let scale_factor = scale_factor_for_positions(raw_ball_position, raw_player_position);
    let center_distance = (raw_ball_position - raw_player_position).length() * scale_factor;
    let features = build_touch_features(ball_body, player_body, center_distance / scale_factor)?;
    if features.player_position.z < MIN_PLAYER_HEIGHT || features.ball_position.z < MIN_BALL_HEIGHT
    {
        return None;
    }
    if features.center_distance > MAX_CENTER_DISTANCE
        || features.underside_alignment < MIN_UNDERSIDE_ALIGNMENT
        || features.local_ball_position.x.abs() > MAX_LOCAL_FORWARD_OFFSET
        || features.local_ball_position.y.abs() > MAX_LOCAL_LATERAL_OFFSET
        || features.local_ball_position.z >= 15.0
    {
        return None;
    }

    let confidence = flip_reset_confidence(&features);
    if confidence < MIN_CONFIDENCE {
        return None;
    }

    Some(FlipResetHeuristic {
        confidence,
        local_ball_position: features.local_ball_position,
    })
}

#[derive(Debug, Clone, Default)]
pub struct FlipResetTracker {
    flip_reset_events: Vec<FlipResetEvent>,
    current_frame_flip_reset_events: Vec<FlipResetEvent>,
    post_wall_dodge_events: Vec<PostWallDodgeEvent>,
    current_frame_post_wall_dodge_events: Vec<PostWallDodgeEvent>,
    flip_reset_followup_dodge_events: Vec<FlipResetFollowupDodgeEvent>,
    current_frame_flip_reset_followup_dodge_events: Vec<FlipResetFollowupDodgeEvent>,
    recent_wall_contact_time: HashMap<PlayerId, f32>,
    recent_flip_reset_candidates: HashMap<PlayerId, FlipResetEvent>,
    recent_flip_reset_proximity_event_time: HashMap<PlayerId, f32>,
    current_frame_dodge_rising_edges: Vec<PlayerId>,
    previous_dodge_active: HashMap<PlayerId, bool>,
}

impl FlipResetTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.update_flip_reset_events(processor, frame.time, frame_index)?;
        self.update_dodge_rising_edges(processor)?;
        self.update_flip_reset_followup_dodge_events(processor, frame, frame_index)?;
        self.update_post_wall_dodge_events(processor, frame, frame_index)?;
        Ok(())
    }

    pub fn flip_reset_events(&self) -> &[FlipResetEvent] {
        &self.flip_reset_events
    }

    pub fn current_frame_flip_reset_events(&self) -> &[FlipResetEvent] {
        &self.current_frame_flip_reset_events
    }

    pub fn post_wall_dodge_events(&self) -> &[PostWallDodgeEvent] {
        &self.post_wall_dodge_events
    }

    pub fn current_frame_post_wall_dodge_events(&self) -> &[PostWallDodgeEvent] {
        &self.current_frame_post_wall_dodge_events
    }

    pub fn flip_reset_followup_dodge_events(&self) -> &[FlipResetFollowupDodgeEvent] {
        &self.flip_reset_followup_dodge_events
    }

    pub fn current_frame_flip_reset_followup_dodge_events(&self) -> &[FlipResetFollowupDodgeEvent] {
        &self.current_frame_flip_reset_followup_dodge_events
    }

    pub fn into_events(
        self,
    ) -> (
        Vec<FlipResetEvent>,
        Vec<PostWallDodgeEvent>,
        Vec<FlipResetFollowupDodgeEvent>,
    ) {
        (
            self.flip_reset_events,
            self.post_wall_dodge_events,
            self.flip_reset_followup_dodge_events,
        )
    }

    fn build_flip_reset_event(
        &self,
        processor: &ReplayProcessor,
        touch_event: &TouchEvent,
        frame_index: usize,
    ) -> Option<FlipResetEvent> {
        let player = touch_event.player.as_ref()?;
        let closest_approach_distance = touch_event.closest_approach_distance?;
        let ball_rigid_body = processor.get_normalized_ball_rigid_body().ok()?;
        let player_rigid_body = processor.get_normalized_player_rigid_body(player).ok()?;
        let heuristic = flip_reset_candidate(
            &ball_rigid_body,
            &player_rigid_body,
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

    fn build_flip_reset_event_for_player(
        &self,
        processor: &ReplayProcessor,
        player: &PlayerId,
        time: f32,
        frame_index: usize,
        is_team_0: bool,
        closest_approach_distance: f32,
    ) -> Option<FlipResetEvent> {
        let ball_rigid_body = processor.get_normalized_ball_rigid_body().ok()?;
        let player_rigid_body = processor.get_normalized_player_rigid_body(player).ok()?;
        let heuristic = flip_reset_candidate(
            &ball_rigid_body,
            &player_rigid_body,
            closest_approach_distance,
        )?;

        Some(FlipResetEvent {
            time,
            frame: frame_index,
            player: player.clone(),
            is_team_0,
            confidence: heuristic.confidence,
            local_ball_position: glam_to_vec(&heuristic.local_ball_position),
            closest_approach_distance,
        })
    }

    fn build_flip_reset_followup_touch_candidate(
        &self,
        processor: &ReplayProcessor,
        touch_event: &TouchEvent,
        frame_index: usize,
    ) -> Option<FlipResetEvent> {
        let player = touch_event.player.as_ref()?;
        let closest_approach_distance = touch_event.closest_approach_distance?;
        let ball_rigid_body = processor.get_normalized_ball_rigid_body().ok()?;
        let player_rigid_body = processor.get_normalized_player_rigid_body(player).ok()?;
        let heuristic = flip_reset_followup_touch_candidate(
            &ball_rigid_body,
            &player_rigid_body,
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

    fn build_flip_reset_proximity_event(
        &self,
        processor: &ReplayProcessor,
        player: &PlayerId,
        time: f32,
        frame_index: usize,
    ) -> Option<FlipResetEvent> {
        let ball_rigid_body = processor.get_normalized_ball_rigid_body().ok()?;
        let player_rigid_body = processor.get_normalized_player_rigid_body(player).ok()?;
        let heuristic = flip_reset_proximity_candidate(&ball_rigid_body, &player_rigid_body)?;
        let raw_ball_position = vec_to_glam(&ball_rigid_body.location);
        let raw_player_position = vec_to_glam(&player_rigid_body.location);
        let scale_factor = scale_factor_for_positions(raw_ball_position, raw_player_position);
        let closest_approach_distance =
            (raw_ball_position - raw_player_position).length() * scale_factor;

        Some(FlipResetEvent {
            time,
            frame: frame_index,
            player: player.clone(),
            is_team_0: processor.get_player_is_team_0(player).unwrap_or(false),
            confidence: heuristic.confidence,
            local_ball_position: glam_to_vec(&heuristic.local_ball_position),
            closest_approach_distance,
        })
    }

    fn update_flip_reset_events(
        &mut self,
        processor: &ReplayProcessor,
        current_time: f32,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        const PROXIMITY_EVENT_DEBOUNCE_SECONDS: f32 = 0.35;

        self.current_frame_flip_reset_events.clear();
        for touch_event in processor.current_frame_touch_events() {
            let event = self
                .build_flip_reset_event(processor, touch_event, frame_index)
                .or_else(|| {
                    let ball_rigid_body = processor.get_normalized_ball_rigid_body().ok()?;
                    let ball_position = vec_to_glam(&ball_rigid_body.location);
                    processor
                        .iter_player_ids_in_order()
                        .filter(|player| {
                            processor.get_player_is_team_0(player).ok()
                                == Some(touch_event.team_is_team_0)
                        })
                        .filter_map(|player| {
                            let player_rigid_body =
                                processor.get_normalized_player_rigid_body(player).ok()?;
                            let player_position = vec_to_glam(&player_rigid_body.location);
                            let fallback_touch_distance =
                                (ball_position - player_position).length();
                            self.build_flip_reset_event_for_player(
                                processor,
                                player,
                                touch_event.time,
                                frame_index,
                                touch_event.team_is_team_0,
                                fallback_touch_distance,
                            )
                        })
                        .max_by(|left, right| {
                            left.confidence
                                .partial_cmp(&right.confidence)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                });
            let Some(event) = event else {
                continue;
            };
            self.current_frame_flip_reset_events.push(event.clone());
            self.flip_reset_events.push(event);
        }

        for player in processor.iter_player_ids_in_order() {
            let already_emitted_this_frame = self
                .current_frame_flip_reset_events
                .iter()
                .any(|event| &event.player == player);
            if already_emitted_this_frame {
                continue;
            }
            if self
                .recent_flip_reset_proximity_event_time
                .get(player)
                .map(|previous_time| {
                    current_time - previous_time < PROXIMITY_EVENT_DEBOUNCE_SECONDS
                })
                .unwrap_or(false)
            {
                continue;
            }
            let Some(event) =
                self.build_flip_reset_proximity_event(processor, player, current_time, frame_index)
            else {
                continue;
            };
            self.recent_flip_reset_proximity_event_time
                .insert(player.clone(), current_time);
            self.current_frame_flip_reset_events.push(event.clone());
            self.flip_reset_events.push(event);
        }
        Ok(())
    }

    fn update_dodge_rising_edges(&mut self, processor: &ReplayProcessor) -> SubtrActorResult<()> {
        self.current_frame_dodge_rising_edges.clear();
        let player_ids: Vec<_> = processor.iter_player_ids_in_order().cloned().collect();

        for player_id in player_ids {
            let dodge_active = processor.get_dodge_active(&player_id).unwrap_or(0) % 2 == 1;
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

    fn update_post_wall_dodge_events(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        const MIN_DELAY_AFTER_WALL_SECONDS: f32 = 0.20;
        const MAX_DELAY_AFTER_WALL_SECONDS: f32 = 1.10;

        self.current_frame_post_wall_dodge_events.clear();
        let current_time = frame.time;
        let player_ids: Vec<_> = processor.iter_player_ids_in_order().cloned().collect();

        for player_id in &player_ids {
            let Ok(player_rigid_body) = processor.get_normalized_player_rigid_body(player_id)
            else {
                self.previous_dodge_active.remove(player_id);
                continue;
            };

            let is_grounded = Self::player_is_grounded_for_wall_sequence(&player_rigid_body);
            if is_grounded {
                self.recent_wall_contact_time.remove(player_id);
            } else if Self::player_is_touching_wall(&player_rigid_body) {
                self.recent_wall_contact_time
                    .insert(player_id.clone(), current_time);
            }
        }

        for player_id in &self.current_frame_dodge_rising_edges {
            let Ok(player_rigid_body) = processor.get_normalized_player_rigid_body(player_id)
            else {
                continue;
            };
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
                is_team_0: processor.get_player_is_team_0(player_id).unwrap_or(false),
                wall_contact_time,
                time_since_wall_contact,
            };
            self.current_frame_post_wall_dodge_events
                .push(event.clone());
            self.post_wall_dodge_events.push(event);
        }

        Ok(())
    }

    fn update_flip_reset_followup_dodge_events(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        const MIN_DELAY_AFTER_CANDIDATE_TOUCH_SECONDS: f32 = 0.05;
        const MAX_DELAY_AFTER_CANDIDATE_TOUCH_SECONDS: f32 = 1.75;

        self.current_frame_flip_reset_followup_dodge_events.clear();
        let current_time = frame.time;
        let player_ids: Vec<_> = processor.iter_player_ids_in_order().cloned().collect();

        for player_id in &player_ids {
            let Ok(player_rigid_body) = processor.get_normalized_player_rigid_body(player_id)
            else {
                self.recent_flip_reset_candidates.remove(player_id);
                continue;
            };
            if Self::player_is_grounded_for_wall_sequence(&player_rigid_body) {
                self.recent_flip_reset_candidates.remove(player_id);
            }
        }

        for touch_event in processor.current_frame_touch_events() {
            let Some(event) =
                self.build_flip_reset_followup_touch_candidate(processor, touch_event, frame_index)
            else {
                continue;
            };
            self.recent_flip_reset_candidates
                .insert(event.player.clone(), event.clone());
        }

        for player_id in &self.current_frame_dodge_rising_edges {
            let Ok(player_rigid_body) = processor.get_normalized_player_rigid_body(player_id)
            else {
                continue;
            };
            if Self::player_is_grounded_for_wall_sequence(&player_rigid_body) {
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
                is_team_0: processor.get_player_is_team_0(player_id).unwrap_or(false),
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
}

impl Collector for FlipResetTracker {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
        _current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        self.on_frame(processor, frame, frame_number)?;
        Ok(TimeAdvance::NextFrame)
    }
}
