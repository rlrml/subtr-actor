use std::collections::HashSet;

use boxcars;

pub use super::calculators::core::*;
pub use super::calculators::standard_soccar_boost_pad_layout;
use crate::*;

pub trait StatsReducer {
    fn on_replay_meta(&mut self, _meta: &ReplayMeta) -> SubtrActorResult<()> {
        Ok(())
    }

    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        Vec::new()
    }

    fn on_sample(&mut self, _sample: &CoreSample) -> SubtrActorResult<()> {
        Ok(())
    }

    fn on_sample_with_context(
        &mut self,
        sample: &CoreSample,
        _ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        self.on_sample(sample)
    }

    fn finish(&mut self) -> SubtrActorResult<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct CompositeStatsReducer {
    children: Vec<Box<dyn StatsReducer>>,
}

impl CompositeStatsReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<R: StatsReducer + 'static>(&mut self, reducer: R) {
        self.children.push(Box::new(reducer));
    }

    pub fn with_child<R: StatsReducer + 'static>(mut self, reducer: R) -> Self {
        self.push(reducer);
        self
    }

    pub fn children(&self) -> &[Box<dyn StatsReducer>] {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut [Box<dyn StatsReducer>] {
        &mut self.children
    }
}

impl StatsReducer for CompositeStatsReducer {
    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        for child in &mut self.children {
            child.on_replay_meta(meta)?;
        }
        Ok(())
    }

    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        let mut signals = HashSet::new();
        for child in &self.children {
            signals.extend(child.required_derived_signals());
        }
        signals.into_iter().collect()
    }

    fn on_sample(&mut self, sample: &CoreSample) -> SubtrActorResult<()> {
        for child in &mut self.children {
            child.on_sample(sample)?;
        }
        Ok(())
    }

    fn on_sample_with_context(
        &mut self,
        sample: &CoreSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        for child in &mut self.children {
            child.on_sample_with_context(sample, ctx)?;
        }
        Ok(())
    }

    fn finish(&mut self) -> SubtrActorResult<()> {
        for child in &mut self.children {
            child.finish()?;
        }
        Ok(())
    }
}

pub struct ReducerCollector<R> {
    reducer: R,
    derived_signals: DerivedSignalGraph,
    last_sample_time: Option<f32>,
    replay_meta_initialized: bool,
    last_demolish_count: usize,
    last_boost_pad_event_count: usize,
    last_touch_event_count: usize,
    last_player_stat_event_count: usize,
    last_goal_event_count: usize,
}

impl<R: StatsReducer> ReducerCollector<R> {
    pub fn new(reducer: R) -> Self {
        let derived_signals = derived_signal_graph_for_ids(reducer.required_derived_signals());
        Self {
            reducer,
            derived_signals,
            last_sample_time: None,
            replay_meta_initialized: false,
            last_demolish_count: 0,
            last_boost_pad_event_count: 0,
            last_touch_event_count: 0,
            last_player_stat_event_count: 0,
            last_goal_event_count: 0,
        }
    }
}

impl<R> ReducerCollector<R> {
    pub fn into_inner(self) -> R {
        self.reducer
    }

    pub fn reducer(&self) -> &R {
        &self.reducer
    }

    pub fn reducer_mut(&mut self) -> &mut R {
        &mut self.reducer
    }
}

impl<R: StatsReducer> From<R> for ReducerCollector<R> {
    fn from(reducer: R) -> Self {
        Self::new(reducer)
    }
}

impl<R: StatsReducer> Collector for ReducerCollector<R> {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        if !self.replay_meta_initialized {
            let replay_meta = processor.get_replay_meta()?;
            self.derived_signals.on_replay_meta(&replay_meta)?;
            self.reducer.on_replay_meta(&replay_meta)?;
            self.replay_meta_initialized = true;
        }

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let mut sample = CoreSample::from_processor(processor, frame_number, current_time, dt)?;
        sample.active_demos.clear();
        sample.demo_events = processor.demolishes[self.last_demolish_count..].to_vec();
        sample.boost_pad_events =
            processor.boost_pad_events[self.last_boost_pad_event_count..].to_vec();
        sample.touch_events = processor.touch_events[self.last_touch_event_count..].to_vec();
        sample.player_stat_events =
            processor.player_stat_events[self.last_player_stat_event_count..].to_vec();
        sample.goal_events = processor.goal_events[self.last_goal_event_count..].to_vec();
        let analysis_context = self.derived_signals.evaluate(&sample)?;
        self.reducer
            .on_sample_with_context(&sample, analysis_context)?;
        self.last_sample_time = Some(current_time);
        self.last_demolish_count = processor.demolishes.len();
        self.last_boost_pad_event_count = processor.boost_pad_events.len();
        self.last_touch_event_count = processor.touch_events.len();
        self.last_player_stat_event_count = processor.player_stat_events.len();
        self.last_goal_event_count = processor.goal_events.len();

        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &ReplayProcessor) -> SubtrActorResult<()> {
        self.derived_signals.finish()?;
        self.reducer.finish()
    }
}

pub mod powerslide;
#[allow(unused_imports)]
pub use powerslide::*;
pub mod analysis;
pub use analysis::*;
pub mod pressure;
#[allow(unused_imports)]
pub use pressure::*;
pub mod rush;
#[allow(unused_imports)]
pub use rush::*;
pub mod possession;
#[allow(unused_imports)]
pub use possession::*;
pub mod settings;
pub use settings::*;
pub mod match_stats;
pub use match_stats::*;
pub mod backboard;
pub use backboard::*;
pub mod double_tap;
pub use double_tap::*;
pub mod demo;
pub use demo::*;
pub mod ceiling_shot;
pub use ceiling_shot::*;
pub mod dodge_reset;
pub use dodge_reset::*;
pub mod musty_flick;
pub use musty_flick::*;
pub mod touch;
pub use touch::*;
pub mod fifty_fifty;
pub use fifty_fifty::*;
pub mod speed_flip;
pub use speed_flip::*;
pub mod movement;
pub use movement::*;
pub mod positioning;
pub use positioning::*;
pub mod ball_carry;
pub use ball_carry::*;
pub mod boost;
pub use boost::*;
