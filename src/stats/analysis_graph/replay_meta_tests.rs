use super::*;
use crate::stats::analysis_graph::{AnalysisNode, AnalysisNodeCollector, AnalysisStateContext};
use crate::{Collector, ReplayProcessor};

#[derive(Default)]
struct RosterState {
    players: Vec<PlayerId>,
    metadata_updates: usize,
    frames: usize,
}

#[derive(Default)]
struct RosterNode(RosterState);

impl AnalysisNode for RosterNode {
    type State = RosterState;

    fn name(&self) -> &'static str {
        "roster_test"
    }

    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        self.0.players = meta
            .player_order()
            .map(|player| player.remote_id.clone())
            .collect();
        self.0.metadata_updates += 1;
        Ok(())
    }

    fn evaluate(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.0.frames += 1;
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.0
    }
}

#[test]
fn same_count_player_replacement_refreshes_metadata_without_resetting_graph() {
    let bytes = std::fs::read("assets/post-eac-ranked-duel-2026-04-28-a.replay").unwrap();
    let replay = boxcars::ParserBuilder::new(&bytes)
        .must_parse_network_data()
        .parse()
        .unwrap();
    let mut processor = ReplayProcessor::new(&replay).unwrap();
    processor.process_long_enough_to_get_actor_ids().unwrap();
    let frame = &replay.network_frames.as_ref().unwrap().frames[0];
    let mut collector =
        AnalysisNodeCollector::new(AnalysisGraph::new().with_node(RosterNode::default()));
    collector.process_frame(&processor, frame, 0, 0.0).unwrap();

    let replacement = boxcars::RemoteId::Steam(u64::MAX);
    processor.team_zero[0] = replacement.clone();
    collector.process_frame(&processor, frame, 1, 0.1).unwrap();
    collector.process_frame(&processor, frame, 2, 0.2).unwrap();

    let state = collector.graph().state::<RosterState>().unwrap();
    assert_eq!(state.players[0], replacement);
    assert_eq!(state.metadata_updates, 2);
    assert_eq!(state.frames, 3);
}
