use super::*;

pub(crate) fn live_car_actor_id(id: &PlayerId) -> SubtrActorResult<boxcars::ActorId> {
    let Some(index) = SaLiveProcessorView::player_index(id) else {
        return SubtrActorError::new_result(SubtrActorErrorVariant::PropertyNotFoundInState {
            property: "live player id",
        });
    };
    let Ok(index) = i32::try_from(index) else {
        return SubtrActorError::new_result(SubtrActorErrorVariant::PropertyNotFoundInState {
            property: "live player id",
        });
    };
    Ok(boxcars::ActorId(index))
}

pub(crate) fn live_demolish_attribute(
    attacker: &PlayerId,
    victim: &PlayerId,
    demolish: Option<&DemolishInfo>,
) -> SubtrActorResult<DemolishAttribute> {
    Ok(DemolishAttribute::Fx(boxcars::DemolishFx {
        custom_demo_flag: false,
        custom_demo_id: 0,
        attacker_flag: true,
        attacker: live_car_actor_id(attacker)?,
        victim_flag: true,
        victim: live_car_actor_id(victim)?,
        attack_velocity: demolish
            .map(|demolish| demolish.attacker_velocity)
            .unwrap_or_else(zero_vec3),
        victim_velocity: demolish
            .map(|demolish| demolish.victim_velocity)
            .unwrap_or_else(zero_vec3),
    }))
}
