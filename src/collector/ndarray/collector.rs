use super::builtins::*;
use super::traits::*;
use crate::collector::{Collector, TimeAdvance};
use crate::*;
use ::ndarray;
use boxcars;
use serde::Serialize;
use std::sync::Arc;

/// Column headers for the frame matrix emitted by [`NDArrayCollector`].
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NDArrayColumnHeaders {
    /// Column names emitted once per frame, independent of player ordering.
    pub global_headers: Vec<String>,
    /// Column names repeated once for each player in replay order.
    pub player_headers: Vec<String>,
}

impl NDArrayColumnHeaders {
    /// Builds a header set from global and per-player column names.
    pub fn new(global_headers: Vec<String>, player_headers: Vec<String>) -> Self {
        Self {
            global_headers,
            player_headers,
        }
    }
}

/// Replay metadata bundled with the ndarray column layout used to produce it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayMetaWithHeaders {
    /// Replay metadata describing the teams and player ordering.
    pub replay_meta: ReplayMeta,
    /// Column headers associated with the emitted ndarray rows.
    pub column_headers: NDArrayColumnHeaders,
}

impl ReplayMetaWithHeaders {
    /// Flattens the global and per-player headers using a default player prefix.
    pub fn headers_vec(&self) -> Vec<String> {
        self.headers_vec_from(|_, _info, index| format!("Player {index} - "))
    }

    /// Flattens the global and per-player headers with a custom player prefix.
    pub fn headers_vec_from<F>(&self, player_prefix_getter: F) -> Vec<String>
    where
        F: Fn(&Self, &PlayerInfo, usize) -> String,
    {
        self.column_headers
            .global_headers
            .iter()
            .cloned()
            .chain(self.replay_meta.player_order().enumerate().flat_map(
                move |(player_index, info)| {
                    let player_prefix = player_prefix_getter(self, info, player_index);
                    self.column_headers
                        .player_headers
                        .iter()
                        .map(move |header| format!("{player_prefix}{header}"))
                },
            ))
            .collect()
    }
}

/// Collects replay frames into a dense 2D feature matrix.
pub struct NDArrayCollector<F> {
    feature_adders: FeatureAdders<F>,
    player_feature_adders: PlayerFeatureAdders<F>,
    data: Vec<F>,
    replay_meta: Option<ReplayMeta>,
    frames_added: usize,
}

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

    fn try_get_frame_feature_count(&self) -> SubtrActorResult<usize> {
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

    fn maybe_set_replay_meta(&mut self, processor: &ReplayProcessor) -> SubtrActorResult<()> {
        if self.replay_meta.is_none() {
            self.replay_meta = Some(processor.get_replay_meta()?);
        }
        Ok(())
    }
}

impl<F> Collector for NDArrayCollector<F> {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
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

fn global_feature_adder_from_name<F>(
    name: &str,
) -> Option<Arc<dyn FeatureAdder<F> + Send + Sync + 'static>>
where
    F: TryFrom<f32> + Send + Sync + 'static,
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    match name {
        "BallRigidBody" => Some(BallRigidBody::<F>::arc_new()),
        "BallRigidBodyNoVelocities" => Some(BallRigidBodyNoVelocities::<F>::arc_new()),
        "BallRigidBodyQuaternions" => Some(BallRigidBodyQuaternions::<F>::arc_new()),
        "BallRigidBodyQuaternionVelocities" => {
            Some(BallRigidBodyQuaternionVelocities::<F>::arc_new())
        }
        "BallRigidBodyBasis" => Some(BallRigidBodyBasis::<F>::arc_new()),
        "VelocityAddedBallRigidBodyNoVelocities" => {
            Some(VelocityAddedBallRigidBodyNoVelocities::<F>::arc_new())
        }
        "InterpolatedBallRigidBodyNoVelocities" => {
            Some(InterpolatedBallRigidBodyNoVelocities::<F>::arc_new(0.0))
        }
        "SecondsRemaining" => Some(SecondsRemaining::<F>::arc_new()),
        "CurrentTime" => Some(CurrentTime::<F>::arc_new()),
        "FrameTime" => Some(FrameTime::<F>::arc_new()),
        "ReplicatedStateName" => Some(ReplicatedStateName::<F>::arc_new()),
        "ReplicatedGameStateTimeRemaining" => {
            Some(ReplicatedGameStateTimeRemaining::<F>::arc_new())
        }
        "BallHasBeenHit" => Some(BallHasBeenHit::<F>::arc_new()),
        _ => None,
    }
}

