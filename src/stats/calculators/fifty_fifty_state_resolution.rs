use super::*;

impl FiftyFiftyStateCalculator {
    pub(super) fn reset(&mut self) {
        self.active_event = None;
    }

    pub(super) fn maybe_resolve_active_event(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        possession_state: &PossessionState,
    ) -> Option<FiftyFiftyEvent> {
        let active = self.active_event.as_ref()?;
        let age = (frame.time - active.last_touch_time).max(0.0);
        if age < FIFTY_FIFTY_RESOLUTION_DELAY_SECONDS {
            return None;
        }

        let winning_team_is_team_0 = FiftyFiftyCalculator::winning_team_from_ball(active, ball);
        let possession_team_is_team_0 = possession_state.current_team_is_team_0;
        if !fifty_fifty_should_resolve(age, winning_team_is_team_0, possession_team_is_team_0) {
            return None;
        }

        let active = self.active_event.take()?;
        let event = FiftyFiftyEvent {
            start_time: active.start_time,
            start_frame: active.start_frame,
            resolve_time: frame.time,
            resolve_frame: frame.frame_number,
            is_kickoff: active.is_kickoff,
            team_zero_player: active.team_zero_player,
            team_one_player: active.team_one_player,
            team_zero_touch_time: active.team_zero_touch_time,
            team_zero_touch_frame: active.team_zero_touch_frame,
            team_zero_dodge_contact: active.team_zero_dodge_contact,
            team_one_touch_time: active.team_one_touch_time,
            team_one_touch_frame: active.team_one_touch_frame,
            team_one_dodge_contact: active.team_one_dodge_contact,
            team_zero_position: active.team_zero_position,
            team_one_position: active.team_one_position,
            midpoint: active.midpoint,
            plane_normal: active.plane_normal,
            winning_team_is_team_0,
            possession_team_is_team_0,
        };
        self.last_resolved_event = Some(event.clone());
        Some(event)
    }
}

fn fifty_fifty_should_resolve(
    age: f32,
    winning_team_is_team_0: Option<bool>,
    possession_team_is_team_0: Option<bool>,
) -> bool {
    winning_team_is_team_0.is_some()
        || possession_team_is_team_0.is_some()
        || age >= FIFTY_FIFTY_MAX_DURATION_SECONDS
}
