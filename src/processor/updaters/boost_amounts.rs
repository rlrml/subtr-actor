use super::*;

impl<'a> ReplayProcessor<'a> {
    /// Updates derived boost amounts for each boost component actor in the current frame.
    pub(crate) fn update_boost_amounts(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        let kickoff_phase_active = self.kickoff_phase_active();
        let kickoff_phase_started = kickoff_phase_active && !self.kickoff_phase_active_last_frame;
        let cached = self.cached_object_ids;
        let boost_type_object_id = self.required_cached_object_id(cached.boost_type, BOOST_TYPE)?;
        let boost_replicated_object_id = cached.boost_replicated;
        let boost_amount_object_id = cached.boost_amount;
        let component_active_object_id = cached.component_active;
        let boost_actor_ids = self
            .actor_state
            .actor_ids_by_type
            .get(&boost_type_object_id)
            .cloned()
            .unwrap_or_default();
        let updates: Vec<_> = boost_actor_ids
            .into_iter()
            .map(|actor_id| {
                let actor_state = self.actor_state.actor_states.get(&actor_id).unwrap();
                let (
                    actor_amount_value,
                    last_value,
                    _,
                    derived_value,
                    has_derived_value,
                    is_active,
                ) = Self::get_current_boost_values(
                    actor_state,
                    boost_replicated_object_id,
                    boost_amount_object_id,
                    component_active_object_id,
                );
                let mut current_value = if kickoff_phase_started {
                    BOOST_KICKOFF_START_AMOUNT
                } else if actor_amount_value == last_value {
                    if has_derived_value {
                        derived_value
                    } else {
                        actor_amount_value.into()
                    }
                } else {
                    actor_amount_value.into()
                };
                if is_active {
                    current_value -= frame.delta * BOOST_USED_RAW_UNITS_PER_SECOND;
                }
                (actor_id, current_value.max(0.0), actor_amount_value)
            })
            .collect();

        for (actor_id, current_value, new_last_value) in updates {
            let actor_state = self.actor_state.actor_states.get_mut(&actor_id).unwrap();
            actor_state.set_derived_attribute(
                LAST_BOOST_AMOUNT_KEY,
                boxcars::Attribute::Byte(new_last_value),
                frame_index,
            );
            actor_state.set_derived_attribute(
                BOOST_AMOUNT_KEY,
                boxcars::Attribute::Float(current_value),
                frame_index,
            );
        }
        self.kickoff_phase_active_last_frame = kickoff_phase_active;
        Ok(())
    }
}
