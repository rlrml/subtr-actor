#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ActiveHalfFlipCandidate {
    pub(super) is_team_0: bool,
    pub(super) start_time: f32,
    pub(super) start_frame: usize,
    pub(super) latest_time: f32,
    pub(super) latest_frame: usize,
    pub(super) start_position: [f32; 3],
    pub(super) end_position: [f32; 3],
    pub(super) start_speed: f32,
    pub(super) end_speed: f32,
    pub(super) start_forward_xy: glam::Vec2,
    pub(super) start_backward_alignment: f32,
    pub(super) best_reorientation_alignment: f32,
    pub(super) best_forward_reversal: f32,
    pub(super) max_forward_vertical: f32,
}
