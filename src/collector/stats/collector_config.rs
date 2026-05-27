use std::marker::PhantomData;

use serde_json::{Map, Value};

use crate::collector::frame_resolution::{StatsFramePersistenceController, StatsFrameResolution};
use crate::SubtrActorResult;

use super::super::playback::CapturedStatsFrame;
use super::{BuiltinModuleSelection, ModuleFrameTransform, SampleMode, StatsCollector};

impl<T, F> StatsCollector<T, F> {
    pub(super) fn with_selection_and_frame_transform(
        modules: BuiltinModuleSelection,
        frame_transform: F,
    ) -> SubtrActorResult<Self> {
        Ok(Self {
            graph: modules.graph()?,
            modules,
            replay_meta: None,
            last_replay_meta_player_count: None,
            frame_transform,
            captured_frames: None,
            sample_mode: SampleMode::Aggregate,
            last_sample_time: None,
            frame_persistence: StatsFramePersistenceController::new(StatsFrameResolution::default()),
            last_demolish_count: 0,
            last_boost_pad_event_count: 0,
            last_touch_event_count: 0,
            last_dodge_refreshed_event_count: 0,
            last_player_stat_event_count: 0,
            last_goal_event_count: 0,
            _marker: PhantomData,
        })
    }

    pub fn capture_frames(mut self) -> Self {
        self.captured_frames = Some(Vec::new());
        self.sample_mode = SampleMode::Timeline;
        self
    }

    pub fn with_frame_resolution(mut self, resolution: StatsFrameResolution) -> Self {
        self.frame_persistence = StatsFramePersistenceController::new(resolution);
        self
    }

    pub fn with_module_transform<Modules, G>(
        self,
        transform: G,
    ) -> StatsCollector<CapturedStatsFrame<Modules>, ModuleFrameTransform<G>>
    where
        G: FnMut(Map<String, Value>) -> SubtrActorResult<Modules>,
    {
        self.with_frame_transform(ModuleFrameTransform::new(transform))
    }
}

impl<T, F> StatsCollector<T, F> {
    pub fn with_frame_transform<U, G>(self, frame_transform: G) -> StatsCollector<U, G> {
        let StatsCollector {
            modules,
            graph,
            replay_meta,
            last_replay_meta_player_count,
            captured_frames,
            sample_mode,
            last_sample_time,
            frame_persistence,
            last_demolish_count,
            last_boost_pad_event_count,
            last_touch_event_count,
            last_dodge_refreshed_event_count,
            last_player_stat_event_count,
            last_goal_event_count,
            ..
        } = self;
        StatsCollector {
            modules,
            graph,
            replay_meta,
            last_replay_meta_player_count,
            frame_transform,
            captured_frames: captured_frames.map(|_| Vec::new()),
            sample_mode,
            last_sample_time,
            frame_persistence,
            last_demolish_count,
            last_boost_pad_event_count,
            last_touch_event_count,
            last_dodge_refreshed_event_count,
            last_player_stat_event_count,
            last_goal_event_count,
            _marker: PhantomData,
        }
    }
}
