use super::*;

impl ReplayProcessor<'_> {
    pub(super) fn get_current_boost_values(
        actor_state: &ActorState,
        boost_replicated_object_id: Option<boxcars::ObjectId>,
        boost_amount_object_id: Option<boxcars::ObjectId>,
        component_active_object_id: Option<boxcars::ObjectId>,
    ) -> (u8, u8, u8, f32, bool, bool) {
        let amount_value = boost_replicated_object_id
            .and_then(|object_id| actor_state.attributes.get(&object_id))
            .and_then(|(attribute, _)| match attribute {
                boxcars::Attribute::ReplicatedBoost(replicated_boost) => {
                    Some(replicated_boost.boost_amount)
                }
                _ => None,
            })
            .or_else(|| {
                boost_amount_object_id
                    .and_then(|object_id| actor_state.attributes.get(&object_id))
                    .and_then(|(attribute, _)| match attribute {
                        boxcars::Attribute::Byte(value) => Some(*value),
                        _ => None,
                    })
            })
            .unwrap_or(0);
        let active_value = component_active_object_id
            .and_then(|object_id| actor_state.attributes.get(&object_id))
            .and_then(|(attribute, _)| match attribute {
                boxcars::Attribute::Byte(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(0);
        let is_active = active_value % 2 == 1;
        let derived_value = actor_state
            .derived_attributes
            .get(BOOST_AMOUNT_KEY)
            .and_then(|(attribute, _)| match attribute {
                boxcars::Attribute::Float(value) => Some(*value),
                _ => None,
            });
        let last_boost_amount = actor_state
            .derived_attributes
            .get(LAST_BOOST_AMOUNT_KEY)
            .and_then(|(attribute, _)| match attribute {
                boxcars::Attribute::Byte(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(amount_value);
        (
            amount_value,
            last_boost_amount,
            active_value,
            derived_value.unwrap_or(0.0),
            derived_value.is_some(),
            is_active,
        )
    }
}
