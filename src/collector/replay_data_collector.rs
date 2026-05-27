use super::*;

/// A collector that extracts comprehensive frame-by-frame data from Rocket League replays.
///
/// [`ReplayDataCollector`] implements the [`Collector`] trait to process replay frames
/// and extract detailed information about ball movement, player actions, and game state.
/// It builds a complete [`ReplayData`] structure containing all available information
/// from the replay.
///
/// # Usage
///
/// The collector is designed to be used with the [`ReplayProcessor`] to extract
/// comprehensive replay data:
///
/// ```rust
/// use subtr_actor::collector::replay_data::ReplayDataCollector;
/// use boxcars::ParserBuilder;
///
/// let data = std::fs::read("assets/replay-format-2025-06-10-v868-32-net10-replicated-boost.replay").unwrap();
/// let replay = ParserBuilder::new(&data).parse().unwrap();
///
/// let collector = ReplayDataCollector::new();
/// let replay_data = collector.get_replay_data(&replay).unwrap();
///
/// // Process the extracted data
/// for (frame_idx, metadata) in replay_data.frame_data.metadata_frames.iter().enumerate() {
///     println!("Frame {}: Time={:.2}s, Remaining={}s",
///              frame_idx, metadata.time, metadata.seconds_remaining);
/// }
/// ```
///
/// # Fields
///
/// * `frame_data` - Internal storage for frame-by-frame data during collection
pub struct ReplayDataCollector {
    /// Internal storage for frame-by-frame data during collection
    pub(super) frame_data: FrameData,
}

impl Default for ReplayDataCollector {
    /// Creates a default [`ReplayDataCollector`] instance.
    ///
    /// This is equivalent to calling [`ReplayDataCollector::new()`].
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayDataCollector {
    /// Creates a new [`ReplayDataCollector`] instance.
    ///
    /// # Returns
    ///
    /// Returns a new collector ready to process replay frames.
    pub fn new() -> Self {
        ReplayDataCollector {
            frame_data: FrameData::new(),
        }
    }

    /// Consumes the collector and returns the collected frame data.
    ///
    /// # Returns
    ///
    /// Returns the [`FrameData`] containing all processed frame information.
    pub fn get_frame_data(self) -> FrameData {
        self.frame_data
    }
}
