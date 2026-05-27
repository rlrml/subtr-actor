use super::touch_movement_credit::directional_ball_distances;
use super::*;

impl TouchCalculator {
    pub(crate) fn resolved_fifty_fifty_winner(
        event: &FiftyFiftyEvent,
    ) -> Option<(&PlayerId, bool)> {
        let winning_team_is_team_0 = event.winning_team_is_team_0?;
        let player = if winning_team_is_team_0 {
            event.team_zero_player.as_ref()
        } else {
            event.team_one_player.as_ref()
        }?;
        Some((player, winning_team_is_team_0))
    }

    pub(crate) fn buffer_fifty_fifty_movement(
        &mut self,
        start_frame: usize,
        delta: glam::Vec3,
        travel_distance: f32,
    ) {
        let pending = self
            .pending_fifty_fifty_movement
            .get_or_insert(PendingFiftyFiftyMovement {
                start_frame,
                travel_distance: 0.0,
                y_delta: 0.0,
            });
        if pending.start_frame != start_frame {
            *pending = PendingFiftyFiftyMovement {
                start_frame,
                travel_distance: 0.0,
                y_delta: 0.0,
            };
        }
        pending.travel_distance += travel_distance;
        pending.y_delta += delta.y;
    }

    pub(crate) fn flush_fifty_fifty_movement(&mut self, event: &FiftyFiftyEvent) {
        let Some(pending) = self.pending_fifty_fifty_movement.take() else {
            return;
        };
        if pending.start_frame != event.start_frame {
            return;
        }
        let Some((player_id, team_is_team_0)) = Self::resolved_fifty_fifty_winner(event) else {
            return;
        };

        let (advance_distance, retreat_distance) =
            directional_ball_distances(pending.y_delta, team_is_team_0);
        self.ball_movement_events.push(TouchBallMovementEvent {
            time: event.resolve_time,
            frame: event.resolve_frame,
            player: player_id.clone(),
            is_team_0: team_is_team_0,
            travel_distance: pending.travel_distance,
            advance_distance,
            retreat_distance,
        });
        self.add_ball_movement_stats(
            player_id,
            pending.travel_distance,
            advance_distance,
            retreat_distance,
        );
    }
}
