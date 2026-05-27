use super::*;

impl<'a> ReplayProcessor<'a> {
    /// Detects goal explosions and records derived goal events for the current frame.
    pub(crate) fn update_goal_events(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.current_frame_goal_events.clear();

        let ball_explosion_data = self
            .get_object_id_for_key(BALL_EXPLOSION_DATA_KEY)
            .ok()
            .copied();
        let ball_explosion_data_extended = self
            .get_object_id_for_key(BALL_EXPLOSION_DATA_EXTENDED_KEY)
            .ok()
            .copied();

        for update in &frame.updated_actors {
            let is_ball_goal_explosion = matches!(
                &update.attribute,
                boxcars::Attribute::Explosion(_) | boxcars::Attribute::ExtendedExplosion(_)
            ) && (ball_explosion_data == Some(update.object_id)
                || ball_explosion_data_extended == Some(update.object_id));

            if !is_ball_goal_explosion {
                continue;
            }

            let score_updates = self.goal_score_updates_from_frame(frame);
            let scoring_team_is_team_0 = self
                .scoring_team_from_score_updates(score_updates)
                .or_else(|| match self.get_scored_on_team_num() {
                    Ok(0) => Some(false),
                    Ok(1) => Some(true),
                    _ => None,
                });
            let observed_scores = self
                .goal_score_tuple_from_frame(frame)
                .or_else(|| self.get_team_scores().ok());
            let scorer = scoring_team_is_team_0.and_then(|team_is_team_0| {
                self.goal_scorer_from_update(update, frame, team_is_team_0)
            });

            if self.goal_event_is_duplicate(
                frame.time,
                scoring_team_is_team_0.unwrap_or(false),
                observed_scores.map(|scores| scores.0),
                observed_scores.map(|scores| scores.1),
            ) {
                continue;
            }
            let Some(scoring_team_is_team_0) = scoring_team_is_team_0 else {
                continue;
            };
            let (team_zero_score, team_one_score) = observed_scores.map_or_else(
                || self.derived_goal_score_tuple(scoring_team_is_team_0),
                |(team_zero, team_one)| (Some(team_zero), Some(team_one)),
            );

            let event = GoalEvent {
                time: frame.time,
                frame: frame_index,
                scoring_team_is_team_0,
                player: scorer,
                team_zero_score,
                team_one_score,
            };
            self.current_frame_goal_events.push(event.clone());
            self.goal_events.push(event);
        }

        Ok(())
    }
}
