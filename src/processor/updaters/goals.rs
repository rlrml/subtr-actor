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

    fn goal_event_is_duplicate(
        &self,
        frame_time: f32,
        scoring_team_is_team_0: bool,
        team_zero_score: Option<i32>,
        team_one_score: Option<i32>,
    ) -> bool {
        const GOAL_EVENT_DEDUPE_WINDOW_SECONDS: f32 = 3.0;

        self.goal_events
            .last()
            .map(|event| {
                match (
                    team_zero_score,
                    team_one_score,
                    event.team_zero_score,
                    event.team_one_score,
                ) {
                    (
                        Some(team_zero),
                        Some(team_one),
                        Some(prev_team_zero),
                        Some(prev_team_one),
                    ) => team_zero == prev_team_zero && team_one == prev_team_one,
                    _ => {
                        event.scoring_team_is_team_0 == scoring_team_is_team_0
                            && (frame_time - event.time).abs() <= GOAL_EVENT_DEDUPE_WINDOW_SECONDS
                    }
                }
            })
            .unwrap_or(false)
    }

    fn derived_goal_score_tuple(&self, scoring_team_is_team_0: bool) -> (Option<i32>, Option<i32>) {
        let (mut team_zero_goals, mut team_one_goals) = self.last_known_goal_score_tuple();
        if scoring_team_is_team_0 {
            team_zero_goals += 1;
        } else {
            team_one_goals += 1;
        }
        (Some(team_zero_goals), Some(team_one_goals))
    }

    fn last_known_goal_score_tuple(&self) -> (i32, i32) {
        self.goal_events
            .last()
            .and_then(|event| event.team_zero_score.zip(event.team_one_score))
            .unwrap_or((0, 0))
    }

    fn goal_score_updates_from_frame(
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

    fn scoring_team_from_score_updates(
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

    fn goal_score_tuple_from_frame(&self, frame: &boxcars::Frame) -> Option<(i32, i32)> {
        let (previous_team_zero, previous_team_one) = self.last_known_goal_score_tuple();
        let (team_zero_score, team_one_score) = self.goal_score_updates_from_frame(frame)?;

        Some((
            team_zero_score.unwrap_or(previous_team_zero),
            team_one_score.unwrap_or(previous_team_one),
        ))
    }

    fn goal_scorer_from_update(
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
