use crate::{
    attribute_type_name, ReplayProcessor, SubtrActorError, SubtrActorErrorVariant,
    SubtrActorResult, BALL_HIT_TEAM_NUM_KEY, BALL_TYPES, IGNORE_SYNCING_KEY,
};

impl<'a> ReplayProcessor<'a> {
    /// Scans the actor graph for the first actor that matches a known ball type.
    pub(crate) fn find_ball_actor(&self) -> Option<boxcars::ActorId> {
        BALL_TYPES
            .iter()
            .filter_map(|ball_type| self.iter_actors_by_type(ball_type))
            .flatten()
            .map(|(actor_id, _)| *actor_id)
            .next()
    }

    /// Returns the tracked actor id for the replay ball.
    pub fn get_ball_actor_id(&self) -> SubtrActorResult<boxcars::ActorId> {
        self.ball_actor_id.ok_or(SubtrActorError::new(
            SubtrActorErrorVariant::BallActorNotFound,
        ))
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
}
