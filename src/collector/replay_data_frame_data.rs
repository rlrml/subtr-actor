use super::*;

/// Contains all frame-by-frame data for a Rocket League replay.
///
/// This structure organizes ball data, player data, and metadata for each
/// frame of the replay, providing a complete picture of the game state
/// throughout the match.
///
/// # Fields
///
/// * `ball_data` - All ball state information across all frames
/// * `players` - Player data for each player, indexed by [`PlayerId`]
/// * `metadata_frames` - Game metadata for each frame including timing information
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FrameData {
    /// All ball state information across all frames
    pub ball_data: BallData,
    /// Player data for each player, indexed by PlayerId
    #[ts(as = "Vec<(crate::ts_bindings::RemoteIdTs, PlayerData)>")]
    pub players: Vec<(PlayerId, PlayerData)>,
    /// Game metadata for each frame including timing information
    pub metadata_frames: Vec<MetadataFrame>,
}

impl FrameData {
    /// Creates a new empty [`FrameData`] instance.
    ///
    /// # Returns
    ///
    /// Returns a new [`FrameData`] with empty ball data, player data, and metadata frames.
    pub(super) fn new() -> Self {
        FrameData {
            ball_data: BallData::new(),
            players: Vec::new(),
            metadata_frames: Vec::new(),
        }
    }

    /// Returns the total number of frames in this frame data.
    ///
    /// # Returns
    ///
    /// Returns the number of metadata frames, which represents the total frame count.
    pub fn frame_count(&self) -> usize {
        self.metadata_frames.len()
    }

    /// Returns the duration of the replay in seconds.
    ///
    /// # Returns
    ///
    /// Returns the time of the last frame, or 0.0 if no frames exist.
    pub fn duration(&self) -> f32 {
        self.metadata_frames.last().map(|f| f.time).unwrap_or(0.0)
    }

    /// Adds a complete frame of data to the frame data structure.
    ///
    /// This method adds metadata, ball data, and player data for a single frame
    /// to their respective collections, maintaining frame synchronization across
    /// all data types.
    ///
    /// # Arguments
    ///
    /// * `frame_metadata` - The metadata for this frame (time, game state, etc.)
    /// * `ball_frame` - The ball state for this frame
    /// * `player_frames` - Player state data for all players in this frame
    ///
    /// # Returns
    ///
    /// Returns a [`SubtrActorResult`] indicating success or failure of the operation.
    ///
    /// # Errors
    ///
    /// May return a [`SubtrActorError`] if frame data cannot be processed correctly.
    pub(super) fn add_frame(
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
                .or_insert_with(PlayerData::new)
                .add_frame(frame_index, frame)
        }
        Ok(())
    }
}
