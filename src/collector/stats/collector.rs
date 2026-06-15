use std::collections::HashSet;
use std::marker::PhantomData;

use serde_json::{Map, Value};

use crate::collector::frame_resolution::{
    FinalStatsFrameAction, StatsFramePersistenceController, StatsFrameResolution,
};
use crate::stats::analysis_graph::{AnalysisGraph, graph_with_builtin_analysis_nodes};
use crate::stats::calculators::ReplayFrameInputBuilder;
use crate::*;

use super::builtins::{
    builtin_module_json, builtin_snapshot_config_json, builtin_snapshot_frame_json,
    builtin_stats_module_names,
};
use super::playback::{
    CapturedStatsData, CapturedStatsFrame, StatsSnapshotData, StatsSnapshotFrame,
};
use super::types::{CollectedStats, CollectedStatsModule, serialize_to_json_value};

#[derive(Default)]
enum SampleMode {
    #[default]
    Aggregate,
    Timeline,
}

/// Map a stats-module name to the analysis node that provides its state.
///
/// Most modules share their providing node's name. The exceptions are modules
/// that are a second view onto another node's calculator: `core` is served by
/// the `match_stats` node, and `air_dribble` by the `ball_carry` node. This is
/// the only place that translation lives — there is no global node-name alias
/// table.
fn stats_module_analysis_node_name(module_name: &str) -> &str {
    match module_name {
        "core" => "match_stats",
        "air_dribble" => "ball_carry",
        other => other,
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

    fn graph(&self) -> SubtrActorResult<AnalysisGraph> {
        if self.module_names == builtin_stats_module_names() {
            return Ok(build_legacy_timeline_graph());
        }
        let mut node_names: Vec<&str> = self
            .module_names
            .iter()
            .map(|module_name| stats_module_analysis_node_name(module_name))
            .collect();
        node_names.push("stats_projection");
        graph_with_builtin_analysis_nodes(node_names)
    }

    fn collected_modules(
        &self,
        graph: &AnalysisGraph,
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

    fn modules_json(&self, graph: &AnalysisGraph) -> SubtrActorResult<Map<String, Value>> {
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
        graph: &AnalysisGraph,
        replay_meta: &ReplayMeta,
    ) -> SubtrActorResult<Map<String, Value>> {
        let mut modules = Map::new();
        for module_name in self.module_names.iter().copied() {
            if let Some(snapshot) = builtin_snapshot_frame_json(module_name, graph, replay_meta)? {
                modules.insert(module_name.to_owned(), snapshot);
            }
            if module_name == "ball_carry" {
                if let Some(snapshot) =
                    builtin_snapshot_frame_json("air_dribble", graph, replay_meta)?
                {
                    modules.insert("air_dribble".to_owned(), snapshot);
                }
            }
        }
        Ok(modules)
    }

    fn snapshot_config_json(&self, graph: &AnalysisGraph) -> SubtrActorResult<Map<String, Value>> {
        let mut config = Map::new();
        for module_name in self.module_names.iter().copied() {
            if let Some(module_config) = builtin_snapshot_config_json(module_name, graph)? {
                config.insert(module_name.to_owned(), module_config);
            }
        }
        Ok(config)
    }

    fn snapshot_frame(
        &self,
        graph: &AnalysisGraph,
        replay_meta: &ReplayMeta,
    ) -> SubtrActorResult<StatsSnapshotFrame> {
        let frame = graph.state::<FrameInfo>().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing FrameInfo state while snapshotting stats frame".to_owned(),
            ))
        })?;
        let gameplay = graph.state::<GameplayState>().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing GameplayState state while snapshotting stats frame".to_owned(),
            ))
        })?;
        let live_play_state = graph.state::<LivePlayState>().cloned().unwrap_or_default();
        Ok(StatsSnapshotFrame {
            frame_number: frame.frame_number,
            time: frame.time,
            dt: frame.dt,
            seconds_remaining: frame.seconds_remaining,
            game_state: gameplay.game_state,
            ball_has_been_hit: gameplay.ball_has_been_hit,
            kickoff_countdown_time: gameplay.kickoff_countdown_time,
            gameplay_phase: live_play_state.gameplay_phase,
            is_live_play: live_play_state.is_live_play,
            modules: self.frame_modules_json(graph, replay_meta)?,
        })
    }
}

