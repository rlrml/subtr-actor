use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::Arc;

use serde_json::{Map, Value};

use crate::*;

use super::builtins::{builtin_stats_module_factories, builtin_stats_module_factory_by_name};
use super::playback::{
    CapturedStatsData, CapturedStatsFrame, StatsPlaybackData, StatsPlaybackFrame,
};
use super::resolver::resolve_stats_module_factories;
use super::types::{
    serialize_to_json_value, stats_module_to_json_value, CollectedStats, RuntimeStatsModule,
    StatsModuleFactory,
};

enum SampleMode {
    Aggregate,
    Timeline,
}

impl Default for SampleMode {
    fn default() -> Self {
        Self::Aggregate
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

struct DynamicReplayStatsFrameTransform {
    modules: StatsTimelineModules,
}

impl DynamicReplayStatsFrameTransform {
    fn new(modules: StatsTimelineModules) -> Self {
        Self { modules }
    }
}

impl FrameTransform for DynamicReplayStatsFrameTransform {
    type Output = DynamicReplayStatsFrame;

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
        .dynamic_replay_stats_frame(&frame, &self.modules)
    }
}

#[derive(Default)]
struct CompositeStatsModules {
    modules: Vec<RuntimeStatsModule>,
}

impl CompositeStatsModules {
    fn into_modules(self) -> Vec<RuntimeStatsModule> {
        self.modules
    }

    fn emitted_module_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.modules
            .iter()
            .filter(|module| module.emit)
            .map(|module| module.module.name())
    }

    fn playback_modules_json(&self) -> SubtrActorResult<Map<String, Value>> {
        let mut modules = Map::new();
        for module in self.modules.iter().filter(|module| module.emit) {
            modules.insert(
                module.module.name().to_owned(),
                stats_module_to_json_value(module.module.as_ref())?,
            );
        }
        Ok(modules)
    }

    fn playback_config_json(&self) -> SubtrActorResult<Map<String, Value>> {
        let mut config = Map::new();
        for module in self.modules.iter().filter(|module| module.emit) {
            if let Some(module_config) = module.module.playback_config_json()? {
                config.insert(module.module.name().to_owned(), module_config);
            }
        }
        Ok(config)
    }

    fn snapshot_frame(
        &self,
        replay_meta: &ReplayMeta,
        sample: &StatsSample,
        is_live_play: bool,
    ) -> SubtrActorResult<StatsPlaybackFrame> {
        let mut modules = Map::new();
        for module in self.modules.iter().filter(|module| module.emit) {
            if let Some(snapshot) = module.module.playback_frame_json(replay_meta)? {
                modules.insert(module.module.name().to_owned(), snapshot);
            }
        }

        Ok(StatsPlaybackFrame {
            frame_number: sample.frame_number,
            time: sample.time,
            dt: sample.dt,
            seconds_remaining: sample.seconds_remaining,
            game_state: sample.game_state,
            is_live_play,
            modules,
        })
    }
}

impl StatsReducer for CompositeStatsModules {
    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        for module in &mut self.modules {
            module.module.on_replay_meta(meta)?;
        }
        Ok(())
    }

    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        let mut signals = HashSet::new();
        for module in &self.modules {
            signals.extend(module.module.required_derived_signals());
        }
        signals.into_iter().collect()
    }

    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        for module in &mut self.modules {
            module.module.on_sample(sample)?;
        }
        Ok(())
    }

    fn on_sample_with_context(
        &mut self,
        sample: &StatsSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        for module in &mut self.modules {
            module.module.on_sample_with_context(sample, ctx)?;
        }
        Ok(())
    }

    fn finish(&mut self) -> SubtrActorResult<()> {
        for module in &mut self.modules {
            module.module.finish()?;
        }
        Ok(())
    }
}

pub struct StatsCollector<T = StatsPlaybackFrame, F = IdentityFrameTransform> {
    modules: CompositeStatsModules,
    derived_signals: DerivedSignalGraph,
    replay_meta: Option<ReplayMeta>,
    frame_transform: F,
    captured_frames: Option<Vec<T>>,
    sample_mode: SampleMode,
    last_sample_time: Option<f32>,
    last_sample: Option<StatsSample>,
    last_live_play: Option<bool>,
    live_play_tracker: LivePlayTracker,
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
        Self::with_modules(builtin_stats_module_factories())
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

    pub fn with_modules<I>(modules: I) -> SubtrActorResult<Self>
    where
        I: IntoIterator<Item = Arc<dyn StatsModuleFactory>>,
    {
        Self::with_modules_and_frame_transform(modules, IdentityFrameTransform)
    }

