use super::*;
use std::collections::HashSet;

impl WhiffCalculator {
    pub(super) fn update_active_candidates(
        &mut self,
        frame: &FrameInfo,
        ball_position: glam::Vec3,
        ball_velocity: glam::Vec3,
        players: &PlayerFrameState,
    ) {
        let mut visible_players = HashSet::new();

        for player in &players.players {
            let player_id = player.player_id.clone();
            visible_players.insert(player_id.clone());
            let distance = Self::hitbox_distance(ball_position, player);

            if let (Some(candidate), Some(distance)) =
                (self.active_candidates.get_mut(&player_id), distance)
            {
                Self::update_candidate_closest(
                    candidate,
                    frame,
                    ball_position,
                    ball_velocity,
                    player,
                    distance,
                );
                if Self::should_emit_candidate(frame, candidate, distance) {
                    if let Some(candidate) = self.active_candidates.remove(&player_id) {
                        self.emit_candidate(candidate, frame, WhiffEventKind::Whiff);
                    }
                }
                continue;
            }

            if let Some(candidate) =
                Self::whiff_candidate(frame, ball_position, ball_velocity, player)
            {
                self.active_candidates.insert(player_id, candidate);
            }
        }

        let missing_players = self
            .active_candidates
            .keys()
            .filter(|player_id| !visible_players.contains(*player_id))
            .cloned()
            .collect::<Vec<_>>();
        for player_id in missing_players {
            self.active_candidates.remove(&player_id);
        }
    }

    fn update_candidate_closest(
        candidate: &mut ActiveWhiffCandidate,
        frame: &FrameInfo,
        ball_position: glam::Vec3,
        ball_velocity: glam::Vec3,
        player: &PlayerSample,
        distance: f32,
    ) {
        if distance >= candidate.closest_approach_distance {
            return;
        }
        candidate.closest_approach_distance = distance;
        candidate.closest_time = frame.time;
        candidate.closest_frame = frame.frame_number;
        if let Some(updated) = Self::whiff_candidate(frame, ball_position, ball_velocity, player) {
            candidate.forward_alignment = updated.forward_alignment;
            candidate.approach_speed = updated.approach_speed;
            candidate.dodge_active |= updated.dodge_active;
            candidate.aerial |= updated.aerial;
        }
    }

    fn should_emit_candidate(
        frame: &FrameInfo,
        candidate: &ActiveWhiffCandidate,
        distance: f32,
    ) -> bool {
        distance > WHIFF_EXIT_DISTANCE
            || frame.time - candidate.start_time > WHIFF_MAX_CANDIDATE_SECONDS
    }
}
