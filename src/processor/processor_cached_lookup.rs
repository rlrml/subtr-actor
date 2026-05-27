use super::*;

impl<'a> ReplayProcessor<'a> {
    pub(crate) fn required_cached_object_id(
        &self,
        object_id: Option<boxcars::ObjectId>,
        name: &'static str,
    ) -> SubtrActorResult<boxcars::ObjectId> {
        object_id
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::ObjectIdNotFound { name }))
    }
}
