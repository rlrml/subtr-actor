use crate::{
    attribute_type_name, ReplayProcessor, SubtrActorError, SubtrActorErrorVariant,
    SubtrActorResult, TEAM_GAME_SCORE_KEY,
};

impl<'a> ReplayProcessor<'a> {
    /// Returns the team actor id for the requested side.
    pub(crate) fn get_team_actor_id_for_side(
        &self,
        is_team_0: bool,
    ) -> SubtrActorResult<boxcars::ActorId> {
        let player_id = if is_team_0 {
            self.team_zero.first()
        } else {
            self.team_one.first()
        }
        .ok_or(SubtrActorError::new(SubtrActorErrorVariant::NoGameActor))?;

        self.player_to_team
            .get(&self.get_player_actor_id(player_id)?)
            .copied()
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::ActorNotFound {
                    name: "Team",
                    player_id: player_id.clone(),
                })
            })
    }

    /// Returns the score for the requested team side.
    pub fn get_team_score(&self, is_team_0: bool) -> SubtrActorResult<i32> {
        let team_actor_id = self.get_team_actor_id_for_side(is_team_0)?;
        get_actor_attribute_matching!(
            self,
            &team_actor_id,
            TEAM_GAME_SCORE_KEY,
            boxcars::Attribute::Int
        )
        .cloned()
    }

    /// Returns `(team_zero_score, team_one_score)`.
    pub fn get_team_scores(&self) -> SubtrActorResult<(i32, i32)> {
        Ok((self.get_team_score(true)?, self.get_team_score(false)?))
    }
}
