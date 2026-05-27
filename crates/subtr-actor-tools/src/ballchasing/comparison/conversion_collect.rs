use subtr_actor::*;

pub(crate) struct ComputedComparableStats {
    pub(super) replay_meta: ReplayMeta,
    pub(super) match_stats: MatchStatsCalculator,
    pub(super) boost: BoostCalculator,
    pub(super) movement: MovementCalculator,
    pub(super) positioning: PositioningCalculator,
    pub(super) demo: DemoCalculator,
    pub(super) powerslide: PowerslideCalculator,
}

#[derive(Default)]
struct ReplayMetaCollector {
    replay_meta: Option<ReplayMeta>,
}

impl Collector for ReplayMetaCollector {
    fn process_frame(
        &mut self,
        _processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        _frame_number: usize,
        _current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, processor: &dyn ProcessorView) -> SubtrActorResult<()> {
        self.replay_meta = Some(processor.get_replay_meta()?);
        Ok(())
    }
}

pub(super) fn collect_final_replay_meta(replay: &boxcars::Replay) -> SubtrActorResult<ReplayMeta> {
    ReplayMetaCollector::default()
        .process_replay(replay)?
        .replay_meta
        .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::NoNetworkFrames))
}

pub(crate) fn compute_comparable_stats(
    replay: &boxcars::Replay,
) -> SubtrActorResult<ComputedComparableStats> {
    let replay_meta = collect_final_replay_meta(replay)?;
    let graph = subtr_actor::stats::analysis_graph::collect_builtin_analysis_graph_for_replay(
        replay,
        [
            "core",
            "boost",
            "movement",
            "positioning",
            "demo",
            "powerslide",
        ],
    )?;
    Ok(ComputedComparableStats {
        replay_meta,
        match_stats: graph
            .state::<MatchStatsCalculator>()
            .cloned()
            .unwrap_or_default(),
        boost: graph
            .state::<BoostCalculator>()
            .cloned()
            .unwrap_or_default(),
        movement: graph
            .state::<MovementCalculator>()
            .cloned()
            .unwrap_or_default(),
        positioning: graph
            .state::<PositioningCalculator>()
            .cloned()
            .unwrap_or_default(),
        demo: graph.state::<DemoCalculator>().cloned().unwrap_or_default(),
        powerslide: graph
            .state::<PowerslideCalculator>()
            .cloned()
            .unwrap_or_default(),
    })
}
