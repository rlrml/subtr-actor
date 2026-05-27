use super::*;

/// Contains all frame data for a single player throughout the replay.
///
/// This structure holds a chronological sequence of [`PlayerFrame`] instances
/// representing the player's state at each processed frame of the replay.
///
/// # Fields
///
/// * `frames` - A vector of [`PlayerFrame`] instances in chronological order
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerData {
    /// Vector of player frames in chronological order
    frames: Vec<PlayerFrame>,
}

impl PlayerData {
    /// Creates a new empty [`PlayerData`] instance.
    ///
    /// # Returns
    ///
    /// Returns a new [`PlayerData`] with an empty frames vector.
    pub(super) fn new() -> Self {
        Self { frames: Vec::new() }
    }

    /// Adds a player frame at the specified frame index.
    ///
    /// If the frame index is beyond the current length of the frames vector,
    /// empty frames will be inserted to fill the gap before adding the new frame.
    ///
    /// # Arguments
    ///
    /// * `frame_index` - The index at which to insert the frame
    /// * `frame` - The [`PlayerFrame`] to add
    pub(super) fn add_frame(&mut self, frame_index: usize, frame: PlayerFrame) {
        let empty_frames_to_add = frame_index - self.frames.len();
        if empty_frames_to_add > 0 {
            for _ in 0..empty_frames_to_add {
                self.frames.push(PlayerFrame::Empty)
            }
        }
        self.frames.push(frame)
    }

    /// Returns a reference to the frames vector.
    ///
    /// # Returns
    ///
    /// Returns a reference to the vector of [`PlayerFrame`] instances.
    pub fn frames(&self) -> &Vec<PlayerFrame> {
        &self.frames
    }

    /// Returns the number of frames in this player's data.
    ///
    /// # Returns
    ///
    /// Returns the total number of frames stored for this player.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}
