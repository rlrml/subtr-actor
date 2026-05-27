use serde::Serialize;

use crate::{glam_to_vec, vec_to_glam};

use super::PlayerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, ts_rs::TS)]
#[ts(export)]
pub enum PlayerStatEventKind {
    Shot,
    Save,
    Assist,
}

const SHOT_TARGET_GOAL_CENTER_Y: f32 = 5120.0;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ShotEventMetadata {
    #[ts(as = "crate::ts_bindings::Vector3fTs")]
    pub ball_position: boxcars::Vector3f,
    #[ts(as = "Option<crate::ts_bindings::Vector3fTs>")]
    pub ball_velocity: Option<boxcars::Vector3f>,
    pub ball_speed: Option<f32>,
    #[ts(as = "Option<crate::ts_bindings::Vector3fTs>")]
    pub player_position: Option<boxcars::Vector3f>,
    #[ts(as = "Option<crate::ts_bindings::Vector3fTs>")]
    pub player_velocity: Option<boxcars::Vector3f>,
    pub player_speed: Option<f32>,
    pub player_distance_to_ball: Option<f32>,
    #[ts(as = "crate::ts_bindings::Vector3fTs")]
    pub target_goal_position: boxcars::Vector3f,
    pub distance_to_goal_center: f32,
    pub distance_to_goal_line: f32,
    pub ball_goal_alignment: Option<f32>,
    pub ball_speed_toward_goal: Option<f32>,
}

impl ShotEventMetadata {
    pub fn from_rigid_bodies(
        is_team_0: bool,
        ball_body: &boxcars::RigidBody,
        player_body: Option<&boxcars::RigidBody>,
    ) -> Self {
        let ball_position = vec_to_glam(&ball_body.location);
        let ball_velocity = ball_body.linear_velocity.as_ref().map(vec_to_glam);
        let player_position = player_body.map(|body| vec_to_glam(&body.location));
        let player_velocity =
            player_body.and_then(|body| body.linear_velocity.as_ref().map(vec_to_glam));
        let target_goal_y = if is_team_0 {
            SHOT_TARGET_GOAL_CENTER_Y
        } else {
            -SHOT_TARGET_GOAL_CENTER_Y
        };
        let target_goal_position = glam::Vec3::new(0.0, target_goal_y, ball_position.z);
        let goal_vector = target_goal_position - ball_position;
        let goal_direction = goal_vector.normalize_or_zero();
        let forward_sign = if is_team_0 { 1.0 } else { -1.0 };
        let distance_to_goal_line = ((target_goal_y - ball_position.y) * forward_sign).max(0.0);
        let ball_goal_alignment = ball_velocity.map(|velocity| {
            if velocity.length_squared() <= f32::EPSILON {
                0.0
            } else {
                goal_direction.dot(velocity.normalize_or_zero())
            }
        });
        Self {
            ball_position: ball_body.location,
            ball_velocity: ball_body.linear_velocity,
            ball_speed: ball_velocity.map(|velocity| velocity.length()),
            player_position: player_body.map(|body| body.location),
            player_velocity: player_body.and_then(|body| body.linear_velocity),
            player_speed: player_velocity.map(|velocity| velocity.length()),
            player_distance_to_ball: player_position
                .map(|position| (position - ball_position).length()),
            target_goal_position: glam_to_vec(&target_goal_position),
            distance_to_goal_center: goal_vector.length(),
            distance_to_goal_line,
            ball_goal_alignment,
            ball_speed_toward_goal: ball_velocity.map(|velocity| goal_direction.dot(velocity)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerStatEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub kind: PlayerStatEventKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shot: Option<ShotEventMetadata>,
}
