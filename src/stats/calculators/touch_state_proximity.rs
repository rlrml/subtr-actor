use super::*;

impl TouchStateCalculator {
    pub(super) fn proximity_touch_candidates(
        &self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        max_collision_distance: f32,
    ) -> Vec<TouchEvent> {
        let Some(ball) = ball.sample() else {
            return Vec::new();
        };
        let ball_position = vec_to_glam(&ball.rigid_body.location);

        let mut candidates = players
            .players
            .iter()
            .filter_map(|player| proximity_touch_event(frame, player, ball_position))
            .filter(|event| {
                event.closest_approach_distance.unwrap_or(f32::INFINITY) <= max_collision_distance
            })
            .collect::<Vec<_>>();

        candidates.sort_by(|left, right| touch_distance(left).total_cmp(&touch_distance(right)));
        candidates
    }

    pub(super) fn candidate_touch_event(
        &self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
    ) -> Option<TouchEvent> {
        const TOUCH_COLLISION_DISTANCE_THRESHOLD: f32 = 300.0;

        self.proximity_touch_candidates(frame, ball, players, TOUCH_COLLISION_DISTANCE_THRESHOLD)
            .into_iter()
            .next()
    }

    pub(super) fn update_recent_touch_candidates(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
    ) {
        const PROXIMITY_CANDIDATE_DISTANCE_THRESHOLD: f32 = 220.0;

        for candidate in self.proximity_touch_candidates(
            frame,
            ball,
            players,
            PROXIMITY_CANDIDATE_DISTANCE_THRESHOLD,
        ) {
            if let Some(player_id) = candidate.player.clone() {
                self.recent_touch_candidates.insert(player_id, candidate);
            }
        }
    }
}

fn proximity_touch_event(
    frame: &FrameInfo,
    player: &PlayerSample,
    ball_position: glam::Vec3,
) -> Option<TouchEvent> {
    Some(TouchEvent {
        time: frame.time,
        frame: frame.frame_number,
        team_is_team_0: player.is_team_0,
        player: Some(player.player_id.clone()),
        closest_approach_distance: Some(collision_distance(player, ball_position)?),
        dodge_contact: player.dodge_active,
    })
}

pub(crate) fn touch_distance(event: &TouchEvent) -> f32 {
    event.closest_approach_distance.unwrap_or(f32::INFINITY)
}
