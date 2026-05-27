use super::*;

impl PossessionCalculator {
    fn apply_possession_time(
        stats: &mut PossessionStats,
        state: PossessionStateLabel,
        field_third: Option<FieldThirdLabel>,
        dt: f32,
    ) {
        match state {
            PossessionStateLabel::TeamZero => stats.team_zero_time += dt,
            PossessionStateLabel::TeamOne => stats.team_one_time += dt,
            PossessionStateLabel::Neutral => stats.neutral_time += dt,
        }
        if let Some(field_third) = field_third {
            stats
                .labeled_time
                .add([state.as_label(), field_third.as_label()], dt);
        } else {
            stats.labeled_time.add([state.as_label()], dt);
        }
    }

    fn emit_event_if_changed(
        &mut self,
        frame: &FrameInfo,
        active: bool,
        possession_state: PossessionStateLabel,
        field_third: Option<FieldThirdLabel>,
    ) {
        let event_state = PossessionEventState {
            active,
            possession_state,
            field_third,
        };
        if self.last_emitted_event_state == Some(event_state) {
            return;
        }
        self.events.push(PossessionEvent {
            time: frame.time,
            frame: frame.frame_number,
            active,
            possession_state: possession_state.as_label_value().to_owned(),
            field_third: field_third.map(|label| label.as_label_value().to_owned()),
        });
        self.last_emitted_event_state = Some(event_state);
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        possession_state: &PossessionState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        if !live_play_state.is_live_play {
            self.emit_event_if_changed(frame, false, PossessionStateLabel::Neutral, None);
            return Ok(());
        }
        self.stats.tracked_time += frame.dt;
        let field_third = ball.sample().map(FieldThirdLabel::from_ball);
        let state = possession_state_label(possession_state.active_team_before_sample);
        Self::apply_possession_time(&mut self.stats, state, field_third, frame.dt);
        self.emit_event_if_changed(frame, true, state, field_third);
        Ok(())
    }
}

fn possession_state_label(team_is_team_0: Option<bool>) -> PossessionStateLabel {
    match team_is_team_0 {
        Some(true) => PossessionStateLabel::TeamZero,
        Some(false) => PossessionStateLabel::TeamOne,
        None => PossessionStateLabel::Neutral,
    }
}
