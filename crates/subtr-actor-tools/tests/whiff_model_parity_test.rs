//! The Rust shadow whiff model must reproduce the Python trainer exactly:
//! same causal candidates, same feature vectors, same class probabilities.

use std::collections::BTreeMap;

use serde::Deserialize;
use subtr_actor_tools::whiff_model::features::{NumericTelemetry, replay_candidates};
use subtr_actor_tools::whiff_model::model::WhiffModel;
use subtr_actor_tools::whiff_model::telemetry::collect_replay_telemetry;

#[derive(Deserialize)]
struct ParityCandidate {
    player_id: String,
    anchor_frame: usize,
    features: Vec<Option<f64>>,
    proba: Vec<f64>,
}

#[derive(Deserialize)]
struct ParityFixture {
    replay_id: String,
    candidates: Vec<ParityCandidate>,
}

fn asset(path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn shadow_model_matches_python_reference() {
    let model = WhiffModel::from_json(
        &std::fs::read_to_string(asset("assets/whiff_model/model.json")).unwrap(),
    )
    .unwrap();
    let fixture: ParityFixture = serde_json::from_str(
        &std::fs::read_to_string(asset("assets/whiff_model/parity.json")).unwrap(),
    )
    .unwrap();
    let data = std::fs::read(asset(&format!("../../assets/{}.replay", fixture.replay_id))).unwrap();
    let replay = boxcars::ParserBuilder::new(&data[..])
        .must_parse_network_data()
        .parse()
        .unwrap();
    let collected = collect_replay_telemetry(&replay).unwrap();
    let numeric = NumericTelemetry::from_telemetry(&collected.telemetry);
    let rust: BTreeMap<(String, usize), Vec<f64>> = replay_candidates(&numeric)
        .into_iter()
        .map(|c| {
            (
                (
                    numeric.players[c.player_index].id.clone(),
                    collected.telemetry.frames.frame[c.anchor_index],
                ),
                c.features,
            )
        })
        .collect();

    assert_eq!(rust.len(), fixture.candidates.len(), "candidate count");
    for expected in &fixture.candidates {
        let features = rust
            .get(&(expected.player_id.clone(), expected.anchor_frame))
            .unwrap_or_else(|| panic!("missing candidate at frame {}", expected.anchor_frame));
        for (got, want) in features.iter().zip(&expected.features) {
            match want {
                Some(want) => assert_eq!(got, want, "frame {}", expected.anchor_frame),
                None => assert!(!got.is_finite(), "frame {}", expected.anchor_frame),
            }
        }
        for (got, want) in model.predict_proba(features).iter().zip(&expected.proba) {
            assert!(
                (got - want).abs() < 1e-12,
                "frame {}",
                expected.anchor_frame
            );
        }
    }
}