pub fn builtin_stats_graph_snapshot_json(
    graph: &AnalysisGraph,
    replay_meta: Option<&ReplayMeta>,
) -> SubtrActorResult<Value> {
    let modules = BuiltinModuleSelection::all();
    let frame = if let Some(replay_meta) = replay_meta {
        if graph.state::<FrameInfo>().is_some() && graph.state::<GameplayState>().is_some() {
            serialize_to_json_value(&modules.snapshot_frame(graph, replay_meta)?)?
        } else {
            Value::Null
        }
    } else {
        Value::Null
    };

    let mut payload = Map::new();
    payload.insert(
        "module_names".to_owned(),
        serialize_to_json_value(&modules.module_names)?,
    );
    payload.insert(
        "config".to_owned(),
        Value::Object(modules.snapshot_config_json(graph)?),
    );
    payload.insert(
        "modules".to_owned(),
        Value::Object(modules.modules_json(graph)?),
    );
    payload.insert("frame".to_owned(), frame);
    Ok(Value::Object(payload))
}

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
        frame: StatsSnapshotFrame,
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

pub struct StatsCollector<T = StatsSnapshotFrame, F = IdentityFrameTransform> {
    modules: BuiltinModuleSelection,
    graph: AnalysisGraph,
    replay_meta: Option<ReplayMeta>,
    frame_transform: F,
    captured_frames: Option<Vec<T>>,
    sample_mode: SampleMode,
    frame_input_builder: ReplayFrameInputBuilder,
    last_replay_meta_player_count: Option<usize>,
    last_sample_time: Option<f32>,
    frame_persistence: StatsFramePersistenceController,
    _marker: PhantomData<T>,
}

impl Default for StatsCollector<StatsSnapshotFrame, IdentityFrameTransform> {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsCollector<StatsSnapshotFrame, IdentityFrameTransform> {
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

    pub fn get_snapshot_data(self, replay: &boxcars::Replay) -> SubtrActorResult<StatsSnapshotData>
    where
        IdentityFrameTransform: FrameTransform<Output = StatsSnapshotFrame>,
    {
        self.capture_frames().get_captured_data(replay)
    }

    /// Collect the legacy full per-frame stats timeline as JSON.
    ///
    /// This serializes cumulative team/player partial sums on every captured
    /// frame. Prefer `StatsTimelineEventCollector` for compact event-backed
    /// timeline transfer.
    pub fn get_legacy_stats_timeline_value(
        self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<Value> {
        serialize_to_json_value(&self.get_legacy_replay_stats_timeline(replay)?)
    }

    /// Collect the legacy full per-frame stats timeline.
    ///
    /// This preserves the pre-event-transfer snapshot shape for compatibility
    /// and parity checks.
    pub fn get_legacy_replay_stats_timeline(
        self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayStatsTimeline> {
        self.with_frame_transform(ReplayStatsFrameTransform)
            .capture_frames()
            .get_captured_data(replay)?
            .into_legacy_replay_stats_timeline()
    }

    #[deprecated(
        note = "use get_legacy_stats_timeline_value for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn get_stats_timeline_value(self, replay: &boxcars::Replay) -> SubtrActorResult<Value> {
        self.get_legacy_stats_timeline_value(replay)
    }

    #[deprecated(
        note = "use get_legacy_stats_timeline_value for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn get_replay_stats_timeline(
        self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayStatsTimeline> {
        self.get_legacy_replay_stats_timeline(replay)
    }

    pub fn into_snapshot_data(self) -> SubtrActorResult<StatsSnapshotData> {
        self.into_captured_data()
    }

    pub fn into_legacy_stats_timeline_value(self) -> SubtrActorResult<Value> {
        self.into_snapshot_data()?.to_legacy_stats_timeline_value()
    }

    pub fn into_legacy_replay_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.into_snapshot_data()?
            .into_legacy_replay_stats_timeline()
    }

