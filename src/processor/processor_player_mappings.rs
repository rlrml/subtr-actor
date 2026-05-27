use super::*;

impl<'a> ReplayProcessor<'a> {
    pub(crate) fn sync_player_order_from_known_mappings(&mut self) {
        let player_ids: Vec<_> = self.player_to_actor_id.keys().cloned().collect();
        for player_id in player_ids {
            let already_ordered =
                self.team_zero.contains(&player_id) || self.team_one.contains(&player_id);
            if already_ordered {
                continue;
            }

            let Ok(is_team_0) = self.get_player_is_team_0(&player_id) else {
                continue;
            };
            if is_team_0 {
                self.team_zero.push(player_id);
            } else {
                self.team_one.push(player_id);
            }
        }
    }

    pub(crate) fn insert_player_actor_id(
        &mut self,
        player_id: PlayerId,
        actor_id: boxcars::ActorId,
    ) {
        let stale_player_ids = self.stale_player_ids_for_actor(&player_id, actor_id);
        for stale_player_id in stale_player_ids {
            self.player_to_actor_id.remove(&stale_player_id);
            self.team_zero
                .retain(|ordered_player_id| ordered_player_id != &stale_player_id);
            self.team_one
                .retain(|ordered_player_id| ordered_player_id != &stale_player_id);
        }

        self.player_to_actor_id.insert(player_id, actor_id);
    }

    fn stale_player_ids_for_actor(
        &self,
        player_id: &PlayerId,
        actor_id: boxcars::ActorId,
    ) -> Vec<PlayerId> {
        self.player_to_actor_id
            .iter()
            .filter(|(existing_player_id, existing_actor_id)| {
                **existing_actor_id == actor_id && *existing_player_id != player_id
            })
            .map(|(existing_player_id, _existing_actor_id)| existing_player_id.clone())
            .collect()
    }
}
