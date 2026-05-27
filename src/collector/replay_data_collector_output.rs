use super::*;

impl ReplayDataCollector {
    /// Builds replay data from this collector and an already-processed
    /// [`ReplayProcessor`].
    ///
    /// This keeps replay-data collection composable: callers can run
    /// [`ReplayDataCollector`] alongside any other collectors with
    /// [`ReplayProcessor::process_all`] and then decide which enrichments to
    /// merge into the final payload.
    pub fn into_replay_data(self, processor: ReplayProcessor<'_>) -> SubtrActorResult<ReplayData> {
        self.into_replay_data_with_boost_pads(processor, Vec::new())
    }

    pub fn into_replay_data_with_boost_pads(
        self,
        processor: ReplayProcessor<'_>,
        boost_pads: Vec<ResolvedBoostPad>,
    ) -> SubtrActorResult<ReplayData> {
        let meta = processor.get_replay_meta()?;
        Ok(ReplayData {
            meta,
            demolish_infos: processor.demolishes().to_vec(),
            boost_pad_events: processor.boost_pad_events().to_vec(),
            boost_pads,
            touch_events: processor.touch_events().to_vec(),
            dodge_refreshed_events: processor.dodge_refreshed_events().to_vec(),
            player_stat_events: processor.player_stat_events().to_vec(),
            goal_events: processor.goal_events().to_vec(),
            frame_data: self.get_frame_data(),
        })
    }

    /// Processes a replay and returns complete replay data.
    ///
    /// This method processes the entire replay using a [`ReplayProcessor`] and
    /// extracts all available information including frame-by-frame data, metadata,
    /// and special events like demolitions.
    ///
    /// # Arguments
    ///
    /// * `replay` - The parsed replay data from the [`boxcars`] library
    ///
    /// # Returns
    ///
    /// Returns a [`SubtrActorResult`] containing the complete [`ReplayData`] structure
    /// with all extracted information.
    ///
    /// # Errors
    ///
    /// Returns a [`SubtrActorError`] if:
    /// - The replay processor cannot be created
    /// - Frame processing fails
    /// - Replay metadata cannot be extracted
    ///
    /// # Example
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
    /// println!("Processed {} frames", replay_data.frame_data.frame_count());
    /// ```
    pub fn get_replay_data(mut self, replay: &boxcars::Replay) -> SubtrActorResult<ReplayData> {
        let mut processor = ReplayProcessor::new(replay)?;
        let mut boost_pad_collector = ResolvedBoostPadCollector::new();
        processor.process_all(&mut [&mut self, &mut boost_pad_collector])?;
        self.into_replay_data_with_boost_pads(
            processor,
            boost_pad_collector.into_resolved_boost_pads(),
        )
    }
}
