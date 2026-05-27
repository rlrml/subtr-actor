use super::*;

impl ReplayProcessor<'_> {
    pub(super) fn update_synthetic_bot_player_mapping(
        &mut self,
        update: &boxcars::UpdatedAttribute,
    ) -> SubtrActorResult<()> {
        let is_bot = *attribute_match!(&update.attribute, boxcars::Attribute::Boolean)?;
        if is_bot
            && !self
                .player_to_actor_id
                .values()
                .any(|actor_id| *actor_id == update.actor_id)
        {
            self.insert_player_actor_id(synthetic_bot_player_id(update.actor_id), update.actor_id);
        }
        Ok(())
    }
}
