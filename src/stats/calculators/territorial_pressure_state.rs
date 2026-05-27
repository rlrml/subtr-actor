use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TerritorialPressureCalculator {
    pub(super) config: TerritorialPressureCalculatorConfig,
    pub(super) stats: TerritorialPressureStats,
    pub(super) events: Vec<TerritorialPressureEvent>,
    pub(super) candidate: Option<CandidateTerritorialPressureSession>,
    pub(super) active: Option<ActiveTerritorialPressureSession>,
    pub(super) last_frame: Option<TerritorialPressureFrameMarker>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct CandidateTerritorialPressureSession {
    pub(super) team_is_team_0: bool,
    pub(super) start_time: f32,
    pub(super) start_frame: usize,
    pub(super) duration: f32,
    pub(super) offensive_half_time: f32,
    pub(super) offensive_third_time: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ActiveTerritorialPressureSession {
    pub(super) team_is_team_0: bool,
    pub(super) start_time: f32,
    pub(super) start_frame: usize,
    pub(super) duration: f32,
    pub(super) offensive_half_time: f32,
    pub(super) offensive_third_time: f32,
    pub(super) relief_time: f32,
    pub(super) confirmed_relief_time: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct TerritorialPressureFrameMarker {
    pub(super) frame_number: usize,
    pub(super) time: f32,
}

impl From<&FrameInfo> for TerritorialPressureFrameMarker {
    fn from(frame: &FrameInfo) -> Self {
        Self {
            frame_number: frame.frame_number,
            time: frame.time,
        }
    }
}
