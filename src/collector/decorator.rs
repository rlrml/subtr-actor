use crate::*;

#[macro_export]
macro_rules! filter_decorate {
    ($filter:expr, $handler:expr) => {
        |p, f, c| {
            if $filter(p, f, c)? {
                $handler(p, f, c)
            } else {
                Ok(())
            }
        }
    };
}

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
