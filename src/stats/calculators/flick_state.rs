use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct FlickControlObservation {
    pub(super) horizontal_gap: f32,
    pub(super) vertical_gap: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ActiveFlickSetup {
    pub(super) is_team_0: bool,
    pub(super) start_time: f32,
    pub(super) start_frame: usize,
    pub(super) last_time: f32,
    pub(super) last_frame: usize,
    pub(super) duration: f32,
    pub(super) horizontal_gap_integral: f32,
    pub(super) vertical_gap_integral: f32,
    pub(super) touch_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct FlickSetupSummary {
    pub(super) is_team_0: bool,
    pub(super) start_time: f32,
    pub(super) start_frame: usize,
    pub(super) last_time: f32,
    pub(super) last_frame: usize,
    pub(super) duration: f32,
    pub(super) average_horizontal_gap: f32,
    pub(super) average_vertical_gap: f32,
    pub(super) touch_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RecentDodgeStart {
    pub(super) time: f32,
    pub(super) frame: usize,
    pub(super) setup: FlickSetupSummary,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FlickCalculator {
    pub(super) player_stats: HashMap<PlayerId, FlickStats>,
    pub(super) events: Vec<FlickEvent>,
    pub(super) active_setups: HashMap<PlayerId, ActiveFlickSetup>,
    pub(super) recent_setups: HashMap<PlayerId, FlickSetupSummary>,
    pub(super) recent_dodge_starts: HashMap<PlayerId, RecentDodgeStart>,
    pub(super) previous_dodge_active: HashMap<PlayerId, bool>,
    pub(super) previous_ball_velocity: Option<glam::Vec3>,
    pub(super) current_last_flick_player: Option<PlayerId>,
}
