use super::*;

impl ReplayProcessor<'_> {
    pub(crate) fn kickoff_phase_active(&self) -> bool {
        let Ok(metadata_actor_id) = self.get_metadata_actor_id() else {
            return false;
        };
        let Ok(metadata_state) = self.get_actor_state(&metadata_actor_id) else {
            return false;
        };
        let metadata_attributes = &metadata_state.attributes;

        let replicated_state_name = self
            .cached_object_ids
            .replicated_state_name
            .and_then(|object_id| metadata_attributes.get(&object_id))
            .and_then(|(attribute, _)| match attribute {
                boxcars::Attribute::Int(value) => Some(*value),
                _ => None,
            });
        let replicated_game_state_time_remaining = self
            .cached_object_ids
            .replicated_game_state_time_remaining
            .and_then(|object_id| metadata_attributes.get(&object_id))
            .and_then(|(attribute, _)| match attribute {
                boxcars::Attribute::Int(value) => Some(*value),
                _ => None,
            });
        let ball_has_been_hit = self
            .cached_object_ids
            .ball_has_been_hit
            .and_then(|object_id| metadata_attributes.get(&object_id))
            .and_then(|(attribute, _)| match attribute {
                boxcars::Attribute::Boolean(value) => Some(*value),
                _ => None,
            });

        replicated_state_name == Some(55)
            || replicated_game_state_time_remaining.is_some_and(|countdown| countdown > 0)
            || ball_has_been_hit == Some(false)
    }
}
