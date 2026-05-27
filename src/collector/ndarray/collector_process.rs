use super::*;

impl<F> NDArrayCollector<F> {
    /// Finalizes collection and returns replay metadata alongside the ndarray.
    pub fn get_meta_and_ndarray(
        self,
    ) -> SubtrActorResult<(ReplayMetaWithHeaders, ndarray::Array2<F>)> {
        let features_per_row = self.try_get_frame_feature_count()?;
        let expected_length = features_per_row * self.frames_added;
        assert!(self.data.len() == expected_length);
        let column_headers = self.get_column_headers();
        Ok((
            ReplayMetaWithHeaders {
                replay_meta: self.replay_meta.ok_or(SubtrActorError::new(
                    SubtrActorErrorVariant::CouldNotBuildReplayMeta,
                ))?,
                column_headers,
            },
            ndarray::Array2::from_shape_vec((self.frames_added, features_per_row), self.data)
                .map_err(SubtrActorErrorVariant::NDArrayShapeError)
                .map_err(SubtrActorError::new)?,
        ))
    }

    /// Processes enough of a replay to determine metadata and column headers.
    pub fn process_and_get_meta_and_headers(
        &mut self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayMetaWithHeaders> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process_long_enough_to_get_actor_ids()?;
        self.maybe_set_replay_meta(&processor)?;
        Ok(ReplayMetaWithHeaders {
            replay_meta: self
                .replay_meta
                .as_ref()
                .ok_or(SubtrActorError::new(
                    SubtrActorErrorVariant::CouldNotBuildReplayMeta,
                ))?
                .clone(),
            column_headers: self.get_column_headers(),
        })
    }
}

impl<F> Collector for NDArrayCollector<F> {
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        self.maybe_set_replay_meta(processor)?;

        for feature_adder in &self.feature_adders {
            feature_adder.add_features(
                processor,
                frame,
                frame_number,
                current_time,
                &mut self.data,
            )?;
        }

        for player_id in processor.iter_player_ids_in_order() {
            for player_feature_adder in &self.player_feature_adders {
                player_feature_adder.add_features(
                    player_id,
                    processor,
                    frame,
                    frame_number,
                    current_time,
                    &mut self.data,
                )?;
            }
        }

        self.frames_added += 1;
        Ok(TimeAdvance::NextFrame)
    }
}
