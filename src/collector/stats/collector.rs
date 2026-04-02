use std::collections::HashSet;
use std::marker::PhantomData;

use serde_json::{Map, Value};

use crate::stats::analysis_nodes::graph_with_builtin_analysis_nodes;
use crate::*;

use super::builtins::{
    builtin_module_json, builtin_playback_config_json, builtin_playback_frame_json,
    builtin_stats_module_names,
};
use super::playback::{
    CapturedStatsData, CapturedStatsFrame, StatsPlaybackData, StatsPlaybackFrame,
};
use super::types::{serialize_to_json_value, CollectedStats, CollectedStatsModule};

enum SampleMode {
    Aggregate,
    Timeline,
}

impl Default for SampleMode {
    fn default() -> Self {
        Self::Aggregate
    }
}

struct BuiltinModuleSelection {
    module_names: Vec<&'static str>,
}

impl BuiltinModuleSelection {
    fn all() -> Self {
        Self {
            module_names: builtin_stats_module_names().to_vec(),
        }
    }

    fn from_names<I, S>(module_names: I) -> SubtrActorResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut selected = Vec::new();
        let mut seen = HashSet::new();
        for module_name in module_names {
            let module_name = module_name.as_ref();
            let resolved_name = builtin_stats_module_names()
                .iter()
                .copied()
                .find(|candidate| *candidate == module_name)
                .ok_or_else(|| {
                    SubtrActorError::new(SubtrActorErrorVariant::UnknownStatsModuleName(
                        module_name.to_owned(),
                    ))
                })?;
            if seen.insert(resolved_name) {
                selected.push(resolved_name);
            }
        }
        Ok(Self {
            module_names: selected,
        })
    }

    fn graph(
        &self,
    ) -> SubtrActorResult<crate::stats::analysis_nodes::analysis_graph::AnalysisGraph> {
        graph_with_builtin_analysis_nodes(self.module_names.iter().copied())
    }

    fn emitted_module_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.module_names.iter().copied()
    }

    fn collected_modules(
        &self,
        graph: &crate::stats::analysis_nodes::analysis_graph::AnalysisGraph,
    ) -> SubtrActorResult<Vec<CollectedStatsModule>> {
        self.module_names
            .iter()
            .copied()
            .map(|module_name| {
                Ok(CollectedStatsModule {
                    name: module_name,
                    value: builtin_module_json(module_name, graph)?,
                })
            })
            .collect()
    }

    fn modules_json(
        &self,
        graph: &crate::stats::analysis_nodes::analysis_graph::AnalysisGraph,
    ) -> SubtrActorResult<Map<String, Value>> {
        let mut modules = Map::new();
        for module_name in self.module_names.iter().copied() {
            modules.insert(
                module_name.to_owned(),
                builtin_module_json(module_name, graph)?,
            );
        }
        Ok(modules)
    }

    fn frame_modules_json(
        &self,
        graph: &crate::stats::analysis_nodes::analysis_graph::AnalysisGraph,
        replay_meta: &ReplayMeta,
    ) -> SubtrActorResult<Map<String, Value>> {
        let mut modules = Map::new();
        for module_name in self.module_names.iter().copied() {
            if let Some(snapshot) = builtin_playback_frame_json(module_name, graph, replay_meta)? {
                modules.insert(module_name.to_owned(), snapshot);
            }
        }
        Ok(modules)
    }

    fn playback_config_json(
        &self,
        graph: &crate::stats::analysis_nodes::analysis_graph::AnalysisGraph,
    ) -> SubtrActorResult<Map<String, Value>> {
        let mut config = Map::new();
        for module_name in self.module_names.iter().copied() {
            if let Some(module_config) = builtin_playback_config_json(module_name, graph)? {
                config.insert(module_name.to_owned(), module_config);
            }
        }
        Ok(config)
    }

    fn snapshot_frame(
        &self,
        graph: &crate::stats::analysis_nodes::analysis_graph::AnalysisGraph,
        replay_meta: &ReplayMeta,
    ) -> SubtrActorResult<StatsPlaybackFrame> {
        let frame = graph.state::<FrameInfo>().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing FrameInfo state while snapshotting playback frame".to_owned(),
            ))
        })?;
        let gameplay = graph.state::<GameplayState>().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing GameplayState state while snapshotting playback frame".to_owned(),
            ))
        })?;
        let is_live_play = graph
            .state::<LivePlayState>()
            .map(|state| state.is_live_play)
            .unwrap_or(false);
        Ok(StatsPlaybackFrame {
            frame_number: frame.frame_number,
            time: frame.time,
            dt: frame.dt,
            seconds_remaining: frame.seconds_remaining,
            game_state: gameplay.game_state,
            is_live_play,
            modules: self.frame_modules_json(graph, replay_meta)?,
        })
    }
}

pub trait FrameTransform {
    type Output;

    fn transform(
        &mut self,
        replay_meta: &ReplayMeta,
        frame: StatsPlaybackFrame,
    ) -> SubtrActorResult<Self::Output>;
}

