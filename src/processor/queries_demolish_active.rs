use crate::{
    attribute_type_name, DemolishAttribute, DemolishFormat, ReplayProcessor, SubtrActorError,
    SubtrActorErrorVariant, SubtrActorResult, CAR_TYPE, DEMOLISH_EXTENDED_KEY,
    DEMOLISH_GOAL_EXPLOSION_KEY,
};

impl<'a> ReplayProcessor<'a> {
    /// Inspects current actor state to infer which demolish attribute format is present.
    pub fn detect_demolish_format(&self) -> Option<DemolishFormat> {
        let actors = self.iter_actors_by_type_err(CAR_TYPE).ok()?;
        for (_actor_id, state) in actors {
            if get_attribute_errors_expected!(
                self,
                &state.attributes,
                DEMOLISH_EXTENDED_KEY,
                boxcars::Attribute::DemolishExtended
            )
            .is_ok()
            {
                return Some(DemolishFormat::Extended);
            }
            if get_attribute_errors_expected!(
                self,
                &state.attributes,
                DEMOLISH_GOAL_EXPLOSION_KEY,
                boxcars::Attribute::DemolishFx
            )
            .is_ok()
            {
                return Some(DemolishFormat::Fx);
            }
        }
        None
    }

    /// Returns an iterator over currently active demolish attributes in actor state.
    pub fn get_active_demos(
        &self,
    ) -> SubtrActorResult<impl Iterator<Item = DemolishAttribute> + '_> {
        let format = self.demolish_format;
        let actors: Vec<_> = self.iter_actors_by_type_err(CAR_TYPE)?.collect();
        Ok(actors
            .into_iter()
            .filter_map(move |(_actor_id, state)| match format {
                Some(DemolishFormat::Extended) => get_attribute_errors_expected!(
                    self,
                    &state.attributes,
                    DEMOLISH_EXTENDED_KEY,
                    boxcars::Attribute::DemolishExtended
                )
                .ok()
                .map(|demo| DemolishAttribute::Extended(**demo)),
                Some(DemolishFormat::Fx) => get_attribute_errors_expected!(
                    self,
                    &state.attributes,
                    DEMOLISH_GOAL_EXPLOSION_KEY,
                    boxcars::Attribute::DemolishFx
                )
                .ok()
                .map(|demo| DemolishAttribute::Fx(**demo)),
                None => None,
            }))
    }
}
