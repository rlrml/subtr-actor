use super::*;

impl MustyFlickCalculator {
    pub(super) fn musty_candidate(
        &self,
        ball: &BallFrameState,
        player: &PlayerSample,
        touch_event: &TouchEvent,
        dodge_start: RecentDodgeStart,
        ball_speed_change: f32,
    ) -> Option<MustyFlickEvent> {
        let metrics = MustyFlickCandidateMetrics::new(
            ball,
            player,
            touch_event,
            dodge_start,
            ball_speed_change,
        )?;

        Some(MustyFlickEvent {
            time: touch_event.time,
            frame: touch_event.frame,
            sample_time: touch_event.time,
            sample_frame: touch_event.frame,
            player: player.player_id.clone(),
            is_team_0: player.is_team_0,
            aerial: metrics.aerial,
            dodge_time: dodge_start.time,
            dodge_frame: dodge_start.frame,
            time_since_dodge: metrics.time_since_dodge,
            confidence: metrics.confidence,
            local_ball_position: metrics.local_ball_position,
            rear_alignment: metrics.rear_alignment,
            top_alignment: metrics.top_alignment,
            forward_approach_speed: metrics.forward_approach_speed,
            pitch_rate: metrics.pitch_rate,
            ball_speed_change: metrics.ball_speed_change,
        })
    }
}