impl<F, T> FrameTransform for F
where
    F: FnMut(&ReplayMeta, StatsPlaybackFrame) -> SubtrActorResult<T>,
{
    type Output = T;

    fn transform(
        &mut self,
        replay_meta: &ReplayMeta,
        frame: StatsPlaybackFrame,
    ) -> SubtrActorResult<Self::Output> {
        self(replay_meta, frame)
    }
}

#[derive(Default, Clone, Copy)]
pub struct IdentityFrameTransform;

impl FrameTransform for IdentityFrameTransform {
    type Output = StatsPlaybackFrame;

    fn transform(
        &mut self,
        _replay_meta: &ReplayMeta,
        frame: StatsPlaybackFrame,
    ) -> SubtrActorResult<Self::Output> {
        Ok(frame)
    }
}

pub struct ModuleFrameTransform<F> {
    transform: F,
}

impl<F> ModuleFrameTransform<F> {
    fn new(transform: F) -> Self {
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
        frame: StatsPlaybackFrame,
    ) -> SubtrActorResult<Self::Output> {
        frame.map_modules(&mut self.transform)
    }
}

struct ReplayStatsFrameTransform;

impl FrameTransform for ReplayStatsFrameTransform {
    type Output = ReplayStatsFrame;

    fn transform(
        &mut self,
        replay_meta: &ReplayMeta,
        frame: StatsPlaybackFrame,
    ) -> SubtrActorResult<Self::Output> {
        CapturedStatsData::<StatsPlaybackFrame> {
            replay_meta: replay_meta.clone(),
            config: Map::new(),
            modules: Map::new(),
            frames: Vec::new(),
        }
        .replay_stats_frame(&frame)
    }
}

pub struct StatsCollector<T = StatsPlaybackFrame, F = IdentityFrameTransform> {
    modules: BuiltinModuleSelection,
    graph: crate::stats::analysis_nodes::analysis_graph::AnalysisGraph,
    replay_meta: Option<ReplayMeta>,
    frame_transform: F,
    captured_frames: Option<Vec<T>>,
    sample_mode: SampleMode,
    last_sample_time: Option<f32>,
    last_demolish_count: usize,
    last_boost_pad_event_count: usize,
    last_touch_event_count: usize,
    last_player_stat_event_count: usize,
    last_goal_event_count: usize,
    _marker: PhantomData<T>,
}

impl Default for StatsCollector<StatsPlaybackFrame, IdentityFrameTransform> {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsCollector<StatsPlaybackFrame, IdentityFrameTransform> {
    pub fn new() -> Self {
        Self::with_selection_and_frame_transform(
            BuiltinModuleSelection::all(),
            IdentityFrameTransform,
        )
        .expect("builtin stats modules should resolve without conflicts")
    }

    pub fn only_modules<I>(modules: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        Self::try_only_modules(modules).expect("builtin stats module names should be valid")
    }

    pub fn try_only_modules<I>(modules: I) -> SubtrActorResult<Self>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        Self::with_builtin_module_names(modules)
    }

    pub fn with_builtin_module_names<I, S>(module_names: I) -> SubtrActorResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self::with_selection_and_frame_transform(
            BuiltinModuleSelection::from_names(module_names)?,
            IdentityFrameTransform,
        )
    }

    pub fn get_playback_data(self, replay: &boxcars::Replay) -> SubtrActorResult<StatsPlaybackData>
    where
        IdentityFrameTransform: FrameTransform<Output = StatsPlaybackFrame>,
    {
        self.capture_frames().get_captured_data(replay)
    }

    pub fn get_stats_timeline_value(self, replay: &boxcars::Replay) -> SubtrActorResult<Value> {
        serialize_to_json_value(&self.get_replay_stats_timeline(replay)?)
    }

    pub fn get_replay_stats_timeline(
        self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayStatsTimeline> {
        self.with_frame_transform(ReplayStatsFrameTransform)
            .capture_frames()
            .get_captured_data(replay)?
            .into_replay_stats_timeline()
    }

    pub fn get_legacy_stats_timeline_value(
        self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<Value> {
        self.get_stats_timeline_value(replay)
    }

    pub fn into_playback_data(self) -> SubtrActorResult<StatsPlaybackData> {
        self.into_captured_data()
    }

    pub fn into_stats_timeline_value(self) -> SubtrActorResult<Value> {
        self.into_playback_data()?.to_stats_timeline_value()
    }

    pub fn into_replay_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.into_playback_data()?.into_stats_timeline()
    }
}

