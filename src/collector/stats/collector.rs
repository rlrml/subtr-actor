use std::collections::HashSet;
use std::sync::Arc;

use crate::*;

use super::builtins::{builtin_stats_module_factories, builtin_stats_module_factory_by_name};
use super::resolver::resolve_stats_module_factories;
use super::types::{CollectedStats, RuntimeStatsModule, StatsModuleFactory};

#[derive(Default)]
struct CompositeStatsModules {
    modules: Vec<RuntimeStatsModule>,
}

impl CompositeStatsModules {
    fn into_modules(self) -> Vec<RuntimeStatsModule> {
        self.modules
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

pub struct StatsCollector {
    collector: ReducerCollector<CompositeStatsModules>,
    replay_meta: Option<ReplayMeta>,
}

impl Default for StatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsCollector {
    pub fn new() -> Self {
        Self::with_modules(builtin_stats_module_factories())
            .expect("builtin stats modules should resolve without conflicts")
    }

    pub fn with_modules<I>(modules: I) -> SubtrActorResult<Self>
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
        Ok(Self {
            collector: ReducerCollector::new(composite),
            replay_meta: None,
        })
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

    pub fn get_stats(mut self, replay: &boxcars::Replay) -> SubtrActorResult<CollectedStats> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        if self.replay_meta.is_none() {
            self.replay_meta = Some(processor.get_replay_meta()?);
        }
        self.into_stats()
    }

    pub fn into_stats(self) -> SubtrActorResult<CollectedStats> {
        let replay_meta = self
            .replay_meta
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))?;
        Ok(CollectedStats {
            replay_meta,
            modules: self.collector.into_inner().into_modules(),
        })
    }
}

impl Collector for StatsCollector {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        self.collector
            .process_frame(processor, frame, frame_number, current_time)
    }

    fn finish_replay(&mut self, processor: &ReplayProcessor) -> SubtrActorResult<()> {
        self.collector.finish_replay(processor)?;
        if self.replay_meta.is_none() {
            self.replay_meta = Some(processor.get_replay_meta()?);
        }
        Ok(())
    }
}