fn player_feature_adder_from_name<F>(
    name: &str,
) -> Option<Arc<dyn PlayerFeatureAdder<F> + Send + Sync + 'static>>
where
    F: TryFrom<f32> + Send + Sync + 'static,
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    match name {
        "PlayerRigidBody" => Some(PlayerRigidBody::<F>::arc_new()),
        "PlayerRigidBodyNoVelocities" => Some(PlayerRigidBodyNoVelocities::<F>::arc_new()),
        "PlayerRigidBodyQuaternions" => Some(PlayerRigidBodyQuaternions::<F>::arc_new()),
        "PlayerRigidBodyQuaternionVelocities" => {
            Some(PlayerRigidBodyQuaternionVelocities::<F>::arc_new())
        }
        "PlayerRigidBodyBasis" => Some(PlayerRigidBodyBasis::<F>::arc_new()),
        "PlayerRelativeBallPosition" => Some(PlayerRelativeBallPosition::<F>::arc_new()),
        "PlayerRelativeBallVelocity" => Some(PlayerRelativeBallVelocity::<F>::arc_new()),
        "PlayerLocalRelativeBallPosition" => Some(PlayerLocalRelativeBallPosition::<F>::arc_new()),
        "PlayerLocalRelativeBallVelocity" => Some(PlayerLocalRelativeBallVelocity::<F>::arc_new()),
        "VelocityAddedPlayerRigidBodyNoVelocities" => {
            Some(VelocityAddedPlayerRigidBodyNoVelocities::<F>::arc_new())
        }
        "InterpolatedPlayerRigidBodyNoVelocities" => {
            Some(InterpolatedPlayerRigidBodyNoVelocities::<F>::arc_new(0.003))
        }
        "PlayerBallDistance" | "PlayerDistanceToBall" => Some(PlayerBallDistance::<F>::arc_new()),
        "PlayerBoost" => Some(PlayerBoost::<F>::arc_new()),
        "PlayerJump" => Some(PlayerJump::<F>::arc_new()),
        "PlayerAnyJump" => Some(PlayerAnyJump::<F>::arc_new()),
        "PlayerDodgeRefreshed" => Some(PlayerDodgeRefreshed::<F>::arc_new()),
        "PlayerDemolishedBy" => Some(PlayerDemolishedBy::<F>::arc_new()),
        _ => None,
    }
}

impl<F> NDArrayCollector<F>
where
    F: TryFrom<f32> + Send + Sync + 'static,
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    /// Builds a collector from the registered string names of feature adders.
    pub fn from_strings_typed(fa_names: &[&str], pfa_names: &[&str]) -> SubtrActorResult<Self> {
        let feature_adders: Vec<Arc<dyn FeatureAdder<F> + Send + Sync>> = fa_names
            .iter()
            .map(|name| {
                global_feature_adder_from_name(name).ok_or_else(|| {
                    SubtrActorError::new(SubtrActorErrorVariant::UnknownFeatureAdderName(
                        name.to_string(),
                    ))
                })
            })
            .collect::<SubtrActorResult<Vec<_>>>()?;
        let player_feature_adders: Vec<Arc<dyn PlayerFeatureAdder<F> + Send + Sync>> = pfa_names
            .iter()
            .map(|name| {
                player_feature_adder_from_name(name).ok_or_else(|| {
                    SubtrActorError::new(SubtrActorErrorVariant::UnknownFeatureAdderName(
                        name.to_string(),
                    ))
                })
            })
            .collect::<SubtrActorResult<Vec<_>>>()?;
        Ok(Self::new(feature_adders, player_feature_adders))
    }
}

impl NDArrayCollector<f32> {
    /// Builds an `f32` collector from the registered string names of feature adders.
    pub fn from_strings(fa_names: &[&str], pfa_names: &[&str]) -> SubtrActorResult<Self> {
        Self::from_strings_typed(fa_names, pfa_names)
    }
}

impl<F: TryFrom<f32> + Send + Sync + 'static> Default for NDArrayCollector<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    fn default() -> Self {
        NDArrayCollector::new(
            vec![BallRigidBody::arc_new()],
            vec![
                PlayerRigidBody::arc_new(),
                PlayerBoost::arc_new(),
                PlayerAnyJump::arc_new(),
            ],
        )
    }
}
