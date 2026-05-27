use super::super::PlayerSample;

#[derive(Debug, Clone, Default)]
pub struct FrameInfo {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct PlayerFrameState {
    pub players: Vec<PlayerSample>,
}
