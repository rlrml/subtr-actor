use super::super::*;

impl<'a> ReplayProcessor<'a> {
    fn get_player_int_stat(
        &self,
        player_id: &PlayerId,
        key: &'static str,
    ) -> SubtrActorResult<i32> {
        get_actor_attribute_matching!(
            self,
            &self.get_player_actor_id(player_id)?,
            key,
            boxcars::Attribute::Int
        )
        .cloned()
    }

    /// Returns the player's match assists counter.
    pub fn get_player_match_assists(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        self.get_player_int_stat(player_id, MATCH_ASSISTS_KEY)
    }

    /// Returns the player's match goals counter.
    pub fn get_player_match_goals(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        self.get_player_int_stat(player_id, MATCH_GOALS_KEY)
    }

    /// Returns the player's match saves counter.
    pub fn get_player_match_saves(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        self.get_player_int_stat(player_id, MATCH_SAVES_KEY)
    }

    /// Returns the player's match score counter.
    pub fn get_player_match_score(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        self.get_player_int_stat(player_id, MATCH_SCORE_KEY)
    }

    /// Returns the player's match shots counter.
    pub fn get_player_match_shots(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        self.get_player_int_stat(player_id, MATCH_SHOTS_KEY)
    }
}
