use super::*;

impl<'a> ReplayProcessor<'a> {
    /// Processes the replay enough to get the actor IDs and then retrieves the replay metadata.
    pub fn process_and_get_replay_meta(&mut self) -> SubtrActorResult<ReplayMeta> {
        if self.player_to_actor_id.is_empty() {
            self.process_long_enough_to_get_actor_ids()?;
        }
        self.get_replay_meta()
    }

    /// Retrieves the replay metadata.
    pub fn get_replay_meta(&self) -> SubtrActorResult<ReplayMeta> {
        let empty_player_stats = Vec::new();
        let player_stats = if let Some((_, boxcars::HeaderProp::Array(per_player))) = self
            .replay
            .properties
            .iter()
            .find(|(key, _)| key == "PlayerStats")
        {
            per_player
        } else {
            &empty_player_stats
        };
        let known_count = self.iter_player_ids_in_order().count();
        if player_stats.len() != known_count {
            log::warn!(
                "Replay does not have player stats for all players. encountered {:?} {:?}",
                known_count,
                player_stats.len()
            )
        }
        let team_zero: SubtrActorResult<Vec<PlayerInfo>> = self
            .team_zero
            .iter()
            .map(|player_id| self.player_info(player_id, player_stats))
            .collect();
        let team_one: SubtrActorResult<Vec<PlayerInfo>> = self
            .team_one
            .iter()
            .map(|player_id| self.player_info(player_id, player_stats))
            .collect();
        Ok(ReplayMeta {
            team_zero: team_zero?,
            team_one: team_one?,
            all_headers: self.replay.properties.clone(),
        })
    }

    fn player_info(
        &self,
        player_id: &PlayerId,
        player_stats: &Vec<Vec<(String, boxcars::HeaderProp)>>,
    ) -> SubtrActorResult<PlayerInfo> {
        let fallback_name = String::new();
        let stats = self
            .get_player_name(player_id)
            .ok()
            .and_then(|name| find_player_stats(player_id, &name, player_stats).ok())
            .or_else(|| find_player_stats(player_id, &fallback_name, player_stats).ok());
        let name = self
            .get_player_name(player_id)
            .ok()
            .or_else(|| stats.as_ref().and_then(header_name))
            .or_else(|| remote_id_display_name(player_id))
            .unwrap_or_else(|| format!("{player_id:?}"));
        Ok(PlayerInfo {
            name,
            stats,
            remote_id: player_id.clone(),
        })
    }
}

fn header_name(stats: &HashMap<String, boxcars::HeaderProp>) -> Option<String> {
    stats.get("Name").and_then(|prop| match prop {
        boxcars::HeaderProp::Str(name) => Some(name.clone()),
        _ => None,
    })
}

fn remote_id_display_name(player_id: &PlayerId) -> Option<String> {
    match player_id {
        boxcars::RemoteId::PlayStation(id) if !id.name.is_empty() => Some(id.name.clone()),
        _ => None,
    }
}
