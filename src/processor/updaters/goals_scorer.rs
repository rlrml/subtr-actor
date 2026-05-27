use super::*;

impl ReplayProcessor<'_> {
    pub(crate) fn goal_scorer_from_update(
        &self,
        goal_update: &boxcars::UpdatedAttribute,
        frame: &boxcars::Frame,
        scoring_team_is_team_0: bool,
    ) -> Option<PlayerId> {
        self.goal_scorer_from_explosion_attribute(&goal_update.attribute, scoring_team_is_team_0)
            .or_else(|| self.goal_scorer_from_frame(frame, scoring_team_is_team_0))
    }

    fn goal_scorer_from_explosion_attribute(
        &self,
        attribute: &boxcars::Attribute,
        scoring_team_is_team_0: bool,
    ) -> Option<PlayerId> {
        let actor_candidates = match attribute {
            boxcars::Attribute::Explosion(explosion) => vec![explosion.actor],
            boxcars::Attribute::ExtendedExplosion(explosion) => {
                vec![explosion.explosion.actor, explosion.secondary_actor]
            }
            _ => return None,
        };

        actor_candidates.into_iter().find_map(|actor_id| {
            self.goal_scorer_from_actor_hint(&actor_id, scoring_team_is_team_0)
        })
    }

    fn goal_scorer_from_actor_hint(
        &self,
        actor_id: &boxcars::ActorId,
        scoring_team_is_team_0: bool,
    ) -> Option<PlayerId> {
        let player_id = self
            .get_player_id_from_actor_id(actor_id)
            .ok()
            .or_else(|| self.get_player_id_from_car_id(actor_id).ok())?;
        let is_team_0 = self.get_player_is_team_0(&player_id).ok()?;
        (is_team_0 == scoring_team_is_team_0).then_some(player_id)
    }

    fn goal_scorer_from_frame(
        &self,
        frame: &boxcars::Frame,
        scoring_team_is_team_0: bool,
    ) -> Option<PlayerId> {
        let match_goals_key = *self.get_object_id_for_key(MATCH_GOALS_KEY).ok()?;

        frame
            .updated_actors
            .iter()
            .filter(|update| update.object_id == match_goals_key)
            .filter_map(|update| {
                let boxcars::Attribute::Int(goals) = update.attribute else {
                    return None;
                };
                let player_id = self.get_player_id_from_actor_id(&update.actor_id).ok()?;
                let is_team_0 = self.get_player_is_team_0(&player_id).ok()?;
                (is_team_0 == scoring_team_is_team_0).then_some((player_id, goals))
            })
            .max_by_key(|(_, goals)| *goals)
            .map(|(player_id, _)| player_id)
    }
}
