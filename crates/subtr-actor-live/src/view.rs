use std::collections::HashSet;

use boxcars::{RemoteId, RigidBody};
use subtr_actor::{
    BoostPadEvent, CarHitbox, DemoEventSample, DemolishAttribute, DemolishInfo,
    DodgeRefreshedEvent, FrameEventsState, GoalEvent, PlayerCameraStateChange, PlayerId,
    PlayerStatEvent, ProcessorView, ReplayMeta, SubtrActorError, SubtrActorErrorVariant,
    SubtrActorResult, TouchEvent, boost_amount_to_percent,
    geometry::apply_velocities_to_rigid_body,
};

use crate::generator::{LiveEventHistory, player_car_hitbox, zero_vec3};
use crate::model::{LiveFrame, LivePlayerFrame};

/// [`ProcessorView`] over one owned [`LiveFrame`], the events derived for that
/// frame, and the accumulated live event history.
pub struct LiveProcessorView<'a> {
    replay_meta: Option<&'a ReplayMeta>,
    frame: LiveFrame,
    player_ids: Vec<PlayerId>,
    events: FrameEventsState,
    event_history: &'a LiveEventHistory,
}

impl<'a> LiveProcessorView<'a> {
    pub fn new(
        replay_meta: Option<&'a ReplayMeta>,
        frame: LiveFrame,
        events: FrameEventsState,
        event_history: &'a LiveEventHistory,
    ) -> Self {
        Self {
            replay_meta,
            player_ids: frame
                .players
                .iter()
                .map(LivePlayerFrame::canonical_player_id)
                .collect(),
            frame,
            events,
            event_history,
        }
    }

    fn missing<T>(property: &'static str) -> SubtrActorResult<T> {
        SubtrActorError::new_result(SubtrActorErrorVariant::PropertyNotFoundInState { property })
    }

    fn player(&self, player_id: &PlayerId) -> SubtrActorResult<&LivePlayerFrame> {
        self.frame
            .players
            .iter()
            .find(|player| &player.canonical_player_id() == player_id)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "live player",
                })
            })
    }
}

pub fn live_car_actor_id(
    players: &[LivePlayerFrame],
    id: &PlayerId,
) -> SubtrActorResult<boxcars::ActorId> {
    let index = match id {
        RemoteId::SplitScreen(index) => Some(*index),
        _ => players
            .iter()
            .find(|player| &player.canonical_player_id() == id)
            .map(|player| player.player_index),
    };
    let Some(index) = index else {
        return SubtrActorError::new_result(SubtrActorErrorVariant::PropertyNotFoundInState {
            property: "live player id",
        });
    };
    let Ok(index) = i32::try_from(index) else {
        return SubtrActorError::new_result(SubtrActorErrorVariant::PropertyNotFoundInState {
            property: "live player id",
        });
    };
    Ok(boxcars::ActorId(index))
}

pub fn live_demolish_attribute(
    players: &[LivePlayerFrame],
    attacker: &PlayerId,
    victim: &PlayerId,
    demolish: Option<&DemolishInfo>,
) -> SubtrActorResult<DemolishAttribute> {
    Ok(DemolishAttribute::Fx(boxcars::DemolishFx {
        custom_demo_flag: false,
        custom_demo_id: 0,
        attacker_flag: true,
        attacker: live_car_actor_id(players, attacker)?,
        victim_flag: true,
        victim: live_car_actor_id(players, victim)?,
        attack_velocity: demolish
            .map(|demolish| demolish.attacker_velocity)
            .unwrap_or_else(zero_vec3),
        victim_velocity: demolish
            .map(|demolish| demolish.victim_velocity)
            .unwrap_or_else(zero_vec3),
    }))
}

fn input_axis_to_replay_byte(value: f32) -> u8 {
    ((value + 1.0) * 127.5).round().clamp(0.0, 255.0) as u8
}

