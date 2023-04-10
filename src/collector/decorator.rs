use crate::*;

// trait CollectorDecorator {
//     fn decorate<H, O>(&mut self, handler: H) -> O
//     where
//         H: FnMut(&ReplayProcessor, &boxcars::Frame, usize) -> Result<(), String>,
//         O: FnMut(&ReplayProcessor, &boxcars::Frame, usize) -> Result<(), String>;
// }

// struct FilterDecorator<G>(G);

// impl<G> CollectorDecorator for FilterDecorator<G>
// where
//     G: FnMut(&ReplayProcessor, &boxcars::Frame, usize) -> bool,
// {
//     fn decorate<H, O>(
//         &mut self,
//         handler: H,
//     ) -> impl FnMut(&ReplayProcessor, &boxcars::Frame, usize) -> Result<(), String>
//     where
//         H: FnMut(&ReplayProcessor, &boxcars::Frame, usize) -> Result<(), String>,
//         O: FnMut(&ReplayProcessor, &boxcars::Frame, usize) -> Result<(), String>,
//     {
//         move |p, f, c| {
//             if self.0(p, f, c) {
//                 handler(p, f, c)
//             } else {
//                 Ok(())
//             }
//         }
//     }
// }

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
    Ok(processor.get_ball_rigid_body().is_ok())
}
