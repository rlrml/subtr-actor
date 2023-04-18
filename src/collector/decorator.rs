use crate::*;

#[macro_export]
macro_rules! filter_decorate {
    ($filter:expr, $handler:expr) => {
        |p: &ReplayProcessor, f: &boxcars::Frame, n: usize| {
            if $filter(p, f, n)? {
                $handler.process_frame(p, f, n)
            } else {
                Ok(())
            }
        }
    };
}

// write a function that wraps an arbitrary collector

pub fn require_ball_rigid_body_exists(
    processor: &ReplayProcessor,
    _f: &boxcars::Frame,
    _: usize,
) -> Result<bool, String> {
    Ok(processor
        .get_ball_rigid_body()
        .map(|rb| !rb.sleeping)
        .unwrap_or(false))
}
