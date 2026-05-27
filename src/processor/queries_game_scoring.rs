use crate::{
    attribute_type_name, ReplayProcessor, SubtrActorError, SubtrActorErrorVariant,
    SubtrActorResult, REPLICATED_SCORED_ON_TEAM_KEY,
};

impl<'a> ReplayProcessor<'a> {
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
}
