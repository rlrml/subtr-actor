use crate::{
    attribute_type_name, ReplayProcessor, SubtrActorError, SubtrActorErrorVariant,
    SubtrActorResult, BALL_HAS_BEEN_HIT_KEY, GAME_TYPE, REPLICATED_GAME_STATE_TIME_REMAINING_KEY,
    REPLICATED_STATE_NAME_KEY, SECONDS_REMAINING_KEY,
};

impl<'a> ReplayProcessor<'a> {
    /// Returns the main game metadata actor id.
    pub fn get_metadata_actor_id(&self) -> SubtrActorResult<boxcars::ActorId> {
        if let Ok(actor_ids) = self.get_actor_ids_by_type(GAME_TYPE) {
            if let Some(actor_id) = actor_ids.first() {
                return Ok(*actor_id);
            }
        }

        let metadata_object_ids = [
            self.cached_object_ids.seconds_remaining,
            self.cached_object_ids.replicated_state_name,
            self.cached_object_ids.replicated_game_state_time_remaining,
            self.cached_object_ids.ball_has_been_hit,
        ];

        self.actor_state
            .actor_states
            .iter()
            .filter_map(|(actor_id, actor_state)| {
                let metadata_attribute_count = metadata_object_ids
                    .iter()
                    .flatten()
                    .filter(|object_id| actor_state.attributes.contains_key(object_id))
                    .count();
                (metadata_attribute_count > 0).then_some((
                    metadata_attribute_count,
                    std::cmp::Reverse(*actor_id),
                    *actor_id,
                ))
            })
            .max()
            .map(|(_, _, actor_id)| actor_id)
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::NoGameActor))
    }

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
}
