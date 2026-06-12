use super::*;

impl<'a> ReplayProcessor<'a> {
    /// Returns the replicated match clock in whole seconds.
    pub fn get_seconds_remaining(&self) -> SubtrActorResult<i32> {
        let seconds_remaining_object_id =
            self.cached_object_ids.seconds_remaining.ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::ObjectIdNotFound {
                    name: SECONDS_REMAINING_KEY,
                })
            })?;
        let metadata_actor_id = self.get_metadata_actor_id()?;
        let metadata_state = self.get_actor_state(&metadata_actor_id)?;
        metadata_state
            .attributes
            .get(&seconds_remaining_object_id)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: SECONDS_REMAINING_KEY,
                })
            })
            .and_then(|(attribute, _)| attribute_match!(attribute, boxcars::Attribute::Int))
            .copied()
    }

    /// Returns the replicated game-state enum value from the metadata actor.
    pub fn get_replicated_state_name(&self) -> SubtrActorResult<i32> {
        get_actor_attribute_matching!(
            self,
            &self.get_metadata_actor_id()?,
            REPLICATED_STATE_NAME_KEY,
            boxcars::Attribute::Int
        )
        .cloned()
    }

    /// Returns the replicated kickoff countdown / time-remaining field.
    pub fn get_replicated_game_state_time_remaining(&self) -> SubtrActorResult<i32> {
        get_actor_attribute_matching!(
            self,
            &self.get_metadata_actor_id()?,
            REPLICATED_GAME_STATE_TIME_REMAINING_KEY,
            boxcars::Attribute::Int
        )
        .cloned()
    }

    /// Returns whether the replay currently reports that the ball has been hit.
    pub fn get_ball_has_been_hit(&self) -> SubtrActorResult<bool> {
        get_actor_attribute_matching!(
            self,
            &self.get_metadata_actor_id()?,
            BALL_HAS_BEEN_HIT_KEY,
            boxcars::Attribute::Boolean
        )
        .cloned()
    }

    /// Returns the ball actor's ignore-syncing flag.
    pub fn get_ignore_ball_syncing(&self) -> SubtrActorResult<bool> {
        let actor_id = self.get_ball_actor_id()?;
        get_actor_attribute_matching!(
            self,
            &actor_id,
            IGNORE_SYNCING_KEY,
            boxcars::Attribute::Boolean
        )
        .cloned()
    }

    /// Returns the current ball rigid body from live actor state.
    pub fn get_ball_rigid_body(&self) -> SubtrActorResult<&boxcars::RigidBody> {
        self.ball_actor_id
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::BallActorNotFound))
            .and_then(|actor_id| self.get_actor_rigid_body(&actor_id).map(|v| v.0))
    }

    /// Returns the current ball rigid body after spatial normalization.
    pub fn get_normalized_ball_rigid_body(&self) -> SubtrActorResult<boxcars::RigidBody> {
        self.get_ball_rigid_body()
            .map(|rigid_body| self.normalize_rigid_body(rigid_body))
    }

    /// Returns whether a non-sleeping ball rigid body is currently available.
    pub fn ball_rigid_body_exists(&self) -> SubtrActorResult<bool> {
        Ok(self
            .get_ball_rigid_body()
            .map(|rb| !rb.sleeping)
            .unwrap_or(false))
    }

    /// Returns the current ball rigid body and the frame where it was last updated.
    pub fn get_ball_rigid_body_and_updated(
        &self,
    ) -> SubtrActorResult<(&boxcars::RigidBody, &usize)> {
        self.ball_actor_id
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::BallActorNotFound))
            .and_then(|actor_id| {
                get_attribute_and_updated!(
                    self,
                    &self.get_actor_state(&actor_id)?.attributes,
                    RIGID_BODY_STATE_KEY,
                    boxcars::Attribute::RigidBody
                )
            })
    }

    /// Applies stored ball velocity forward to the requested time.
    pub fn get_velocity_applied_ball_rigid_body(
        &self,
        target_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        let (current_rigid_body, frame_index) = self.get_ball_rigid_body_and_updated()?;
        self.velocities_applied_rigid_body(current_rigid_body, *frame_index, target_time)
    }

    /// Interpolates the ball rigid body to the requested time.
    pub fn get_interpolated_ball_rigid_body(
        &self,
        time: f32,
        close_enough: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        self.get_interpolated_actor_rigid_body(&self.get_ball_actor_id()?, time, close_enough)
    }

    /// Returns the player's replicated display name.
    pub fn get_player_name(&self, player_id: &PlayerId) -> SubtrActorResult<String> {
        get_actor_attribute_matching!(
            self,
            &self.get_player_actor_id(player_id)?,
            PLAYER_NAME_KEY,
            boxcars::Attribute::String
        )
        .cloned()
    }

    fn player_stats_headers(&self) -> Option<&Vec<Vec<(String, boxcars::HeaderProp)>>> {
        self.replay
            .properties
            .iter()
            .find_map(|(key, value)| match (key.as_str(), value) {
                ("PlayerStats", boxcars::HeaderProp::Array(player_stats)) => Some(player_stats),
                _ => None,
            })
    }

    fn player_header_stats(
        &self,
        player_id: &PlayerId,
    ) -> Option<std::collections::HashMap<String, boxcars::HeaderProp>> {
        let player_stats = self.player_stats_headers()?;
        let fallback_name = String::new();
        self.get_player_name(player_id)
            .ok()
            .and_then(|name| {
                crate::replay_meta::find_player_stats(player_id, &name, player_stats).ok()
            })
            .or_else(|| {
                crate::replay_meta::find_player_stats(player_id, &fallback_name, player_stats).ok()
            })
    }

    pub(crate) fn get_player_loadout_body_name(&self, player_id: &PlayerId) -> Option<String> {
        self.player_header_stats(player_id)?
            .get("LoadoutBody")
            .and_then(|property| match property {
                boxcars::HeaderProp::Str(body_name) => Some(body_name.clone()),
                _ => None,
            })
    }

    pub(crate) fn get_player_loadout_body_id(&self, player_id: &PlayerId) -> Option<u32> {
        let player_actor_id = self.get_player_actor_id(player_id).ok()?;
        let loadout = self.player_actor_to_loadout.get(&player_actor_id)?;
        match self.get_player_is_team_0(player_id).ok() {
            Some(true) => Some(loadout.blue.body),
            Some(false) => Some(loadout.orange.body),
            None if loadout.blue.body == loadout.orange.body => Some(loadout.blue.body),
            None => Some(loadout.blue.body),
        }
    }

    /// Returns the player's replicated Rocket League camera preset, when one
    /// was captured from a `TAGame.CameraSettingsActor_TA` actor while
    /// processing frames.
    pub(crate) fn get_player_camera_settings(
        &self,
        player_id: &PlayerId,
    ) -> Option<PlayerCameraSettings> {
        let player_actor_id = self.get_player_actor_id(player_id).ok()?;
        self.player_actor_to_camera_settings
            .get(&player_actor_id)
            .copied()
    }

    pub(crate) fn get_player_car_hitbox(&self, player_id: &PlayerId) -> CarHitbox {
        car_hitbox_for_body_id_or_name(
            self.get_player_loadout_body_id(player_id),
            self.get_player_loadout_body_name(player_id).as_deref(),
        )
        .unwrap_or_else(default_car_hitbox)
    }

    fn get_player_int_stat(
        &self,
        player_id: &PlayerId,
        key: &'static str,
    ) -> SubtrActorResult<i32> {
        get_actor_attribute_matching!(
            self,
            &self.get_player_actor_id(player_id)?,
            key,
            boxcars::Attribute::Int
        )
        .cloned()
    }

    /// Returns the replay object-name key for the player's team actor.
    pub fn get_player_team_key(&self, player_id: &PlayerId) -> SubtrActorResult<String> {
        let team_actor_id = self
            .player_to_team
            .get(&self.get_player_actor_id(player_id)?)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::UnknownPlayerTeam {
                    player_id: player_id.clone(),
                })
            })?;
        let state = self.get_actor_state(team_actor_id)?;
        self.object_id_to_name
            .get(&state.object_id)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::UnknownPlayerTeam {
                    player_id: player_id.clone(),
                })
            })
            .cloned()
    }

    /// Returns whether the player belongs to team 0.
    pub fn get_player_is_team_0(&self, player_id: &PlayerId) -> SubtrActorResult<bool> {
        Ok(self
            .get_player_team_key(player_id)?
            .chars()
            .last()
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::EmptyTeamName {
                    player_id: player_id.clone(),
                })
            })?
            == '0')
    }

    /// Returns the team actor id for the requested side.
    pub(crate) fn get_team_actor_id_for_side(
        &self,
        is_team_0: bool,
    ) -> SubtrActorResult<boxcars::ActorId> {
        let player_id = if is_team_0 {
            self.team_zero.first()
        } else {
            self.team_one.first()
        }
        .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::NoGameActor))?;

        self.player_to_team
            .get(&self.get_player_actor_id(player_id)?)
            .copied()
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::ActorNotFound {
                    name: "Team",
                    player_id: player_id.clone(),
                })
            })
    }

    /// Returns the score for the requested team side.
    pub fn get_team_score(&self, is_team_0: bool) -> SubtrActorResult<i32> {
        let team_actor_id = self.get_team_actor_id_for_side(is_team_0)?;
        get_actor_attribute_matching!(
            self,
            &team_actor_id,
            TEAM_GAME_SCORE_KEY,
            boxcars::Attribute::Int
        )
        .cloned()
    }

    /// Returns `(team_zero_score, team_one_score)`.
    pub fn get_team_scores(&self) -> SubtrActorResult<(i32, i32)> {
        Ok((self.get_team_score(true)?, self.get_team_score(false)?))
    }

    /// Returns the player's current car rigid body.
    pub fn get_player_rigid_body(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<&boxcars::RigidBody> {
        self.get_car_actor_id(player_id)
            .and_then(|actor_id| self.get_actor_rigid_body(&actor_id).map(|v| v.0))
    }

    /// Returns the player's current car rigid body after spatial normalization.
    pub fn get_normalized_player_rigid_body(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        self.get_player_rigid_body(player_id)
            .map(|rigid_body| self.normalize_rigid_body(rigid_body))
    }

    /// Returns the player's current normalized car position.
    pub(crate) fn get_normalized_player_position(
        &self,
        player_id: &PlayerId,
    ) -> Option<boxcars::Vector3f> {
        self.get_normalized_player_rigid_body(player_id)
            .ok()
            .map(|rigid_body| rigid_body.location)
    }

    /// Returns the player's rigid body and the frame where it was last updated.
    pub fn get_player_rigid_body_and_updated(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<(&boxcars::RigidBody, &usize)> {
        self.get_car_actor_id(player_id).and_then(|actor_id| {
            get_attribute_and_updated!(
                self,
                &self.get_actor_state(&actor_id)?.attributes,
                RIGID_BODY_STATE_KEY,
                boxcars::Attribute::RigidBody
            )
        })
    }

    /// Like [`Self::get_player_rigid_body_and_updated`], but can use recently deleted state.
    pub fn get_player_rigid_body_and_updated_or_recently_deleted(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<(&boxcars::RigidBody, &usize)> {
        self.get_car_actor_id(player_id)
            .and_then(|actor_id| self.get_actor_rigid_body_or_recently_deleted(&actor_id))
    }

    /// Applies stored player velocity forward to the requested time.
    pub fn get_velocity_applied_player_rigid_body(
        &self,
        player_id: &PlayerId,
        target_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        let (current_rigid_body, frame_index) =
            self.get_player_rigid_body_and_updated(player_id)?;
        self.velocities_applied_rigid_body(current_rigid_body, *frame_index, target_time)
    }

    /// Interpolates the player's car rigid body to the requested time.
    pub fn get_interpolated_player_rigid_body(
        &self,
        player_id: &PlayerId,
        time: f32,
        close_enough: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        self.get_car_actor_id(player_id).and_then(|car_actor_id| {
            self.get_interpolated_actor_rigid_body(&car_actor_id, time, close_enough)
        })
    }

    /// Returns the player's current boost amount in raw replay units.
    pub fn get_player_boost_level(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        self.get_boost_actor_id(player_id).and_then(|actor_id| {
            let boost_state = self.get_actor_state(&actor_id)?;
            get_derived_attribute!(
                boost_state.derived_attributes,
                BOOST_AMOUNT_KEY,
                boxcars::Attribute::Float
            )
            .cloned()
        })
    }

    /// Returns the previous boost amount recorded for the player in raw replay units.
    pub fn get_player_last_boost_level(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        self.get_boost_actor_id(player_id).and_then(|actor_id| {
            let boost_state = self.get_actor_state(&actor_id)?;
            get_derived_attribute!(
                boost_state.derived_attributes,
                LAST_BOOST_AMOUNT_KEY,
                boxcars::Attribute::Byte
            )
            .map(|value| *value as f32)
        })
    }

    /// Returns the player's boost level scaled to the conventional 0.0-100.0 range.
    pub fn get_player_boost_percentage(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        self.get_player_boost_level(player_id)
            .map(boost_amount_to_percent)
    }

    /// Returns the player's match assists counter.
    pub fn get_player_match_assists(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        self.get_player_int_stat(player_id, MATCH_ASSISTS_KEY)
    }

    /// Returns the player's match goals counter.
    pub fn get_player_match_goals(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        self.get_player_int_stat(player_id, MATCH_GOALS_KEY)
    }

    /// Returns the player's match saves counter.
    pub fn get_player_match_saves(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        self.get_player_int_stat(player_id, MATCH_SAVES_KEY)
    }

    /// Returns the player's match score counter.
    pub fn get_player_match_score(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        self.get_player_int_stat(player_id, MATCH_SCORE_KEY)
    }

    /// Returns the player's match shots counter.
    pub fn get_player_match_shots(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        self.get_player_int_stat(player_id, MATCH_SHOTS_KEY)
    }

    /// Returns the team number recorded as the last ball-touching side.
    pub fn get_ball_hit_team_num(&self) -> SubtrActorResult<u8> {
        let ball_actor_id = self.get_ball_actor_id()?;
        get_actor_attribute_matching!(
            self,
            &ball_actor_id,
            BALL_HIT_TEAM_NUM_KEY,
            boxcars::Attribute::Byte
        )
        .cloned()
    }

    /// Returns the team number currently marked as having been scored on.
    pub fn get_scored_on_team_num(&self) -> SubtrActorResult<u8> {
        get_actor_attribute_matching!(
            self,
            &self.get_metadata_actor_id()?,
            REPLICATED_SCORED_ON_TEAM_KEY,
            boxcars::Attribute::Byte
        )
        .cloned()
    }

    /// Returns a component actor's active byte.
    pub fn get_component_active(&self, actor_id: &boxcars::ActorId) -> SubtrActorResult<u8> {
        get_actor_attribute_matching!(
            self,
            &actor_id,
            COMPONENT_ACTIVE_KEY,
            boxcars::Attribute::Byte
        )
        .cloned()
    }

    /// Returns the active byte for the player's boost component.
    pub fn get_boost_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        self.get_boost_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    /// Returns the active byte for the player's jump component.
    pub fn get_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        self.get_jump_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    /// Returns the active byte for the player's double-jump component.
    pub fn get_double_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        self.get_double_jump_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    /// Returns the active byte for the player's dodge component.
    pub fn get_dodge_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        self.get_dodge_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    /// Returns whether the player's handbrake / powerslide flag is active.
    pub fn get_powerslide_active(&self, player_id: &PlayerId) -> SubtrActorResult<bool> {
        get_actor_attribute_matching!(
            self,
            &self.get_car_actor_id(player_id)?,
            HANDBRAKE_KEY,
            boxcars::Attribute::Boolean
        )
        .cloned()
    }
}
