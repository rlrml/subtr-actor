use boxcars;

pub(crate) fn get_actor_id_from_active_actor<T>(
    _: T,
    active_actor: &boxcars::ActiveActor,
) -> boxcars::ActorId {
    active_actor.actor
}

pub(crate) fn use_update_actor<T>(id: boxcars::ActorId, _: T) -> boxcars::ActorId {
    id
}
