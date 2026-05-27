use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FiftyFiftyState {
    pub active_event: Option<ActiveFiftyFifty>,
    pub resolved_events: Vec<FiftyFiftyEvent>,
    pub last_resolved_event: Option<FiftyFiftyEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveFiftyFifty {
    pub start_time: f32,
    pub start_frame: usize,
    pub last_touch_time: f32,
    pub last_touch_frame: usize,
    pub is_kickoff: bool,
    pub team_zero_player: Option<PlayerId>,
    pub team_one_player: Option<PlayerId>,
    pub team_zero_touch_time: Option<f32>,
    pub team_zero_touch_frame: Option<usize>,
    pub team_zero_dodge_contact: bool,
    pub team_one_touch_time: Option<f32>,
    pub team_one_touch_frame: Option<usize>,
    pub team_one_dodge_contact: bool,
    pub team_zero_position: [f32; 3],
    pub team_one_position: [f32; 3],
    pub midpoint: [f32; 3],
    pub plane_normal: [f32; 3],
}

impl ActiveFiftyFifty {
    pub fn midpoint_vec(&self) -> glam::Vec3 {
        glam::Vec3::from_array(self.midpoint)
    }

    pub fn plane_normal_vec(&self) -> glam::Vec3 {
        glam::Vec3::from_array(self.plane_normal)
    }

    pub fn contains_team_touch(&self, touch_events: &[TouchEvent]) -> bool {
        touch_events.iter().any(|touch| {
            (touch.team_is_team_0 && self.team_zero_player.is_some())
                || (!touch.team_is_team_0 && self.team_one_player.is_some())
        })
    }
}
