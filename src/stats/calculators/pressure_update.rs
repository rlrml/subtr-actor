use super::*;

impl PressureCalculator {
    fn apply_pressure_time(stats: &mut PressureStats, half: PressureHalfLabel, dt: f32) {
        match half {
            PressureHalfLabel::TeamZeroSide => stats.team_zero_side_time += dt,
            PressureHalfLabel::TeamOneSide => stats.team_one_side_time += dt,
            PressureHalfLabel::Neutral => stats.neutral_time += dt,
        }

        stats.labeled_time.add([half.as_label()], dt);
    }

    fn emit_event_if_changed(
        &mut self,
        frame: &FrameInfo,
        active: bool,
        field_half: PressureHalfLabel,
    ) {
        let event_state = PressureEventState { active, field_half };
        if self.last_emitted_event_state == Some(event_state) {
            return;
        }
        self.events.push(PressureEvent {
            time: frame.time,
            frame: frame.frame_number,
            active,
            field_half: field_half.as_label_value().to_owned(),
        });
        self.last_emitted_event_state = Some(event_state);
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        if !live_play_state.is_live_play {
            self.emit_event_if_changed(frame, false, PressureHalfLabel::Neutral);
            return Ok(());
        }
        let Some(ball) = ball.sample() else {
            self.emit_event_if_changed(frame, false, PressureHalfLabel::Neutral);
            return Ok(());
        };

        self.stats.tracked_time += frame.dt;
        let half = self.pressure_half(ball.position().y);
        Self::apply_pressure_time(&mut self.stats, half, frame.dt);
        self.emit_event_if_changed(frame, true, half);
        Ok(())
    }

    fn pressure_half(&self, ball_y: f32) -> PressureHalfLabel {
        if ball_y.abs() <= self.config.neutral_zone_half_width_y {
            PressureHalfLabel::Neutral
        } else if ball_y < 0.0 {
            PressureHalfLabel::TeamZeroSide
        } else {
            PressureHalfLabel::TeamOneSide
        }
    }
}
