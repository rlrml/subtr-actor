use boxcars;
use std::collections::HashMap;

use crate::{processor::*, Collector};

#[derive(Debug, Clone, PartialEq)]
pub enum BallFrame {
    Empty,
    Data { rigid_body: boxcars::RigidBody },
}

impl BallFrame {
    fn new_from_processor(processor: &ReplayProcessor) -> Self {
        if processor.get_ignore_ball_syncing().unwrap_or(false) {
            Self::Empty
        } else if let Ok(rigid_body) = processor.get_ball_rigid_body() {
            Self::new_from_rigid_body(rigid_body)
        } else {
            Self::Empty
        }
    }

    fn new_from_rigid_body(rigid_body: &boxcars::RigidBody) -> Self {
        if rigid_body.sleeping {
            Self::Empty
        } else {
            Self::Data {
                rigid_body: rigid_body.clone(),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    ) -> Result<Self, String> {
        let rigid_body = processor.get_player_rigid_body(player_id)?;

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
            *boost_amount,
            boost_active,
            jump_active,
            double_jump_active,
            dodge_active,
        ))
    }

    fn from_data(
        rigid_body: &boxcars::RigidBody,
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
                rigid_body: rigid_body.clone(),
                boost_amount,
                boost_active,
                jump_active,
                double_jump_active,
                dodge_active,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerData {
    frames: Vec<PlayerFrame>,
}

impl PlayerData {
    fn new() -> Self {
        Self { frames: Vec::new() }
    }

    fn add_frame(&mut self, frame_number: usize, frame: PlayerFrame) {
        let empty_frames_to_add = frame_number - self.frames.len();
        if empty_frames_to_add > 0 {
            for _ in 0..empty_frames_to_add {
                self.frames.push(PlayerFrame::Empty)
            }
        }
        self.frames.push(frame)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BallData {
    frames: Vec<BallFrame>,
}

impl BallData {
    fn add_frame(&mut self, frame_number: usize, frame: BallFrame) {
        let empty_frames_to_add = frame_number - self.frames.len();
        if empty_frames_to_add > 0 {
            for _ in 0..empty_frames_to_add {
                self.frames.push(BallFrame::Empty)
            }
        }
        self.frames.push(frame)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataFrame {
    pub time: f32,
    pub seconds_remaining: i32,
}

impl MetadataFrame {
    fn new_from_processor(processor: &ReplayProcessor, time: f32) -> Result<Self, String> {
        Ok(Self::new(time, *processor.get_seconds_remaining()?))
    }

    fn new(time: f32, seconds_remaining: i32) -> Self {
        MetadataFrame {
            time,
            seconds_remaining,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayData {
    ball_data: BallData,
    players: HashMap<PlayerId, PlayerData>,
    frame_metadata: Vec<MetadataFrame>,
}

impl ReplayData {
    fn new() -> Self {
        ReplayData {
            ball_data: BallData { frames: Vec::new() },
            players: HashMap::new(),
            frame_metadata: Vec::new(),
        }
    }

    fn add_frame(
        &mut self,
        frame_metadata: MetadataFrame,
        ball_frame: BallFrame,
        player_frames: Vec<(PlayerId, PlayerFrame)>,
    ) -> Result<(), String> {
        self.frame_metadata.push(frame_metadata);
        let frame_number = self.frame_metadata.len();
        self.ball_data.add_frame(frame_number, ball_frame);
        for (player_id, frame) in player_frames {
            self.players
                .entry(player_id)
                .or_insert_with(|| PlayerData::new())
                .add_frame(frame_number, frame)
        }
        Ok(())
    }
}

pub struct ReplayDataCollector {
    replay_data: ReplayData,
}

impl ReplayDataCollector {
    pub fn new() -> Self {
        ReplayDataCollector {
            replay_data: ReplayData::new(),
        }
    }

    pub fn process_replay(replay: &boxcars::Replay) -> Result<ReplayData, String> {
        Self::new().build_processer_and_process(replay)
    }

    pub fn build_processer_and_process(
        mut self,
        replay: &boxcars::Replay,
    ) -> Result<ReplayData, String> {
        ReplayProcessor::new(replay).process(&mut self)?;
        Ok(self.replay_data)
    }

    fn get_player_frames(
        &self,
        processor: &ReplayProcessor,
    ) -> Result<Vec<(PlayerId, PlayerFrame)>, String> {
        Ok(processor
            .iter_player_ids_in_order()
            .map(|player_id| {
                (
                    player_id.clone(),
                    PlayerFrame::new_from_processor(processor, player_id)
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
        frame: &boxcars::Frame,
        _frame_number: usize,
    ) -> Result<(), String> {
        let metadata_frame = MetadataFrame::new_from_processor(processor, frame.time)?;
        let ball_frame = BallFrame::new_from_processor(processor);
        let player_frames = self.get_player_frames(processor)?;
        self.replay_data
            .add_frame(metadata_frame, ball_frame, player_frames)?;
        Ok(())
    }
}