impl<T, F> StatsCollector<T, F> {
    fn with_selection_and_frame_transform(
        modules: BuiltinModuleSelection,
        frame_transform: F,
    ) -> SubtrActorResult<Self> {
        Ok(Self {
            graph: modules.graph()?,
            modules,
            replay_meta: None,
            frame_transform,
            captured_frames: None,
            sample_mode: SampleMode::Aggregate,
            last_sample_time: None,
            last_demolish_count: 0,
            last_boost_pad_event_count: 0,
            last_touch_event_count: 0,
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

    pub fn with_frame_transform<U, G>(self, frame_transform: G) -> StatsCollector<U, G> {
        let StatsCollector {
            modules,
            graph,
            replay_meta,
            captured_frames,
            sample_mode,
            last_sample_time,
            last_demolish_count,
            last_boost_pad_event_count,
            last_touch_event_count,
            last_player_stat_event_count,
            last_goal_event_count,
            ..
        } = self;
        StatsCollector {
            modules,
            graph,
            replay_meta,
            frame_transform,
            captured_frames: captured_frames.map(|_| Vec::new()),
            sample_mode,
            last_sample_time,
            last_demolish_count,
            last_boost_pad_event_count,
            last_touch_event_count,
            last_player_stat_event_count,
            last_goal_event_count,
            _marker: PhantomData,
        }
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

    pub fn get_stats(mut self, replay: &boxcars::Replay) -> SubtrActorResult<CollectedStats>
    where
        F: FrameTransform<Output = T>,
    {
        self.sample_mode = SampleMode::Aggregate;
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        if self.replay_meta.is_none() {
            self.replay_meta = Some(processor.get_replay_meta()?);
        }
        self.into_stats()
    }

    pub fn get_captured_data(
        mut self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<CapturedStatsData<T>>
    where
        F: FrameTransform<Output = T>,
    {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        if self.replay_meta.is_none() {
            self.replay_meta = Some(processor.get_replay_meta()?);
        }
        self.into_captured_data()
    }

    pub fn into_stats(self) -> SubtrActorResult<CollectedStats> {
        let replay_meta = self
            .replay_meta
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))?;
        Ok(CollectedStats {
            replay_meta,
            modules: self.modules.collected_modules(&self.graph)?,
        })
    }

    pub fn into_captured_data(self) -> SubtrActorResult<CapturedStatsData<T>> {
        let replay_meta = self
            .replay_meta
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))?;
        Ok(CapturedStatsData {
            replay_meta: replay_meta.clone(),
            config: self.modules.playback_config_json(&self.graph)?,
            modules: self.modules.modules_json(&self.graph)?,
            frames: self.captured_frames.unwrap_or_default(),
        })
    }

    fn capture_frame_snapshot(
        &mut self,
        replay_meta: &ReplayMeta,
        frame: StatsPlaybackFrame,
    ) -> SubtrActorResult<()>
    where
        F: FrameTransform<Output = T>,
    {
        if let Some(frames) = &mut self.captured_frames {
            frames.push(self.frame_transform.transform(replay_meta, frame)?);
        }
        Ok(())
    }

    fn replace_last_frame_snapshot(
        &mut self,
        replay_meta: &ReplayMeta,
        frame: StatsPlaybackFrame,
    ) -> SubtrActorResult<()>
    where
        F: FrameTransform<Output = T>,
    {
        if let Some(frames) = &mut self.captured_frames {
            if let Some(last_frame) = frames.last_mut() {
                *last_frame = self.frame_transform.transform(replay_meta, frame)?;
            }
        }
        Ok(())
    }
}

impl<T, F> Collector for StatsCollector<T, F>
where
    F: FrameTransform<Output = T>,
{
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        if self.replay_meta.is_none() {
            let replay_meta = processor.get_replay_meta()?;
            self.graph.on_replay_meta(&replay_meta)?;
            self.replay_meta = Some(replay_meta);
        }

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let frame_input = match self.sample_mode {
            SampleMode::Aggregate => FrameInput::aggregate(
                processor,
                frame_number,
                current_time,
                dt,
                self.last_demolish_count,
                self.last_boost_pad_event_count,
                self.last_touch_event_count,
                self.last_player_stat_event_count,
                self.last_goal_event_count,
            ),
            SampleMode::Timeline => FrameInput::timeline(processor, frame_number, current_time, dt),
        };
        self.graph.evaluate_with_state(&frame_input)?;

        if self.captured_frames.is_some() {
            let replay_meta = self
                .replay_meta
                .as_ref()
                .expect("replay metadata should be initialized before snapshotting")
                .clone();
            self.capture_frame_snapshot(
                &replay_meta,
                self.modules.snapshot_frame(&self.graph, &replay_meta)?,
            )?;
        }

        self.last_sample_time = Some(current_time);
        if matches!(self.sample_mode, SampleMode::Aggregate) {
            self.last_demolish_count = processor.demolishes.len();
            self.last_boost_pad_event_count = processor.boost_pad_events.len();
            self.last_touch_event_count = processor.touch_events.len();
            self.last_player_stat_event_count = processor.player_stat_events.len();
            self.last_goal_event_count = processor.goal_events.len();
        }

        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &ReplayProcessor) -> SubtrActorResult<()> {
        self.graph.finish()?;
        let Some(replay_meta) = self.replay_meta.as_ref().cloned() else {
            return Ok(());
        };
        let Some(_) = self.graph.state::<FrameInfo>() else {
            return Ok(());
        };
        let final_snapshot = self.modules.snapshot_frame(&self.graph, &replay_meta)?;
        if self.captured_frames.is_some() {
            self.replace_last_frame_snapshot(&replay_meta, final_snapshot)?;
        }
        Ok(())
    }
}
