use super::super::*;

impl<'a> ReplayProcessor<'a> {
    /// Returns the player's replicated display name.
    pub fn get_player_name(&self, player_id: &PlayerId) -> SubtrActorResult<String> {
        get_actor_attribute_matching!(
            self,
            &self.get_player_actor_id(player_id)?,
            PLAYER_NAME_KEY,
            boxcars::Attribute::String
        )
        .cloned()
    }

    /// Returns the replay object-name key for the player's team actor.
    pub fn get_player_team_key(&self, player_id: &PlayerId) -> SubtrActorResult<String> {
        let team_actor_id = self
            .player_to_team
            .get(&self.get_player_actor_id(player_id)?)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::UnknownPlayerTeam {
                    player_id: player_id.clone(),
                })
            })?;
        let state = self.get_actor_state(team_actor_id)?;
        self.object_id_to_name
            .get(&state.object_id)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::UnknownPlayerTeam {
                    player_id: player_id.clone(),
                })
            })
            .cloned()
    }

    /// Returns whether the player belongs to team 0.
    pub fn get_player_is_team_0(&self, player_id: &PlayerId) -> SubtrActorResult<bool> {
        Ok(self
            .get_player_team_key(player_id)?
            .chars()
            .last()
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::EmptyTeamName {
                    player_id: player_id.clone(),
                })
            })?
            == '0')
    }
}