    #[deprecated(
        note = "use into_legacy_stats_timeline_value for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn into_stats_timeline_value(self) -> SubtrActorResult<Value> {
        self.into_legacy_stats_timeline_value()
    }

    #[deprecated(
        note = "use into_legacy_replay_stats_timeline for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn into_replay_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.into_legacy_replay_stats_timeline()
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
            frame_input_builder: ReplayFrameInputBuilder::default(),
            last_replay_meta_player_count: None,
            last_sample_time: None,
            frame_persistence: StatsFramePersistenceController::new(StatsFrameResolution::default()),
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
            frame_input_builder,
            last_replay_meta_player_count,
            last_sample_time,
            frame_persistence,
            ..
        } = self;
        StatsCollector {
            modules,
            graph,
            replay_meta,
            frame_transform,
            captured_frames: captured_frames.map(|_| Vec::new()),
            sample_mode,
            frame_input_builder,
            last_replay_meta_player_count,
            last_sample_time,
            frame_persistence,
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

    pub fn with_frame_resolution(mut self, resolution: StatsFrameResolution) -> Self {
        self.frame_persistence = StatsFramePersistenceController::new(resolution);
        self
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
            config: self.modules.snapshot_config_json(&self.graph)?,
            modules: self.modules.modules_json(&self.graph)?,
            frames: self.captured_frames.unwrap_or_default(),
        })
    }

    fn capture_frame_snapshot(
        &mut self,
        replay_meta: &ReplayMeta,
        frame: StatsSnapshotFrame,
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
        frame: StatsSnapshotFrame,
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

    fn refresh_replay_meta(&mut self, processor: &dyn ProcessorView) -> SubtrActorResult<()> {
        let player_count = processor.player_count();
        if self.last_replay_meta_player_count == Some(player_count) {
            return Ok(());
        }

        let replay_meta = processor.get_replay_meta()?;
        self.graph.on_replay_meta(&replay_meta)?;
        self.replay_meta = Some(replay_meta);
        self.last_replay_meta_player_count = Some(player_count);
        Ok(())
    }
}

impl<T, F> Collector for StatsCollector<T, F>
where
    F: FrameTransform<Output = T>,
{
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        self.refresh_replay_meta(processor)?;

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let frame_input = match self.sample_mode {
            SampleMode::Aggregate => {
                self.frame_input_builder
                    .aggregate(processor, frame_number, current_time, dt)
            }
            SampleMode::Timeline => {
                self.frame_input_builder
                    .timeline(processor, frame_number, current_time, dt)
            }
        };
        self.graph.evaluate_with_state(&frame_input)?;

        if self.captured_frames.is_some() {
            let replay_meta = self
                .replay_meta
                .as_ref()
                .expect("replay metadata should be initialized before snapshotting")
                .clone();
            if let Some(emitted_dt) = self.frame_persistence.on_frame(frame_number, current_time) {
                let mut frame = self.modules.snapshot_frame(&self.graph, &replay_meta)?;
                frame.dt = emitted_dt;
                self.capture_frame_snapshot(&replay_meta, frame)?;
            }
        }
        self.last_sample_time = Some(current_time);

        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &dyn ProcessorView) -> SubtrActorResult<()> {
        self.graph.finish()?;
        let Some(replay_meta) = self.replay_meta.as_ref().cloned() else {
            return Ok(());
        };
        let Some(_) = self.graph.state::<FrameInfo>() else {
            return Ok(());
        };
        let mut final_snapshot = self.modules.snapshot_frame(&self.graph, &replay_meta)?;
        if self.captured_frames.is_some() {
            match self
                .frame_persistence
                .final_frame_action(final_snapshot.frame_number, final_snapshot.time)
            {
                Some(FinalStatsFrameAction::Append { dt }) => {
                    final_snapshot.dt = dt;
                    self.capture_frame_snapshot(&replay_meta, final_snapshot)?;
                }
                Some(FinalStatsFrameAction::ReplaceLast { dt }) => {
                    final_snapshot.dt = dt;
                    self.replace_last_frame_snapshot(&replay_meta, final_snapshot)?;
                }
                None => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "collector_tests.rs"]
mod tests;
