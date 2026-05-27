use super::*;

impl CeilingShotCalculator {
    pub(super) fn candidate_event(
        &self,
        ball: &BallFrameState,
        player: &PlayerSample,
        touch_event: &TouchEvent,
        recent_contact: RecentCeilingContact,
        ball_speed_change: f32,
    ) -> Option<CeilingShotEvent> {
        let metrics = CeilingShotCandidateMetrics::new(
            ball,
            player,
            touch_event,
            recent_contact,
            ball_speed_change,
        )?;

        Some(CeilingShotEvent {
            time: touch_event.time,
            frame: touch_event.frame,
            player: player.player_id.clone(),
            is_team_0: player.is_team_0,
            ceiling_contact_time: recent_contact.time,
            ceiling_contact_frame: recent_contact.frame,
            time_since_ceiling_contact: metrics.time_since_ceiling_contact,
            ceiling_contact_position: recent_contact.position,
            touch_position: metrics.touch_position,
            local_ball_position: metrics.local_ball_position,
            separation_from_ceiling: metrics.separation_from_ceiling,
            roof_alignment: recent_contact.roof_alignment,
            forward_alignment: metrics.forward_alignment,
            forward_approach_speed: metrics.forward_approach_speed,
            ball_speed_change: metrics.ball_speed_change,
            confidence: metrics.confidence,
        })
    }
}
