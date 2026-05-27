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
