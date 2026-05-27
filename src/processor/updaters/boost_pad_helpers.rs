use super::*;

// Replay pickup sequence values are not globally unique; a pad can reuse the
// same sequence value after a real respawn. Keep this bounded by the minimum
// pad respawn time so we suppress only impossible stale replication repeats.
const MIN_BOOST_PAD_RESPAWN_SECONDS: f32 = 4.0;

impl ReplayProcessor<'_> {
    pub(crate) fn actor_is_boost_pad(&self, actor_id: &boxcars::ActorId) -> bool {
        self.get_actor_state_or_recently_deleted(actor_id)
            .ok()
            .and_then(|state| usize::try_from(state.object_id.0).ok())
            .and_then(|index| self.is_boost_pad_object.get(index))
            .copied()
            .unwrap_or(false)
    }

    pub(crate) fn boost_pad_pickup_sequence_is_recent(
        &self,
        pad_id: &str,
        sequence: u8,
        event_time: f32,
    ) -> bool {
        self.boost_pad_pickup_sequence_times
            .get(&(pad_id.to_owned(), sequence))
            .is_some_and(|last_time| {
                let elapsed = event_time - *last_time;
                (0.0..MIN_BOOST_PAD_RESPAWN_SECONDS).contains(&elapsed)
            })
    }

    pub(crate) fn get_actor_instance_name(
        &self,
        actor_id: &boxcars::ActorId,
    ) -> SubtrActorResult<String> {
        let state = self.get_actor_state_or_recently_deleted(actor_id)?;
        if let Some(name_id) = state.name_id {
            if let Some(name) = self.replay.names.get(name_id as usize) {
                return Ok(name.clone());
            }
        }
        self.object_id_to_name
            .get(&state.object_id)
            .cloned()
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::NoStateForActorId {
                    actor_id: *actor_id,
                })
            })
    }
}
