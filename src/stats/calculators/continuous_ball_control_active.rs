use super::*;

#[derive(Debug, Clone)]
pub(crate) struct ActiveBallControlSequence<K> {
    pub(crate) player_id: PlayerId,
    pub(crate) is_team_0: bool,
    pub(crate) kind: K,
    pub(crate) start_frame: usize,
    pub(crate) last_frame: usize,
    pub(crate) start_time: f32,
    pub(crate) last_time: f32,
    pub(crate) start_position: glam::Vec3,
    pub(crate) last_position: glam::Vec3,
    pub(crate) duration: f32,
    pub(crate) path_distance: f32,
    pub(crate) horizontal_gap_integral: f32,
    pub(crate) vertical_gap_integral: f32,
    pub(crate) speed_integral: f32,
    pub(crate) touch_count: u32,
    pub(crate) air_touch_count: u32,
}
