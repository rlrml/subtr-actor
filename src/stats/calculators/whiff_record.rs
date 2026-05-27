use super::*;

impl WhiffCalculator {
    pub(super) fn emit_candidate(
        &mut self,
        candidate: ActiveWhiffCandidate,
        frame: &FrameInfo,
        kind: WhiffEventKind,
    ) {
        let (time, frame_number) = match kind {
            WhiffEventKind::Whiff => (candidate.closest_time, candidate.closest_frame),
            WhiffEventKind::BeatenToBall => (frame.time, frame.frame_number),
        };
        let event = WhiffEvent {
            kind,
            time,
            frame: frame_number,
            resolved_time: frame.time,
            resolved_frame: frame.frame_number,
            player: candidate.player.clone(),
            is_team_0: candidate.is_team_0,
            closest_approach_distance: candidate.closest_approach_distance,
            forward_alignment: candidate.forward_alignment,
            approach_speed: candidate.approach_speed,
            dodge_active: candidate.dodge_active,
            aerial: candidate.aerial,
        };

        let stats = self
            .player_stats
            .entry(candidate.player.clone())
            .or_default();
        match event.kind {
            WhiffEventKind::Whiff => {
                stats.record_whiff(&event);
                stats.is_last_whiff = true;
                stats.time_since_last_whiff = Some((frame.time - event.time).max(0.0));
                stats.frames_since_last_whiff =
                    Some(frame.frame_number.saturating_sub(event.frame));
                self.current_last_whiff_player = Some(candidate.player);
            }
            WhiffEventKind::BeatenToBall => {
                stats.beaten_to_ball_count += 1;
            }
        }
        self.events.push(event);
    }
}