impl ProcessorView for LiveProcessorView<'_> {
    fn get_replay_meta(&self) -> SubtrActorResult<ReplayMeta> {
        self.replay_meta
            .cloned()
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))
    }

    fn player_count(&self) -> usize {
        self.frame.players.len()
    }

    fn iter_player_ids_in_order(&self) -> Box<dyn Iterator<Item = &PlayerId> + '_> {
        Box::new(self.player_ids.iter())
    }

    fn current_in_game_team_player_counts(&self) -> [usize; 2] {
        let mut counts = [0, 0];
        for player in &self.frame.players {
            counts[usize::from(!player.is_team_0)] += 1;
        }
        counts
    }

    fn get_seconds_remaining(&self) -> SubtrActorResult<i32> {
        self.frame.seconds_remaining.ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                property: "seconds_remaining",
            })
        })
    }

    fn get_replicated_state_name(&self) -> SubtrActorResult<i32> {
        self.frame.game_state.ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                property: "game_state",
            })
        })
    }

    fn get_replicated_game_state_time_remaining(&self) -> SubtrActorResult<i32> {
        self.frame.kickoff_countdown_time.ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                property: "kickoff_countdown_time",
            })
        })
    }

    fn get_ball_has_been_hit(&self) -> SubtrActorResult<bool> {
        self.frame.ball_has_been_hit.ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                property: "ball_has_been_hit",
            })
        })
    }

    fn get_ignore_ball_syncing(&self) -> SubtrActorResult<bool> {
        Ok(false)
    }

    fn get_team_scores(&self) -> SubtrActorResult<(i32, i32)> {
        match (self.frame.team_zero_score, self.frame.team_one_score) {
            (Some(team_zero_score), Some(team_one_score)) => Ok((team_zero_score, team_one_score)),
            _ => Self::missing("team_scores"),
        }
    }

    fn get_ball_hit_team_num(&self) -> SubtrActorResult<u8> {
        self.frame
            .possession_team_is_team_0
            .map(|is_team_0| if is_team_0 { 0 } else { 1 })
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "possession_team",
                })
            })
    }

    fn get_scored_on_team_num(&self) -> SubtrActorResult<u8> {
        self.frame
            .scored_on_team_is_team_0
            .map(|is_team_0| if is_team_0 { 0 } else { 1 })
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "scored_on_team",
                })
            })
    }

    fn get_normalized_ball_rigid_body(&self) -> SubtrActorResult<RigidBody> {
        self.frame.ball.ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                property: "ball",
            })
        })
    }

    fn get_velocity_applied_ball_rigid_body(
        &self,
        target_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        let rigid_body = self.get_normalized_ball_rigid_body()?;
        Ok(apply_velocities_to_rigid_body(
            &rigid_body,
            target_time - self.frame.time,
        ))
    }

    fn get_interpolated_ball_rigid_body(
        &self,
        target_time: f32,
        close_enough_to_frame_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        let rigid_body = self.get_normalized_ball_rigid_body()?;
        if (target_time - self.frame.time).abs() <= close_enough_to_frame_time.abs() {
            return Ok(rigid_body);
        }
        Ok(apply_velocities_to_rigid_body(
            &rigid_body,
            target_time - self.frame.time,
        ))
    }

    fn get_normalized_player_rigid_body(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<RigidBody> {
        let player = self.player(player_id)?;
        player.rigid_body.ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                property: "player rigid body",
            })
        })
    }

    fn get_velocity_applied_player_rigid_body(
        &self,
        player_id: &PlayerId,
        target_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        let rigid_body = self.get_normalized_player_rigid_body(player_id)?;
        Ok(apply_velocities_to_rigid_body(
            &rigid_body,
            target_time - self.frame.time,
        ))
    }

    fn get_player_car_hitbox(&self, player_id: &PlayerId) -> CarHitbox {
        self.player(player_id)
            .map(player_car_hitbox)
            .unwrap_or_else(|_| subtr_actor::default_car_hitbox())
    }

    fn get_interpolated_player_rigid_body(
        &self,
        player_id: &PlayerId,
        target_time: f32,
        close_enough_to_frame_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        let rigid_body = self.get_normalized_player_rigid_body(player_id)?;
        if (target_time - self.frame.time).abs() <= close_enough_to_frame_time.abs() {
            return Ok(rigid_body);
        }
        Ok(apply_velocities_to_rigid_body(
            &rigid_body,
            target_time - self.frame.time,
        ))
    }

    fn get_player_name(&self, player_id: &PlayerId) -> SubtrActorResult<String> {
        let player = self.player(player_id)?;
        player.name.clone().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                property: "player name",
            })
        })
    }

    fn get_player_team_key(&self, player_id: &PlayerId) -> SubtrActorResult<String> {
        Ok(if self.get_player_is_team_0(player_id)? {
            "0".to_owned()
        } else {
            "1".to_owned()
        })
    }

    fn get_player_is_team_0(&self, player_id: &PlayerId) -> SubtrActorResult<bool> {
        Ok(self.player(player_id)?.is_team_0)
    }

    fn get_player_id_from_car_id(&self, actor_id: &boxcars::ActorId) -> SubtrActorResult<PlayerId> {
        let Some(index) = u32::try_from(actor_id.0).ok() else {
            return Err(SubtrActorError::new(
                SubtrActorErrorVariant::NoMatchingPlayerId {
                    actor_id: *actor_id,
                },
            ));
        };
        self.frame
            .players
            .iter()
            .find(|player| player.player_index == index)
            .map(LivePlayerFrame::canonical_player_id)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::NoMatchingPlayerId {
                    actor_id: *actor_id,
                })
            })
    }

    fn get_player_boost_level(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        Ok(self.player(player_id)?.boost_amount)
    }

    fn get_player_last_boost_level(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        Ok(self.player(player_id)?.last_boost_amount)
    }

    fn get_player_boost_percentage(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        self.get_player_boost_level(player_id)
            .map(boost_amount_to_percent)
    }

    fn get_boost_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        Ok(self.player(player_id)?.boost_active)
    }

    fn get_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        Ok(self.player(player_id)?.jump_active)
    }

    fn get_double_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        Ok(self.player(player_id)?.double_jump_active)
    }

    fn get_dodge_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        Ok(self.player(player_id)?.dodge_active)
    }

    fn get_powerslide_active(&self, player_id: &PlayerId) -> SubtrActorResult<bool> {
        Ok(self.player(player_id)?.powerslide_active)
    }

    fn get_throttle(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        let player = self.player(player_id)?;
        match &player.input {
            Some(input) => Ok(input_axis_to_replay_byte(input.throttle)),
            None => Self::missing("throttle"),
        }
    }

    fn get_steer(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        let player = self.player(player_id)?;
        match &player.input {
            Some(input) => Ok(input_axis_to_replay_byte(input.steer)),
            None => Self::missing("steer"),
        }
    }

    fn get_dodge_impulse(&self, player_id: &PlayerId) -> SubtrActorResult<(f32, f32, f32)> {
        let player = self.player(player_id)?;
        match player.dodge_impulse {
            Some([x, y, z]) => Ok((x, y, z)),
            None => Self::missing("dodge impulse"),
        }
    }

    fn get_dodge_torque(&self, player_id: &PlayerId) -> SubtrActorResult<(f32, f32, f32)> {
        let player = self.player(player_id)?;
        match player.dodge_torque {
            Some([x, y, z]) => Ok((x, y, z)),
            None => Self::missing("dodge torque"),
        }
    }

    fn get_camera_pitch(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        let player = self.player(player_id)?;
        match player.camera.as_ref().and_then(|camera| camera.pitch) {
            Some(pitch) => Ok(pitch),
            None => Self::missing("camera pitch"),
        }
    }

    fn get_camera_yaw(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        let player = self.player(player_id)?;
        match player.camera.as_ref().and_then(|camera| camera.yaw) {
            Some(yaw) => Ok(yaw),
            None => Self::missing("camera yaw"),
        }
    }

    fn get_player_match_assists(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        let player = self.player(player_id)?;
        player
            .match_stats
            .map(|stats| stats.assists)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "match assists",
                })
            })
    }

    fn get_player_match_goals(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        let player = self.player(player_id)?;
        player.match_stats.map(|stats| stats.goals).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                property: "match goals",
            })
        })
    }

    fn get_player_match_saves(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        let player = self.player(player_id)?;
        player.match_stats.map(|stats| stats.saves).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                property: "match saves",
            })
        })
    }

    fn get_player_match_score(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        let player = self.player(player_id)?;
        player.match_stats.map(|stats| stats.score).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                property: "match score",
            })
        })
    }

    fn get_player_match_shots(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        let player = self.player(player_id)?;
        player.match_stats.map(|stats| stats.shots).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                property: "match shots",
            })
        })
    }

    fn get_active_demos(&self) -> SubtrActorResult<Vec<DemolishAttribute>> {
        let mut seen = HashSet::new();
        let mut demos = Vec::new();
        for sample in &self.events.active_demos {
            if !seen.insert((sample.attacker.clone(), sample.victim.clone())) {
                continue;
            }
            let demolish = self.events.demo_events.iter().find(|demolish| {
                demolish.attacker == sample.attacker && demolish.victim == sample.victim
            });
            demos.push(live_demolish_attribute(
                &self.frame.players,
                &sample.attacker,
                &sample.victim,
                demolish,
            )?);
        }
        Ok(demos)
    }

    fn demolishes(&self) -> &[DemolishInfo] {
        &self.event_history.demo_events
    }

    fn boost_pad_events(&self) -> &[BoostPadEvent] {
        &self.event_history.boost_pad_events
    }

    fn touch_events(&self) -> &[TouchEvent] {
        &self.event_history.touch_events
    }

    fn dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent] {
        &self.event_history.dodge_refreshed_events
    }

    fn dodge_refreshed_counter_available(&self) -> bool {
        false
    }

    // Live paths do not derive coalesced camera-toggle changes.
    fn player_camera_events(&self) -> &[(PlayerId, PlayerCameraStateChange)] {
        &[]
    }

    fn player_stat_events(&self) -> &[PlayerStatEvent] {
        &self.event_history.player_stat_events
    }

    fn goal_events(&self) -> &[GoalEvent] {
        &self.event_history.goal_events
    }

    fn current_frame_active_demo_events(&self) -> &[DemoEventSample] {
        &self.events.active_demos
    }

    fn current_frame_demolish_events(&self) -> &[DemolishInfo] {
        &self.events.demo_events
    }

    fn current_frame_boost_pad_events(&self) -> &[BoostPadEvent] {
        &self.events.boost_pad_events
    }

    fn current_frame_touch_events(&self) -> &[TouchEvent] {
        &self.events.touch_events
    }

    fn current_frame_dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent] {
        &self.events.dodge_refreshed_events
    }

    fn current_frame_player_stat_events(&self) -> &[PlayerStatEvent] {
        &self.events.player_stat_events
    }

    fn current_frame_goal_events(&self) -> &[GoalEvent] {
        &self.events.goal_events
    }
}

#[cfg(test)]
#[path = "view_tests.rs"]
mod tests;
