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
    pub fn process_long_enough_to_get_actor_ids(&mut self) -> SubtrActorResult<()> {
        let mut handler = |_p: &dyn ProcessorView, _f: &boxcars::Frame, n: usize, _current_time| {
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
}
