use super::super::*;

impl<'a> ReplayProcessor<'a> {
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
