use super::*;

impl ReplayDataCollector {
    /// Extracts player frame data for all players at the specified time.
    ///
    /// This method iterates through all players in the replay and extracts their
    /// state information at the given time, returning a vector of player frames
    /// indexed by player ID.
    ///
    /// # Arguments
    ///
    /// * `processor` - The [`ReplayProcessor`] containing the replay data
    /// * `current_time` - The time in seconds at which to extract player states
    ///
    /// # Returns
    ///
    /// Returns a [`SubtrActorResult`] containing a vector of tuples with player IDs
    /// and their corresponding [`PlayerFrame`] data.
    ///
    /// # Errors
    ///
    /// Returns a [`SubtrActorError`] if player frame data cannot be extracted.
    fn get_player_frames(
        &self,
        processor: &dyn ProcessorView,
        current_time: f32,
    ) -> SubtrActorResult<Vec<(PlayerId, PlayerFrame)>> {
        Ok(processor
            .iter_player_ids_in_order()
            .map(|player_id| {
                (
                    player_id.clone(),
                    PlayerFrame::new_from_processor(processor, player_id, current_time)
                        .unwrap_or(PlayerFrame::Empty),
                )
            })
            .collect())
    }
}

impl Collector for ReplayDataCollector {
    /// Processes a single frame of the replay and extracts all relevant data.
    ///
    /// This method is called by the [`ReplayProcessor`] for each frame in the replay.
    /// It extracts metadata, ball state, and player state information and adds them
    /// to the internal frame data structure.
    ///
    /// # Arguments
    ///
    /// * `processor` - The [`ReplayProcessor`] containing the replay data and context
    /// * `_frame` - The current frame data (unused in this implementation)
    /// * `_frame_number` - The current frame number (unused in this implementation)
    /// * `current_time` - The current time in seconds since the start of the replay
    ///
    /// # Returns
    ///
    /// Returns a [`SubtrActorResult`] containing [`TimeAdvance::NextFrame`] to
    /// indicate that processing should continue to the next frame.
    ///
    /// # Errors
    ///
    /// Returns a [`SubtrActorError`] if:
    /// - Metadata frame cannot be created
    /// - Player frame data cannot be extracted
    /// - Frame data cannot be added to the collection
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
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
