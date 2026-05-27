use super::*;

impl ReplayProcessor<'_> {
    pub(crate) fn goal_score_updates_from_frame(
        &self,
        frame: &boxcars::Frame,
    ) -> Option<(Option<i32>, Option<i32>)> {
        let team_zero_actor_id = self.get_team_actor_id_for_side(true).ok()?;
        let team_one_actor_id = self.get_team_actor_id_for_side(false).ok()?;
        let team_game_score_key = self
            .get_object_id_for_key(TEAM_GAME_SCORE_KEY)
            .ok()
            .copied();
        let team_info_score_key = self
            .get_object_id_for_key(TEAM_INFO_SCORE_KEY)
            .ok()
            .copied();
        let mut team_zero_score = None;
        let mut team_one_score = None;

        for update in &frame.updated_actors {
            let is_score_update = Some(update.object_id) == team_game_score_key
                || Some(update.object_id) == team_info_score_key;
            if !is_score_update {
                continue;
            }
            let boxcars::Attribute::Int(score) = update.attribute else {
                continue;
            };
            if update.actor_id == team_zero_actor_id {
                team_zero_score = Some(score);
            } else if update.actor_id == team_one_actor_id {
                team_one_score = Some(score);
            }
        }

        (team_zero_score.is_some() || team_one_score.is_some())
            .then_some((team_zero_score, team_one_score))
    }

    pub(crate) fn scoring_team_from_score_updates(
        &self,
        score_updates: Option<(Option<i32>, Option<i32>)>,
    ) -> Option<bool> {
        let (team_zero_score, team_one_score) = score_updates?;
        let (previous_team_zero, previous_team_one) = self.last_known_goal_score_tuple();

        match (team_zero_score, team_one_score) {
            (Some(team_zero), Some(team_one))
                if team_zero == previous_team_zero + 1 && team_one == previous_team_one =>
            {
                Some(true)
            }
            (Some(team_zero), Some(team_one))
                if team_zero == previous_team_zero && team_one == previous_team_one + 1 =>
            {
                Some(false)
            }
            (Some(team_zero), _) if team_zero == previous_team_zero + 1 => Some(true),
            (_, Some(team_one)) if team_one == previous_team_one + 1 => Some(false),
            _ => None,
        }
    }

    pub(crate) fn goal_score_tuple_from_frame(&self, frame: &boxcars::Frame) -> Option<(i32, i32)> {
        let (previous_team_zero, previous_team_one) = self.last_known_goal_score_tuple();
        let (team_zero_score, team_one_score) = self.goal_score_updates_from_frame(frame)?;

        Some((
            team_zero_score.unwrap_or(previous_team_zero),
            team_one_score.unwrap_or(previous_team_one),
        ))
    }
}
