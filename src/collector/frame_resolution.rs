const FRAME_RESOLUTION_EPSILON_SECONDS: f32 = 1e-4;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum StatsFrameResolution {
    #[default]
    EveryFrame,
    TimeStep {
        seconds: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FinalStatsFrameAction {
    Append { dt: f32 },
    ReplaceLast { dt: f32 },
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StatsFramePersistenceController {
    resolution: StatsFrameResolution,
    next_emit_time: Option<f32>,
    last_emitted_frame_number: Option<usize>,
    last_emitted_time: Option<f32>,
    last_emitted_dt: f32,
}

impl StatsFramePersistenceController {
    pub(crate) fn new(resolution: StatsFrameResolution) -> Self {
        Self {
            resolution,
            next_emit_time: None,
            last_emitted_frame_number: None,
            last_emitted_time: None,
            last_emitted_dt: 0.0,
        }
    }

    pub(crate) fn on_frame(&mut self, frame_number: usize, current_time: f32) -> Option<f32> {
        if self.last_emitted_time.is_none() {
            return Some(self.record_emit(frame_number, current_time));
        }

        match self.resolution {
            StatsFrameResolution::EveryFrame => Some(self.record_emit(frame_number, current_time)),
            StatsFrameResolution::TimeStep { seconds } => {
                if !seconds.is_finite() || seconds <= 0.0 {
                    return Some(self.record_emit(frame_number, current_time));
                }

                let next_emit_time = self.next_emit_time.unwrap_or(current_time + seconds);
                if current_time + FRAME_RESOLUTION_EPSILON_SECONDS < next_emit_time {
                    self.next_emit_time = Some(next_emit_time);
                    return None;
                }

                let dt = self.record_emit(frame_number, current_time);
                let mut advanced_next_emit_time = next_emit_time;
                while advanced_next_emit_time <= current_time + FRAME_RESOLUTION_EPSILON_SECONDS {
                    advanced_next_emit_time += seconds;
                }
                self.next_emit_time = Some(advanced_next_emit_time);
                Some(dt)
            }
        }
    }

    pub(crate) fn final_frame_action(
        &self,
        frame_number: usize,
        current_time: f32,
    ) -> Option<FinalStatsFrameAction> {
        let Some(last_emitted_time) = self.last_emitted_time else {
            return Some(FinalStatsFrameAction::Append { dt: 0.0 });
        };

        if self.last_emitted_frame_number == Some(frame_number) {
            return Some(FinalStatsFrameAction::ReplaceLast {
                dt: self.last_emitted_dt,
            });
        }

        Some(FinalStatsFrameAction::Append {
            dt: (current_time - last_emitted_time).max(0.0),
        })
    }

    fn record_emit(&mut self, frame_number: usize, current_time: f32) -> f32 {
        let dt = self
            .last_emitted_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        self.last_emitted_frame_number = Some(frame_number);
        self.last_emitted_time = Some(current_time);
        self.last_emitted_dt = dt;
        self.next_emit_time = match self.resolution {
            StatsFrameResolution::EveryFrame => None,
            StatsFrameResolution::TimeStep { seconds } if seconds.is_finite() && seconds > 0.0 => {
                Some(current_time + seconds)
            }
            StatsFrameResolution::TimeStep { .. } => None,
        };
        dt
    }
}

#[cfg(test)]
mod tests {
    use super::{FinalStatsFrameAction, StatsFramePersistenceController, StatsFrameResolution};

    #[test]
    fn every_frame_resolution_emits_every_frame() {
        let mut controller = StatsFramePersistenceController::new(StatsFrameResolution::EveryFrame);

        assert_eq!(controller.on_frame(10, 0.0), Some(0.0));
        assert_eq!(controller.on_frame(11, 0.1), Some(0.1));
        assert_eq!(controller.on_frame(12, 0.25), Some(0.15));
        assert_eq!(
            controller.final_frame_action(12, 0.25),
            Some(FinalStatsFrameAction::ReplaceLast { dt: 0.15 })
        );
    }

    #[test]
    fn timestep_resolution_emits_crossings_and_appends_final_frame() {
        let mut controller =
            StatsFramePersistenceController::new(StatsFrameResolution::TimeStep { seconds: 0.5 });

        assert_eq!(controller.on_frame(0, 0.0), Some(0.0));
        assert_eq!(controller.on_frame(1, 0.2), None);
        assert_eq!(controller.on_frame(2, 0.49), None);
        assert_eq!(controller.on_frame(3, 0.5), Some(0.5));
        assert_eq!(controller.on_frame(4, 0.74), None);
        match controller.final_frame_action(4, 0.74) {
            Some(FinalStatsFrameAction::Append { dt }) => {
                assert!(
                    (dt - 0.24).abs() < 1e-6,
                    "expected dt close to 0.24, got {dt}"
                );
            }
            action => panic!("expected append action, got {action:?}"),
        }
    }
}
