mod common;

use subtr_actor::stats::analysis_graph::graph_with_builtin_analysis_nodes;
use subtr_actor::{
    Collector, FrameInput, ReplayFrameInputBuilder, ReplayProcessor, SubtrActorResult, TimeAdvance,
    TouchCandidateScoring, TouchState, car_hitbox_for_body_id, car_hitbox_for_body_name,
};

const TOUCH_STATE_FIXTURES: &[&str] = &[
    "assets/post-eac-ranked-duel-2026-04-28-a.replay",
    "assets/recent-ranked-doubles-2026-03-10.replay",
    "assets/colonelpanic8-double-tap-third-goal-2026-05-24.replay",
];

const HITBOX_FIXTURES: &[&str] = &[
    "assets/post-eac-ranked-duel-2026-04-28-a.replay",
    "assets/post-eac-ranked-duel-2026-04-28-b.replay",
    "assets/post-eac-ranked-doubles-2026-04-28.replay",
    "assets/post-eac-ranked-standard-2026-04-28.replay",
    "assets/post-eac-private-2026-04-28.replay",
    "assets/recent-ranked-doubles-2026-03-10.replay",
    "assets/recent-ranked-standard-2026-03-10-a.replay",
    "assets/recent-ranked-standard-2026-03-10-b.replay",
    "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
];

const RELAXED_TOUCH_CONTACT_GAP_THRESHOLD: f32 =
    TouchCandidateScoring::DEFAULT.relaxed_contact_gap_threshold;

#[derive(Default)]
struct TouchStateFixtureStats {
    touch_count: usize,
    player_attributed_count: usize,
    contact_gap_count: usize,
    contact_gaps: Vec<f32>,
    max_contact_gap: f32,
    out_of_range_contact_gaps: Vec<String>,
}

struct TouchStateFixtureCollector {
    graph: subtr_actor::stats::analysis_graph::AnalysisGraph,
    frame_input_builder: ReplayFrameInputBuilder,
    last_sample_time: Option<f32>,
    last_replay_meta_player_count: Option<usize>,
    stats: TouchStateFixtureStats,
}

impl TouchStateFixtureCollector {
    fn new() -> Self {
        Self {
            graph: graph_with_builtin_analysis_nodes(["touch_state"])
                .expect("touch_state analysis graph should be valid"),
            frame_input_builder: ReplayFrameInputBuilder::default(),
            last_sample_time: None,
            last_replay_meta_player_count: None,
            stats: TouchStateFixtureStats::default(),
        }
    }

    fn record_touch_state(&mut self, frame_number: usize) {
        let Some(touch_state) = self.graph.state::<TouchState>() else {
            return;
        };
        for touch in &touch_state.touch_events {
            self.stats.touch_count += 1;
            if touch.player.is_some() {
                self.stats.player_attributed_count += 1;
            }
            if let Some(gap) = touch.closest_approach_distance {
                self.stats.contact_gap_count += 1;
                self.stats.contact_gaps.push(gap);
                self.stats.max_contact_gap = self.stats.max_contact_gap.max(gap);
                if !gap.is_finite() || !(0.0..=RELAXED_TOUCH_CONTACT_GAP_THRESHOLD).contains(&gap) {
                    self.stats
                        .out_of_range_contact_gaps
                        .push(format!("frame {frame_number}: gap {gap}"));
                }
            }
        }
    }
}

