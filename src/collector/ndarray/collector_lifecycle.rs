use super::*;

impl<F> NDArrayCollector<F> {
    /// Creates a collector from explicit global and per-player feature adders.
    pub fn new(
        feature_adders: FeatureAdders<F>,
        player_feature_adders: PlayerFeatureAdders<F>,
    ) -> Self {
        Self {
            feature_adders,
            player_feature_adders,
            data: Vec::new(),
            replay_meta: None,
            frames_added: 0,
        }
    }

    /// Returns the column headers implied by the configured feature adders.
    pub fn get_column_headers(&self) -> NDArrayColumnHeaders {
        let global_headers = self
            .feature_adders
            .iter()
            .flat_map(move |fa| {
                fa.get_column_headers()
                    .iter()
                    .map(move |column_name| column_name.to_string())
            })
            .collect();
        let player_headers = self
            .player_feature_adders
            .iter()
            .flat_map(move |pfa| {
                pfa.get_column_headers()
                    .iter()
                    .map(move |base_name| base_name.to_string())
            })
            .collect();
        NDArrayColumnHeaders::new(global_headers, player_headers)
    }

    /// Finalizes collection and returns only the ndarray payload.
    pub fn get_ndarray(self) -> SubtrActorResult<ndarray::Array2<F>> {
        self.get_meta_and_ndarray().map(|a| a.1)
    }

    pub(super) fn maybe_set_replay_meta(
        &mut self,
        processor: &dyn ProcessorView,
    ) -> SubtrActorResult<()> {
        if self.replay_meta.is_none() {
            self.replay_meta = Some(processor.get_replay_meta()?);
        }
        Ok(())
    }

    pub(super) fn try_get_frame_feature_count(&self) -> SubtrActorResult<usize> {
        let player_count = self
            .replay_meta
            .as_ref()
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::CouldNotBuildReplayMeta,
            ))?
            .player_count();
        let global_feature_count: usize = self
            .feature_adders
            .iter()
            .map(|fa| fa.features_added())
            .sum();
        let player_feature_count: usize = self
            .player_feature_adders
            .iter()
            .map(|pfa| pfa.features_added() * player_count)
            .sum();
        Ok(global_feature_count + player_feature_count)
    }
}
