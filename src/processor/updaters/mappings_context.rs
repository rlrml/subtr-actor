use super::*;

pub(crate) struct MappingUpdateContext {
    pub(super) player_type_actor_ids: Vec<boxcars::ActorId>,
    pub(super) car_type_actor_ids: Vec<boxcars::ActorId>,
    pub(super) boost_type_actor_ids: Vec<boxcars::ActorId>,
    pub(super) dodge_type_actor_ids: Vec<boxcars::ActorId>,
    pub(super) jump_type_actor_ids: Vec<boxcars::ActorId>,
    pub(super) double_jump_type_actor_ids: Vec<boxcars::ActorId>,
    pub(super) unique_id_object_id: boxcars::ObjectId,
    pub(super) team_object_id: boxcars::ObjectId,
    pub(super) bot_object_id: Option<boxcars::ObjectId>,
    pub(super) player_replication_object_id: boxcars::ObjectId,
    pub(super) vehicle_object_id: boxcars::ObjectId,
}

pub(crate) fn synthetic_bot_player_id(actor_id: boxcars::ActorId) -> PlayerId {
    let actor_id = u32::try_from(actor_id.0).unwrap_or(0);
    boxcars::RemoteId::SplitScreen(u32::MAX - actor_id)
}

impl ReplayProcessor<'_> {
    pub(super) fn mapping_update_context(&self) -> SubtrActorResult<MappingUpdateContext> {
        let cached = self.cached_object_ids;
        let player_type_object_id =
            self.required_cached_object_id(cached.player_type, PLAYER_TYPE)?;
        let car_type_object_id = self.required_cached_object_id(cached.car_type, CAR_TYPE)?;
        let boost_type_object_id = self.required_cached_object_id(cached.boost_type, BOOST_TYPE)?;
        let dodge_type_object_id = self.required_cached_object_id(cached.dodge_type, DODGE_TYPE)?;
        let jump_type_object_id = self.required_cached_object_id(cached.jump_type, JUMP_TYPE)?;
        let double_jump_type_object_id =
            self.required_cached_object_id(cached.double_jump_type, DOUBLE_JUMP_TYPE)?;
        let unique_id_object_id =
            self.required_cached_object_id(cached.unique_id, UNIQUE_ID_KEY)?;
        let team_object_id = self.required_cached_object_id(cached.team, TEAM_KEY)?;
        let player_replication_object_id =
            self.required_cached_object_id(cached.player_replication, PLAYER_REPLICATION_KEY)?;
        let vehicle_object_id = self.required_cached_object_id(cached.vehicle, VEHICLE_KEY)?;

        Ok(MappingUpdateContext {
            player_type_actor_ids: self
                .get_actor_ids_by_object_id(&player_type_object_id)
                .to_vec(),
            car_type_actor_ids: self
                .get_actor_ids_by_object_id(&car_type_object_id)
                .to_vec(),
            boost_type_actor_ids: self
                .get_actor_ids_by_object_id(&boost_type_object_id)
                .to_vec(),
            dodge_type_actor_ids: self
                .get_actor_ids_by_object_id(&dodge_type_object_id)
                .to_vec(),
            jump_type_actor_ids: self
                .get_actor_ids_by_object_id(&jump_type_object_id)
                .to_vec(),
            double_jump_type_actor_ids: self
                .get_actor_ids_by_object_id(&double_jump_type_object_id)
                .to_vec(),
            unique_id_object_id,
            team_object_id,
            bot_object_id: cached.bot,
            player_replication_object_id,
            vehicle_object_id,
        })
    }
}
