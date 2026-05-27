use super::*;

impl PositioningCalculator {
    pub(crate) fn event_delta<'a>(
        deltas: &'a mut HashMap<PlayerId, PositioningEvent>,
        frame: &FrameInfo,
        player_id: &PlayerId,
        is_team_0: bool,
    ) -> &'a mut PositioningEvent {
        deltas
            .entry(player_id.clone())
            .or_insert_with(|| PositioningEvent::new(frame, player_id.clone(), is_team_0))
    }

    pub(crate) fn record_goal_positioning_events(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        ball_position: glam::Vec3,
        event_deltas: &mut HashMap<PlayerId, PositioningEvent>,
    ) {
        for goal_event in &events.goal_events {
            let defending_team_is_team_0 = !goal_event.scoring_team_is_team_0;
            let normalized_ball_y = normalized_y(defending_team_is_team_0, ball_position);
            if normalized_ball_y > GOAL_CAUGHT_AHEAD_MAX_BALL_Y {
                continue;
            }

            for player in players
                .players
                .iter()
                .filter(|player| player.is_team_0 == defending_team_is_team_0)
            {
                let Some(position) = player.position() else {
                    continue;
                };
                let normalized_player_y = normalized_y(defending_team_is_team_0, position);
                if normalized_player_y < GOAL_CAUGHT_AHEAD_MIN_PLAYER_Y {
                    continue;
                }
                if normalized_player_y - normalized_ball_y < GOAL_CAUGHT_AHEAD_MIN_BALL_DELTA_Y {
                    continue;
                }

                self.player_stats
                    .entry(player.player_id.clone())
                    .or_default()
                    .times_caught_ahead_of_play_on_conceded_goals += 1;
                Self::event_delta(event_deltas, frame, &player.player_id, player.is_team_0)
                    .times_caught_ahead_of_play_on_conceded_goals += 1;
            }
        }
    }
}
