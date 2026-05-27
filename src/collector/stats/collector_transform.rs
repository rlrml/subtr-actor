use serde_json::{Map, Value};

use crate::{ReplayMeta, ReplayStatsFrame, StatsSnapshotFrame, SubtrActorResult};

use super::super::playback::{CapturedStatsData, CapturedStatsFrame};

pub trait FrameTransform {
    type Output;

    fn transform(
        &mut self,
        replay_meta: &ReplayMeta,
        frame: StatsSnapshotFrame,
    ) -> SubtrActorResult<Self::Output>;
}

impl<F, T> FrameTransform for F
where
    F: FnMut(&ReplayMeta, StatsSnapshotFrame) -> SubtrActorResult<T>,
{
    type Output = T;

    fn transform(
        &mut self,
        replay_meta: &ReplayMeta,
        frame: StatsSnapshotFrame,
    ) -> SubtrActorResult<Self::Output> {
        self(replay_meta, frame)
    }
}

#[derive(Default, Clone, Copy)]
pub struct IdentityFrameTransform;

impl FrameTransform for IdentityFrameTransform {
    type Output = StatsSnapshotFrame;

    fn transform(
        &mut self,
        _replay_meta: &ReplayMeta,
        frame: StatsSnapshotFrame,
    ) -> SubtrActorResult<Self::Output> {
        Ok(frame)
    }
}

pub struct ModuleFrameTransform<F> {
    transform: F,
}

impl<F> ModuleFrameTransform<F> {
    pub(super) fn new(transform: F) -> Self {
        Self { transform }
    }
}

impl<F, Modules> FrameTransform for ModuleFrameTransform<F>
where
    F: FnMut(Map<String, Value>) -> SubtrActorResult<Modules>,
{
    type Output = CapturedStatsFrame<Modules>;

    fn transform(
        &mut self,
        _replay_meta: &ReplayMeta,
        frame: StatsSnapshotFrame,
    ) -> SubtrActorResult<Self::Output> {
        frame.map_modules(&mut self.transform)
    }
}

pub(super) struct ReplayStatsFrameTransform;

impl FrameTransform for ReplayStatsFrameTransform {
    type Output = ReplayStatsFrame;

    fn transform(
        &mut self,
        replay_meta: &ReplayMeta,
        frame: StatsSnapshotFrame,
    ) -> SubtrActorResult<Self::Output> {
        CapturedStatsData::<StatsSnapshotFrame> {
            replay_meta: replay_meta.clone(),
            config: Map::new(),
            modules: Map::new(),
            frames: Vec::new(),
        }
        .replay_stats_frame(&frame)
    }
}
