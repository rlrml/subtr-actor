use super::*;

impl TerritorialPressureCalculator {
    pub fn finish(&mut self) -> SubtrActorResult<()> {
        if let Some(frame) = self.last_frame {
            self.end_active_session_parts(
                frame.frame_number,
                frame.time,
                TerritorialPressureEndReason::ReplayEnd,
            );
        }
        Ok(())
    }

    pub(super) fn end_active_session(
        &mut self,
        frame: &FrameInfo,
        end_reason: TerritorialPressureEndReason,
    ) {
        self.end_active_session_parts(frame.frame_number, frame.time, end_reason);
    }

    pub(super) fn end_active_session_parts(
        &mut self,
        end_frame: usize,
        end_time: f32,
        end_reason: TerritorialPressureEndReason,
    ) {
        let Some(active) = self.active.take() else {
            return;
        };
        self.events.push(TerritorialPressureEvent {
            start_time: active.start_time,
            start_frame: active.start_frame,
            end_time,
            end_frame,
            team_is_team_0: active.team_is_team_0,
            duration: active.duration,
            offensive_half_time: active.offensive_half_time,
            offensive_third_time: active.offensive_third_time,
            end_reason,
        });
    }
}
