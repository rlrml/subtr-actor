use super::*;

impl WallAerialCalculator {
    pub(super) fn completed_setup(active: &ActiveWallControl) -> Option<CompletedWallSetup> {
        let duration = active.last_time - active.start_time;
        (duration >= WALL_AERIAL_MIN_CONTROL_DURATION).then_some(CompletedWallSetup {
            start_time: active.start_time,
            start_frame: active.start_frame,
            duration,
        })
    }
}
