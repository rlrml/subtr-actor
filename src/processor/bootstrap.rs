use super::*;

impl<'a> ReplayProcessor<'a> {
    /// Attempts to seed player ordering from replay headers before falling back to frames.
    pub(crate) fn set_player_order_from_headers(&mut self) -> SubtrActorResult<()> {
        let _player_stats = self
            .replay
            .properties
            .iter()
            .find(|(key, _)| key == "PlayerStats")
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PlayerStatsHeaderNotFound)
            })?;
        // XXX: implementation incomplete
        SubtrActorError::new_result(SubtrActorErrorVariant::PlayerStatsHeaderNotFound)
    }

    /// Processes the replay until it has gathered enough information to map
    /// players to their actor IDs.
    ///
    /// This function is designed to ensure that each player that participated
    /// in the game is associated with a corresponding actor ID. It runs the
    /// processing operation for approximately the first 10 seconds of the
    /// replay (10 * 30 frames), as this time span is generally sufficient to
    /// identify all players.
    ///
    /// Note that this function is particularly necessary because the headers of
    /// replays sometimes omit some players.
    ///
    /// # Errors
    ///
    /// If any error other than `FinishProcessingEarly` occurs during the
    /// processing operation, it is propagated up by this function.
    pub fn process_long_enough_to_get_actor_ids(&mut self) -> SubtrActorResult<()> {
        let mut handler = |_p: &ReplayProcessor, _f: &boxcars::Frame, n: usize, _current_time| {
            // XXX: 10 seconds should be enough to find everyone, right?
            if n > 10 * 30 {
                SubtrActorError::new_result(SubtrActorErrorVariant::FinishProcessingEarly)
            } else {
                Ok(TimeAdvance::NextFrame)
            }
        };
        let process_result = self.process(&mut handler);
        if let Some(SubtrActorErrorVariant::FinishProcessingEarly) =
            process_result.as_ref().err().map(|e| e.variant.clone())
        {
            Ok(())
        } else {
            process_result
        }
    }

    /// Rebuilds team ordering by sampling early replay frames for player/team links.
    pub(crate) fn set_player_order_from_frames(&mut self) -> SubtrActorResult<()> {
        self.process_long_enough_to_get_actor_ids()?;
        let player_to_team_0: HashMap<PlayerId, bool> = self
            .player_to_actor_id
            .keys()
            .filter_map(|player_id| {
                self.get_player_is_team_0(player_id)
                    .ok()
                    .map(|is_team_0| (player_id.clone(), is_team_0))
            })
            .collect();

        let (team_zero, team_one): (Vec<_>, Vec<_>) = player_to_team_0
            .keys()
            .cloned()
            // The unwrap here is fine because we know the get will succeed
            .partition(|player_id| *player_to_team_0.get(player_id).unwrap());

        self.team_zero = team_zero;
        self.team_one = team_one;

        self.team_zero
            .sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        self.team_one
            .sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));

        self.reset();
        Ok(())
    }

    /// Verifies that the discovered in-replay players match the stored player ordering.
    pub fn check_player_id_set(&self) -> SubtrActorResult<()> {
        let known_players =
            std::collections::HashSet::<_>::from_iter(self.player_to_actor_id.keys());
        let original_players =
            std::collections::HashSet::<_>::from_iter(self.iter_player_ids_in_order());

        if original_players != known_players {
            SubtrActorError::new_result(SubtrActorErrorVariant::InconsistentPlayerSet {
                found: known_players.into_iter().cloned().collect(),
                original: original_players.into_iter().cloned().collect(),
            })
        } else {
            Ok(())
        }
    }

    /// Processes the replay enough to get the actor IDs and then retrieves the replay metadata.
    ///
    /// This method is a convenience function that combines the functionalities
    /// of
    /// [`process_long_enough_to_get_actor_ids`](Self::process_long_enough_to_get_actor_ids)
    /// and [`get_replay_meta`](Self::get_replay_meta) into a single operation.
    /// It's meant to be used when you don't necessarily want to process the
    /// whole replay and need only the replay's metadata.
    pub fn process_and_get_replay_meta(&mut self) -> SubtrActorResult<ReplayMeta> {
        if self.player_to_actor_id.is_empty() {
            self.process_long_enough_to_get_actor_ids()?;
        }
        self.get_replay_meta()
    }

    /// Retrieves the replay metadata.
    ///
    /// This function collects information about each player in the replay and
    /// groups them by team. For each player, it gets the player's name and
    /// statistics. All this information is then wrapped into a [`ReplayMeta`]
    /// object along with the properties from the replay.
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
        let get_player_info = |player_id| {
            let fallback_name = String::new();
            let stats = self
                .get_player_name(player_id)
                .ok()
                .and_then(|name| find_player_stats(player_id, &name, player_stats).ok())
                .or_else(|| find_player_stats(player_id, &fallback_name, player_stats).ok());
            let name = self
                .get_player_name(player_id)
                .ok()
                .or_else(|| {
                    stats.as_ref().and_then(|stats| {
                        stats.get("Name").and_then(|prop| match prop {
                            boxcars::HeaderProp::Str(name) => Some(name.clone()),
                            _ => None,
                        })
                    })
                })
                .unwrap_or_else(|| format!("{player_id:?}"));
            Ok(PlayerInfo {
                name,
                stats,
                remote_id: player_id.clone(),
            })
        };
        let team_zero: SubtrActorResult<Vec<PlayerInfo>> =
            self.team_zero.iter().map(get_player_info).collect();
        let team_one: SubtrActorResult<Vec<PlayerInfo>> =
            self.team_one.iter().map(get_player_info).collect();
        Ok(ReplayMeta {
            team_zero: team_zero?,
            team_one: team_one?,
            all_headers: self.replay.properties.clone(),
        })
    }
}
