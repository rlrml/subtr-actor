use boxcars;
use serde::Serialize;

use crate::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum BallFrame {
    Empty,
    Data { rigid_body: boxcars::RigidBody },
}

impl BallFrame {
    fn new_from_processor(processor: &ReplayProcessor, current_time: f32) -> Self {
        if processor.get_ignore_ball_syncing().unwrap_or(false) {
            Self::Empty
        } else if let Ok(rigid_body) = processor.get_interpolated_ball_rigid_body(current_time, 0.0)
        {
            Self::new_from_rigid_body(rigid_body)
        } else {
            Self::Empty
        }
    }

    fn new_from_rigid_body(rigid_body: boxcars::RigidBody) -> Self {
        if rigid_body.sleeping {
            Self::Empty
        } else {
            Self::Data {
                rigid_body: rigid_body.clone(),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum PlayerFrame {
    Empty,
    Data {
        rigid_body: boxcars::RigidBody,
        boost_amount: f32,
        boost_active: bool,
        jump_active: bool,
        double_jump_active: bool,
        dodge_active: bool,
    },
}

impl PlayerFrame {
    fn new_from_processor(
        processor: &ReplayProcessor,
        player_id: &PlayerId,
        current_time: f32,
    ) -> SubtrActorResult<Self> {
        let rigid_body =
            processor.get_interpolated_player_rigid_body(player_id, current_time, 0.0)?;

        if rigid_body.sleeping {
            return Ok(PlayerFrame::Empty);
        }

        let boost_amount = processor.get_player_boost_level(player_id)?;
        let boost_active = processor.get_boost_active(player_id).unwrap_or(0) % 2 == 1;
        let jump_active = processor.get_jump_active(player_id).unwrap_or(0) % 2 == 1;
        let double_jump_active = processor.get_double_jump_active(player_id).unwrap_or(0) % 2 == 1;
        let dodge_active = processor.get_dodge_active(player_id).unwrap_or(0) % 2 == 1;

        Ok(Self::from_data(
            rigid_body,
            boost_amount,
            boost_active,
            jump_active,
            double_jump_active,
            dodge_active,
        ))
    }

    fn from_data(
        rigid_body: boxcars::RigidBody,
        boost_amount: f32,
        boost_active: bool,
        jump_active: bool,
        double_jump_active: bool,
        dodge_active: bool,
    ) -> Self {
        if rigid_body.sleeping {
            Self::Empty
        } else {
            Self::Data {
                rigid_body,
                boost_amount,
                boost_active,
                jump_active,
                double_jump_active,
                dodge_active,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayerData {
    frames: Vec<PlayerFrame>,
}

impl PlayerData {
    fn new() -> Self {
        Self { frames: Vec::new() }
    }

    fn add_frame(&mut self, frame_index: usize, frame: PlayerFrame) {
        let empty_frames_to_add = frame_index - self.frames.len();
        if empty_frames_to_add > 0 {
            for _ in 0..empty_frames_to_add {
                self.frames.push(PlayerFrame::Empty)
            }
        }
        self.frames.push(frame)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BallData {
    frames: Vec<BallFrame>,
}

impl BallData {
    fn add_frame(&mut self, frame_index: usize, frame: BallFrame) {
        let empty_frames_to_add = frame_index - self.frames.len();
        if empty_frames_to_add > 0 {
            for _ in 0..empty_frames_to_add {
                self.frames.push(BallFrame::Empty)
            }
        }
        self.frames.push(frame)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MetadataFrame {
    pub time: f32,
    pub seconds_remaining: i32,
}

impl MetadataFrame {
    fn new_from_processor(processor: &ReplayProcessor, time: f32) -> SubtrActorResult<Self> {
        Ok(Self::new(time, processor.get_seconds_remaining()?))
    }

    fn new(time: f32, seconds_remaining: i32) -> Self {
        MetadataFrame {
            time,
            seconds_remaining,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FrameData {
    pub ball_data: BallData,
    pub players: Vec<(PlayerId, PlayerData)>,
    pub metadata_frames: Vec<MetadataFrame>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayData {
    pub frame_data: FrameData,
    pub meta: ReplayMeta,
    pub demolish_infos: Vec<DemolishInfo>,
}

impl FrameData {
    fn new() -> Self {
        FrameData {
            ball_data: BallData { frames: Vec::new() },
            players: Vec::new(),
            metadata_frames: Vec::new(),
        }
    }

    fn add_frame(
        &mut self,
        frame_metadata: MetadataFrame,
        ball_frame: BallFrame,
        player_frames: Vec<(PlayerId, PlayerFrame)>,
    ) -> SubtrActorResult<()> {
        let frame_index = self.metadata_frames.len();
        self.metadata_frames.push(frame_metadata);
        self.ball_data.add_frame(frame_index, ball_frame);
        for (player_id, frame) in player_frames {
            self.players
                .get_entry(player_id)
                .or_insert_with(|| PlayerData::new())
                .add_frame(frame_index, frame)
        }
        Ok(())
    }
}

pub struct ReplayDataCollector {
    frame_data: FrameData,
}

impl ReplayDataCollector {
    pub fn new() -> Self {
        ReplayDataCollector {
            frame_data: FrameData::new(),
        }
    }

    pub fn get_frame_data(self) -> FrameData {
        self.frame_data
    }

    pub fn get_replay_data(mut self, replay: &boxcars::Replay) -> SubtrActorResult<ReplayData> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        let meta = processor.get_replay_meta()?;
        Ok(ReplayData {
            meta,
            demolish_infos: processor.demolishes,
            frame_data: self.get_frame_data(),
        })
    }

    fn get_player_frames(
        &self,
        processor: &ReplayProcessor,
        current_time: f32,
    ) -> SubtrActorResult<Vec<(PlayerId, PlayerFrame)>> {
        Ok(processor
            .iter_player_ids_in_order()
            .map(|player_id| {
                (
                    player_id.clone(),
                    PlayerFrame::new_from_processor(processor, player_id, current_time)
                        .unwrap_or_else(|_err| PlayerFrame::Empty),
                )
            })
            .collect())
    }
}

impl Collector for ReplayDataCollector {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        _frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        let metadata_frame = MetadataFrame::new_from_processor(processor, current_time)?;
        let ball_frame = BallFrame::new_from_processor(processor, current_time);
        let player_frames = self.get_player_frames(processor, current_time)?;
        self.frame_data
            .add_frame(metadata_frame, ball_frame, player_frames)?;
        Ok(TimeAdvance::NextFrame)
    }
}
