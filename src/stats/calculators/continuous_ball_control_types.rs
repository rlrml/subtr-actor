use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContinuousBallControlState {
    pub completed_sequences: Vec<CompletedBallControlSequence<BallCarryKind>>,
}

#[derive(Debug, Clone, Copy)]
pub struct ContinuousBallControlSample<K> {
    pub kind: K,
    pub player_position: glam::Vec3,
    pub horizontal_gap: f32,
    pub vertical_gap: f32,
    pub speed: f32,
}

#[derive(Debug, Clone)]
pub struct ContinuousBallControlCandidate<K> {
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub touch_count: u32,
    pub air_touch_count: u32,
    pub sample: ContinuousBallControlSample<K>,
}

#[derive(Debug, Clone)]
pub struct ContinuousBallControlPlayerStatus {
    pub player_id: PlayerId,
    pub is_airborne: bool,
}

#[derive(Debug, Clone)]
pub struct ContinuousBallControlTouch {
    pub player_id: PlayerId,
    pub is_airborne: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletedBallControlSequence<K> {
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub kind: K,
    pub start_frame: usize,
    pub end_frame: usize,
    pub start_time: f32,
    pub end_time: f32,
    pub duration: f32,
    pub straight_line_distance: f32,
    pub path_distance: f32,
    pub average_horizontal_gap: f32,
    pub average_vertical_gap: f32,
    pub average_speed: f32,
    pub start_position: glam::Vec3,
    pub end_position: glam::Vec3,
    pub touch_count: u32,
    pub air_touch_count: u32,
}
