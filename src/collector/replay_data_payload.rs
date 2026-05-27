use super::*;

/// Complete replay data structure containing all extracted information from a Rocket League replay.
///
/// This is the top-level structure that contains all processed replay data including
/// frame-by-frame information, replay metadata, and special events like demolitions.
///
/// # Fields
///
/// * `frame_data` - All frame-by-frame data including ball, player, and metadata information
/// * `meta` - Replay metadata including player information, game settings, and statistics
/// * `demolish_infos` - Information about all demolition events that occurred during the replay
/// * `boost_pad_events` - Exact boost pad pickup/availability events detected while processing
/// * `boost_pads` - Resolved standard boost pad layout annotated with replay pad ids when known
/// * `touch_events` - Exact team touch events plus attributed player when available
/// * `dodge_refreshed_events` - Exact counter-derived dodge refresh events from the replay
/// * `player_stat_events` - Exact shot/save/assist counter increment events
/// * `goal_events` - Exact goal explosion events with scorer and cumulative score when available
///
/// # Example
///
/// ```rust
/// use subtr_actor::collector::replay_data::ReplayDataCollector;
/// use boxcars::ParserBuilder;
///
/// let data = std::fs::read("assets/replay-format-2025-06-10-v868-32-net10-replicated-boost.replay").unwrap();
/// let replay = ParserBuilder::new(&data).parse().unwrap();
/// let collector = ReplayDataCollector::new();
/// let replay_data = collector.get_replay_data(&replay).unwrap();
///
/// // Access replay metadata
/// println!("Team 0 players: {}", replay_data.meta.team_zero.len());
///
/// // Access frame data
/// println!("Total frames: {}", replay_data.frame_data.metadata_frames.len());
///
/// // Access demolition events
/// println!("Total demolitions: {}", replay_data.demolish_infos.len());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayData {
    /// All frame-by-frame data including ball, player, and metadata information
    pub frame_data: FrameData,
    /// Replay metadata including player information, game settings, and statistics
    pub meta: ReplayMeta,
    /// Information about all demolition events that occurred during the replay
    pub demolish_infos: Vec<DemolishInfo>,
    /// Exact boost pad pickup and availability events observed during the replay
    pub boost_pad_events: Vec<BoostPadEvent>,
    /// Resolved standard boost pad layout annotated with replay pad ids when known
    pub boost_pads: Vec<ResolvedBoostPad>,
    /// Exact touch events observed during the replay
    pub touch_events: Vec<TouchEvent>,
    /// Exact dodge refresh events observed via the replay's refreshed-dodge counter
    pub dodge_refreshed_events: Vec<DodgeRefreshedEvent>,
    /// Exact player stat counter increments observed during the replay
    pub player_stat_events: Vec<PlayerStatEvent>,
    /// Exact goal events observed during the replay
    pub goal_events: Vec<GoalEvent>,
}

impl ReplayData {
    /// Serializes the replay data to a JSON string.
    ///
    /// # Returns
    ///
    /// Returns a [`Result`] containing either the JSON string representation
    /// of the replay data or a [`serde_json::Error`] if serialization fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use subtr_actor::collector::replay_data::ReplayDataCollector;
    /// use boxcars::ParserBuilder;
    ///
    /// let data = std::fs::read("assets/replay-format-2025-06-10-v868-32-net10-replicated-boost.replay").unwrap();
    /// let replay = ParserBuilder::new(&data).parse().unwrap();
    /// let collector = ReplayDataCollector::new();
    /// let replay_data = collector.get_replay_data(&replay).unwrap();
    ///
    /// let json_string = replay_data.as_json().unwrap();
    /// println!("Replay as JSON: {}", json_string);
    /// ```
    pub fn as_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serializes the replay data to a pretty-printed JSON string.
    ///
    /// # Returns
    ///
    /// Returns a [`Result`] containing either the pretty-printed JSON string
    /// representation of the replay data or a [`serde_json::Error`] if serialization fails.
    pub fn as_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
