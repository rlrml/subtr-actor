use super::*;

/// Contains all frame data for the ball throughout the replay.
///
/// This structure holds a chronological sequence of [`BallFrame`] instances
/// representing the ball's state at each processed frame of the replay.
///
/// # Fields
///
/// * `frames` - A vector of [`BallFrame`] instances in chronological order
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BallData {
    /// Vector of ball frames in chronological order
    frames: Vec<BallFrame>,
}

impl BallData {
    /// Creates a new empty [`BallData`] instance.
    ///
    /// # Returns
    ///
    /// Returns a new [`BallData`] with an empty frames vector.
    pub(super) fn new() -> Self {
        Self { frames: Vec::new() }
    }

    /// Adds a ball frame at the specified frame index.
    ///
    /// If the frame index is beyond the current length of the frames vector,
    /// empty frames will be inserted to fill the gap before adding the new frame.
    ///
    /// # Arguments
    ///
    /// * `frame_index` - The index at which to insert the frame
    /// * `frame` - The [`BallFrame`] to add
    pub(super) fn add_frame(&mut self, frame_index: usize, frame: BallFrame) {
        let empty_frames_to_add = frame_index - self.frames.len();
        if empty_frames_to_add > 0 {
            for _ in 0..empty_frames_to_add {
                self.frames.push(BallFrame::Empty)
            }
        }
        self.frames.push(frame)
    }

    /// Returns a reference to the frames vector.
    ///
    /// # Returns
    ///
    /// Returns a reference to the vector of [`BallFrame`] instances.
    pub fn frames(&self) -> &Vec<BallFrame> {
        &self.frames
    }

    /// Returns the number of frames in the ball data.
    ///
    /// # Returns
    ///
    /// Returns the total number of frames stored for the ball.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}
