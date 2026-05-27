use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ActiveSpeedFlipCandidate {
    pub(super) is_team_0: bool,
    pub(super) is_kickoff: bool,
    pub(super) kickoff_start_time: Option<f32>,
    pub(super) start_time: f32,
    pub(super) start_frame: usize,
    pub(super) start_position: [f32; 3],
    pub(super) end_position: [f32; 3],
    pub(super) start_velocity_xy: glam::Vec2,
    pub(super) start_forward_xy: glam::Vec2,
    pub(super) start_speed: f32,
    pub(super) max_speed: f32,
    pub(super) best_alignment: f32,
    pub(super) best_boost_alignment: f32,
    pub(super) boost_alignment_sample_count: u32,
    pub(super) best_dodge_forward_delta: f32,
    pub(super) best_dodge_delta_alignment: f32,
    pub(super) dodge_acceleration_sample_count: u32,
    pub(super) best_diagonal_score: f32,
    pub(super) min_forward_z: f32,
    pub(super) latest_forward_z: f32,
    pub(super) latest_time: f32,
    pub(super) latest_frame: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpeedFlipCalculator {
    pub(super) player_stats: HashMap<PlayerId, SpeedFlipStats>,
    pub(super) events: Vec<SpeedFlipEvent>,
    pub(super) active_candidates: HashMap<PlayerId, ActiveSpeedFlipCandidate>,
    pub(super) previous_dodge_active: HashMap<PlayerId, bool>,
    pub(super) kickoff_approach_active_last_frame: bool,
    pub(super) current_kickoff_start_time: Option<f32>,
    pub(super) current_last_speed_flip_player: Option<PlayerId>,
}