impl Collector for TouchStateFixtureCollector {
    fn process_frame(
        &mut self,
        processor: &dyn subtr_actor::ProcessorView,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        let player_count = processor.player_count();
        if self.last_replay_meta_player_count != Some(player_count) {
            self.graph.on_replay_meta(&processor.get_replay_meta()?)?;
            self.last_replay_meta_player_count = Some(player_count);
        }

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let frame_input: FrameInput =
            self.frame_input_builder
                .aggregate(processor, frame_number, current_time, dt);
        self.graph.evaluate_with_state(&frame_input)?;
        self.record_touch_state(frame_number);
        self.last_sample_time = Some(current_time);

        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(
        &mut self,
        _processor: &dyn subtr_actor::ProcessorView,
    ) -> SubtrActorResult<()> {
        self.graph.finish()
    }
}

#[test]
fn fixture_player_loadout_body_ids_are_threaded_to_player_meta() {
    let mut checked_players = 0usize;
    let mut missing_body_ids = Vec::new();
    let mut unknown_body_ids = std::collections::BTreeSet::new();
    let mut resolved_body_ids = std::collections::BTreeSet::new();

    for fixture in HITBOX_FIXTURES {
        let replay = common::parse_replay(fixture);
        let mut processor =
            ReplayProcessor::new(&replay).unwrap_or_else(|error| panic!("{fixture}: {error:?}"));
        let meta = processor
            .process_and_get_replay_meta()
            .unwrap_or_else(|error| panic!("{fixture}: {error:?}"));

        for player in meta.player_order() {
            checked_players += 1;
            let Some(body_id) = player.car_body_id else {
                missing_body_ids.push(format!("{fixture}: {}", player.name));
                continue;
            };

            if let Some(body_name) = player.car_body_name.as_deref() {
                if let Some(hitbox) = car_hitbox_for_body_name(body_name) {
                    assert_eq!(
                        player.car_hitbox_family.as_deref(),
                        Some(format!("{:?}", hitbox.family).as_str()),
                        "{fixture}: {} body name {body_name}",
                        player.name
                    );
                }
            }

            match car_hitbox_for_body_id(body_id) {
                Some(hitbox) => {
                    resolved_body_ids.insert((body_id, format!("{:?}", hitbox.family)));
                    assert_eq!(
                        player.car_hitbox_family.as_deref(),
                        Some(format!("{:?}", hitbox.family).as_str()),
                        "{fixture}: {} body id {body_id}",
                        player.name
                    );
                }
                None => {
                    unknown_body_ids.insert(body_id);
                    assert_eq!(
                        player.car_hitbox_family, None,
                        "{fixture}: {} body id {body_id}",
                        player.name
                    );
                }
            }
        }
    }

    assert!(
        checked_players > 0,
        "expected to inspect at least one replay player"
    );
    assert!(
        missing_body_ids.is_empty(),
        "missing car_body_id values:\n{}",
        missing_body_ids.join("\n")
    );
    assert_eq!(
        resolved_body_ids,
        std::collections::BTreeSet::from([
            (23, "Octane".to_string()),
            (25, "Octane".to_string()),
            (403, "Dominus".to_string()),
            (4284, "Octane".to_string()),
            (4770, "Dominus".to_string()),
            (11315, "Dominus".to_string()),
        ])
    );
    assert_eq!(
        unknown_body_ids,
        std::collections::BTreeSet::new(),
        "unexpected unknown body ids"
    );
}

#[test]
fn fixture_touch_state_uses_hitbox_contact_gaps_for_replay_touches() {
    for fixture in TOUCH_STATE_FIXTURES {
        let replay = common::parse_replay(fixture);
        let collector = TouchStateFixtureCollector::new()
            .process_replay(&replay)
            .unwrap_or_else(|error| panic!("{fixture}: {error:?}"));
        let stats = collector.stats;

        assert!(
            stats.touch_count > 0,
            "{fixture}: expected touch_state touches"
        );
        assert_eq!(
            stats.player_attributed_count, stats.touch_count,
            "{fixture}: expected every emitted touch_state touch to have player attribution"
        );
        assert_eq!(
            stats.contact_gap_count, stats.touch_count,
            "{fixture}: expected every emitted touch_state touch to carry a hitbox contact gap"
        );
        assert!(
            stats.out_of_range_contact_gaps.is_empty(),
            "{fixture}: hitbox contact gaps outside relaxed threshold {RELAXED_TOUCH_CONTACT_GAP_THRESHOLD}:\n{}",
            stats.out_of_range_contact_gaps.join("\n")
        );

        let mut sorted_gaps = stats.contact_gaps;
        sorted_gaps.sort_by(f32::total_cmp);
        let p75_index = ((sorted_gaps.len() - 1) * 3) / 4;
        let p75_gap = sorted_gaps[p75_index];
        assert!(
            p75_gap <= TouchCandidateScoring::DEFAULT.strict_contact_gap_threshold,
            "{fixture}: expected p75 hitbox contact gap {p75_gap} to stay within strict threshold {} (max {})",
            TouchCandidateScoring::DEFAULT.strict_contact_gap_threshold,
            stats.max_contact_gap
        );
    }
}
