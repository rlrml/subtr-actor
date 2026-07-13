use super::*;
use crate::collector::ndarray::traits::{
    dynamic_analysis_feature_adder, dynamic_analysis_player_feature_adder,
};
use crate::stats::analysis_graph::{AnalysisNode, AnalysisStateContext};
use crate::{Collector, FrameRateDecorator};
use std::path::Path;

const NDARRAY_ANALYSIS_FIXTURE: &str = "assets/post-eac-ranked-doubles-2026-04-28.replay";
const NDARRAY_DUEL_FIXTURE: &str = "assets/post-eac-ranked-duel-2026-04-28-a.replay";

#[derive(Default)]
struct SharedAnalysisState {
    evaluations: usize,
}

#[derive(Default)]
struct SharedAnalysisNode {
    state: SharedAnalysisState,
}

impl AnalysisNode for SharedAnalysisNode {
    type State = SharedAnalysisState;

    fn name(&self) -> &'static str {
        "ndarray_test_shared_analysis"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![AnalysisDependency::required::<FrameInput>()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let _frame_input = ctx.get::<FrameInput>()?;
        self.state.evaluations += 1;
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

fn boxed_shared_analysis_node() -> Box<dyn crate::stats::analysis_graph::AnalysisNodeDyn> {
    Box::<SharedAnalysisNode>::default()
}

fn shared_analysis_dependency() -> Vec<AnalysisDependency> {
    vec![AnalysisDependency::with_default::<SharedAnalysisState>(
        boxed_shared_analysis_node,
    )]
}

build_analysis_global_feature_adder!(
    MacroSharedAnalysisCount,
    |_self_: &MacroSharedAnalysisCount<F>| shared_analysis_dependency(),
    |_self_: &MacroSharedAnalysisCount<F>,
     context: &AnalysisFeatureContext<'_>,
     _processor: &dyn ProcessorView,
     _frame: &boxcars::Frame,
     _frame_count: usize,
     _current_time: f32| {
        let state = context.state::<SharedAnalysisState>()?;
        convert_all_floats!(state.evaluations as f32)
    },
    "macro shared analysis count",
);

fn parse_replay(path: &str) -> boxcars::Replay {
    let replay_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let data = std::fs::read(&replay_path)
        .unwrap_or_else(|_| panic!("Failed to read replay file: {}", replay_path.display()));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {}", replay_path.display()))
}

#[test]
fn analysis_feature_adders_share_one_graph_evaluation_per_ndarray_row() {
    static DYNAMIC_HEADERS: [&str; 1] = ["dynamic shared analysis count"];
    static PLAYER_HEADERS: [&str; 1] = ["player shared analysis count"];

    let replay = parse_replay(NDARRAY_ANALYSIS_FIXTURE);
    let mut collector = NDArrayCollector::<f32>::new(
        vec![
            NDArrayFeatureAdder::analysis(MacroSharedAnalysisCount::arc_new()),
            NDArrayFeatureAdder::analysis(dynamic_analysis_feature_adder(
                &DYNAMIC_HEADERS,
                shared_analysis_dependency(),
                |context, _processor, _frame, _frame_count, _current_time| {
                    let state = context.state::<SharedAnalysisState>()?;
                    Ok([state.evaluations as f32])
                },
            )),
        ],
        vec![NDArrayPlayerFeatureAdder::analysis(
            dynamic_analysis_player_feature_adder(
                &PLAYER_HEADERS,
                shared_analysis_dependency(),
                |context, _player_id, _processor, _frame, _frame_count, _current_time| {
                    let state = context.state::<SharedAnalysisState>()?;
                    Ok([state.evaluations as f32])
                },
            ),
        )],
    );
    FrameRateDecorator::new_from_fps(30.0, &mut collector)
        .process_replay(&replay)
        .expect("collector should process replay");

    let (meta, ndarray) = collector
        .get_meta_and_ndarray()
        .expect("collector should produce ndarray");

    assert_eq!(
        meta.column_headers.global_headers,
        vec![
            "macro shared analysis count".to_owned(),
            "dynamic shared analysis count".to_owned()
        ]
    );
    assert_eq!(
        meta.column_headers.player_headers,
        vec!["player shared analysis count".to_owned()]
    );
    assert!(ndarray.nrows() > 5);
    assert_eq!(ndarray.ncols(), 2 + meta.replay_meta.player_count());

    for (row_index, row) in ndarray.outer_iter().take(5).enumerate() {
        let expected = (row_index + 1) as f32;
        assert_eq!(row[0], expected);
        assert_eq!(row[1], expected);
        for player_column in 2..row.len() {
            assert_eq!(row[player_column], expected);
        }
    }
}

#[test]
fn string_feature_names_can_create_analysis_backed_touch_adders() {
    let replay = parse_replay(NDARRAY_ANALYSIS_FIXTURE);
    let mut collector = NDArrayCollector::<f32>::from_strings(&[], &["PlayerEvent:touch"])
        .expect("analysis-backed feature names should resolve");
    FrameRateDecorator::new_from_fps(30.0, &mut collector)
        .process_replay(&replay)
        .expect("collector should process replay");

    let (meta, ndarray) = collector
        .get_meta_and_ndarray()
        .expect("collector should produce ndarray");

    assert_eq!(meta.column_headers.global_headers, Vec::<String>::new());
    assert_eq!(
        meta.column_headers.player_headers,
        vec!["analysis touch event".to_owned()]
    );
    assert!(ndarray.nrows() > 5);
    assert_eq!(ndarray.ncols(), meta.replay_meta.player_count());

    let mut touch_event_count = 0usize;
    for value in ndarray.iter().copied() {
        assert!(
            value == 0.0 || value == 1.0,
            "player touch event feature should be binary"
        );
        if value == 1.0 {
            touch_event_count += 1;
        }
    }
    assert!(
        touch_event_count > 0,
        "fixture should produce at least one string-created player touch event"
    );
}

#[test]
fn threat_training_rows_use_ndarray_features_with_streaming_model_parity() {
    let replay = parse_replay(NDARRAY_ANALYSIS_FIXTURE);
    let collector = NDArrayCollector::<f32>::from_strings(
        &["CurrentTime", "ThreatFeatures", "ThreatModelValues"],
        &[],
    )
    .expect("threat ndarray features should be registered")
    .with_analysis_frame_filter(LiveThreatSampleFilter::new(0.25))
    .process_replay(&replay)
    .expect("threat ndarray should process replay");

    assert!(
        collector
            .analysis_state::<ExpectedGoalsCalculator>()
            .is_some(),
        "requesting model values should wire the opt-in expected-goals node"
    );
    let (meta, matrix) = collector
        .get_meta_and_ndarray()
        .expect("threat ndarray should materialize");
    assert!(matrix.nrows() > 5);
    assert_eq!(matrix.ncols(), 1 + 2 * THREAT_FEATURE_COUNT + 2);
    for (team_index, team_name) in ["team_zero", "team_one"].into_iter().enumerate() {
        for (feature_index, feature_name) in ThreatFeatures::FEATURE_NAMES.iter().enumerate() {
            assert_eq!(
                meta.column_headers.global_headers
                    [1 + team_index * THREAT_FEATURE_COUNT + feature_index],
                format!("{team_name}_threat_{feature_name}")
            );
        }
    }

    for row in matrix.rows().into_iter().take(8) {
        for team_index in 0..2 {
            let start = 1 + team_index * THREAT_FEATURE_COUNT;
            let values: [f32; THREAT_FEATURE_COUNT] =
                std::array::from_fn(|index| row[start + index]);
            let model_value = row[1 + 2 * THREAT_FEATURE_COUNT + team_index];
            assert!((threat_value_from_array(&values) - model_value).abs() < 1e-6);
        }
    }
}

#[test]
fn threat_ndarray_rejects_non_doubles_replays_by_producing_no_rows() {
    let replay = parse_replay(NDARRAY_DUEL_FIXTURE);
    let collector = NDArrayCollector::<f32>::from_strings(
        &["CurrentTime", "ThreatFeatures", "ThreatModelValues"],
        &[],
    )
    .expect("threat ndarray features should be registered")
    .with_analysis_frame_filter(LiveThreatSampleFilter::new(0.25))
    .process_replay(&replay)
    .expect("non-doubles replay traversal itself should remain valid");
    let (_, matrix) = collector
        .get_meta_and_ndarray()
        .expect("empty threat matrix should still have a valid schema");
    assert_eq!(matrix.nrows(), 0);
    assert_eq!(matrix.ncols(), 1 + 2 * THREAT_FEATURE_COUNT + 2);
}

#[test]
fn player_event_names_create_registered_analysis_indicators() {
    let event_names = [
        "PlayerEvent:touch",
        "PlayerEvent:center",
        "PlayerEvent:double_tap",
        "PlayerEvent:one_timer",
        "PlayerEvent:wall_aerial",
        "PlayerEvent:wall_aerial_shot",
        "PlayerEvent:ceiling_shot",
        "PlayerEvent:flick",
        "PlayerEvent:dodge_reset",
        "PlayerEvent:flip_reset_dodge",
        "PlayerEvent:half_flip",
        "PlayerEvent:half_volley",
        "PlayerEvent:wavedash",
        "PlayerEvent:whiff",
        "PlayerEvent:speed_flip",
        "PlayerEvent:dodge",
        "PlayerEvent:powerslide",
        "PlayerEvent:ball_carry",
        "PlayerEvent:boost_pickup",
        "PlayerEvent:boost_respawn",
        "PlayerEvent:bump",
        "PlayerEvent:pass",
        "PlayerEvent:rotation",
        "PlayerEvent:movement",
        "PlayerEvent:positioning",
    ];
    let collector = NDArrayCollector::<f32>::from_strings(&[], &event_names)
        .expect("registered event feature names should resolve");
    let headers = collector.get_column_headers();

    assert_eq!(headers.global_headers, Vec::<String>::new());
    assert_eq!(headers.player_headers.len(), event_names.len());
    assert_eq!(headers.player_headers[0], "analysis touch event");
    assert_eq!(
        headers.player_headers.last(),
        Some(&"analysis positioning event".to_owned())
    );
}