    pub fn with_builtin_module_names<I, S>(module_names: I) -> SubtrActorResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut modules = Vec::new();
        for module_name in module_names {
            let module_name = module_name.as_ref();
            modules.push(
                builtin_stats_module_factory_by_name(module_name).ok_or_else(|| {
                    SubtrActorError::new(SubtrActorErrorVariant::UnknownStatsModuleName(
                        module_name.to_owned(),
                    ))
                })?,
            );
        }
        Self::with_modules(modules)
    }

    pub fn get_playback_data(
        mut self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<StatsPlaybackData>
    where
        IdentityFrameTransform: FrameTransform<Output = StatsPlaybackFrame>,
    {
        self.captured_frames = Some(Vec::new());
        self.sample_mode = SampleMode::Timeline;
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        if self.replay_meta.is_none() {
            self.replay_meta = Some(processor.get_replay_meta()?);
        }
        self.into_playback_data()
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

    pub fn get_dynamic_replay_stats_timeline(
        self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<DynamicReplayStatsTimeline> {
        let modules = self.stats_timeline_modules()?;
        self.with_frame_transform(DynamicReplayStatsFrameTransform::new(modules))
            .capture_frames()
            .get_captured_data(replay)?
            .into_dynamic_replay_stats_timeline()
    }

    pub fn get_dynamic_stats_timeline_value(
        self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<Value> {
        serialize_to_json_value(&self.get_dynamic_replay_stats_timeline(replay)?)
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

    pub fn into_dynamic_replay_stats_timeline(
        self,
    ) -> SubtrActorResult<DynamicReplayStatsTimeline> {
        self.into_playback_data()?.into_dynamic_stats_timeline()
    }

    pub fn into_dynamic_stats_timeline_value(self) -> SubtrActorResult<Value> {
        self.into_playback_data()?.to_dynamic_stats_timeline_value()
    }
}

impl<T, F> StatsCollector<T, F> {
    pub fn with_modules_and_frame_transform<I>(
        modules: I,
        frame_transform: F,
    ) -> SubtrActorResult<Self>
    where
        I: IntoIterator<Item = Arc<dyn StatsModuleFactory>>,
    {
        let resolved = resolve_stats_module_factories(modules)?;
        let composite = CompositeStatsModules {
            modules: resolved
                .into_iter()
                .map(|resolved_module| RuntimeStatsModule {
                    emit: resolved_module.emit,
                    module: resolved_module.factory.build(),
                })
                .collect(),
        };
        let derived_signals = derived_signal_graph_for_ids(composite.required_derived_signals());
        Ok(Self {
            modules: composite,
            derived_signals,
            replay_meta: None,
            frame_transform,
            captured_frames: None,
            sample_mode: SampleMode::Aggregate,
            last_sample_time: None,
            last_sample: None,
            last_live_play: None,
            live_play_tracker: LivePlayTracker::default(),
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
            derived_signals,
            replay_meta,
            captured_frames,
            sample_mode,
            last_sample_time,
            last_sample,
            last_live_play,
            live_play_tracker,
            last_demolish_count,
            last_boost_pad_event_count,
            last_touch_event_count,
            last_player_stat_event_count,
            last_goal_event_count,
            ..
        } = self;
        StatsCollector {
            modules,
            derived_signals,
            replay_meta,
            frame_transform,
            captured_frames: captured_frames.map(|_| Vec::new()),
            sample_mode,
            last_sample_time,
            last_sample,
            last_live_play,
            live_play_tracker,
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
            modules: self.modules.into_modules(),
        })
    }

    pub fn into_captured_data(self) -> SubtrActorResult<CapturedStatsData<T>> {
        let replay_meta = self
            .replay_meta
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))?;
        Ok(CapturedStatsData {
            replay_meta,
            config: self.modules.playback_config_json()?,
            modules: self.modules.playback_modules_json()?,
            frames: self.captured_frames.unwrap_or_default(),
        })
    }

    fn stats_timeline_modules(&self) -> SubtrActorResult<StatsTimelineModules> {
        StatsTimelineModules::from_builtin_names(self.modules.emitted_module_names())
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
            self.derived_signals.on_replay_meta(&replay_meta)?;
            self.modules.on_replay_meta(&replay_meta)?;
            self.replay_meta = Some(replay_meta);
        }

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let mut sample = StatsSample::from_processor(processor, frame_number, current_time, dt)?;
        if matches!(self.sample_mode, SampleMode::Aggregate) {
            sample.active_demos.clear();
            sample.demo_events = processor.demolishes[self.last_demolish_count..].to_vec();
            sample.boost_pad_events =
                processor.boost_pad_events[self.last_boost_pad_event_count..].to_vec();
            sample.touch_events = processor.touch_events[self.last_touch_event_count..].to_vec();
            sample.player_stat_events =
                processor.player_stat_events[self.last_player_stat_event_count..].to_vec();
            sample.goal_events = processor.goal_events[self.last_goal_event_count..].to_vec();
        }
        let is_live_play = self.live_play_tracker.is_live_play(&sample);
        let analysis_context = self.derived_signals.evaluate(&sample)?;
        self.modules
            .on_sample_with_context(&sample, analysis_context)?;

        if self.captured_frames.is_some() {
            let replay_meta = self
                .replay_meta
                .as_ref()
                .expect("replay metadata should be initialized before snapshotting")
                .clone();
            self.capture_frame_snapshot(
                &replay_meta,
                self.modules
                    .snapshot_frame(&replay_meta, &sample, is_live_play)?,
            )?;
        }

        self.last_sample_time = Some(current_time);
        self.last_live_play = Some(is_live_play);
        self.last_sample = Some(sample);
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
        self.derived_signals.finish()?;
        self.modules.finish()?;
        let Some(last_sample) = self.last_sample.as_ref() else {
            return Ok(());
        };
        let Some(replay_meta) = self.replay_meta.as_ref().cloned() else {
            return Ok(());
        };
        let final_snapshot = self.modules.snapshot_frame(
            &replay_meta,
            last_sample,
            self.last_live_play.unwrap_or(false),
        )?;
        if self.captured_frames.is_some() {
            self.replace_last_frame_snapshot(&replay_meta, final_snapshot)?;
        }
        Ok(())
    }
}
