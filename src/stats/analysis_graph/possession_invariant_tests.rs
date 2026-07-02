use super::*;
use crate::{PlayerPossessionCalculator, PossessionCalculator};
use std::path::Path;

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

/// Team control (the strict `possession` stream) must cover at least the summed
/// per-player possession of its players: a player can only possess the ball
/// while their team controls it. This broke when the strict resolver's window
/// was tighter than the eager tracker's loose-ball timeout (holds with touches
/// 1.5–3s apart earned zero team credit) and when player spans credited loose
/// tails after the final touch that the resolver sent to neutral.
#[test]
#[ignore = "real-replay sweep is slow; run explicitly when changing possession resolution"]
fn team_control_covers_summed_player_possession_on_real_replays() {
    let fixtures = [
        "assets/post-eac-ranked-duel-2026-04-28-a.replay",
        "assets/post-eac-ranked-duel-2026-04-28-b.replay",
        "assets/post-eac-ranked-doubles-2026-04-28.replay",
        "assets/post-eac-ranked-standard-2026-04-28.replay",
        "assets/recent-ranked-doubles-2026-03-10.replay",
        "assets/recent-ranked-standard-2026-03-10-a.replay",
        "assets/recent-ranked-standard-2026-03-10-b.replay",
        "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
    ];
    for fixture in fixtures {
        let replay = parse_replay(fixture);
        let graph = collect_analysis_graph_for_replay(&replay, graph_with_all_analysis_nodes())
            .expect("graph should evaluate");
        let possession = graph
            .state::<PossessionCalculator>()
            .expect("possession state");
        let player_possession = graph
            .state::<PlayerPossessionCalculator>()
            .expect("player possession state");

        let mut team = [0.0f64; 2];
        for event in possession.events() {
            match (event.active, event.possession_state.as_str()) {
                (true, "team_zero") => team[0] += event.duration as f64,
                (true, "team_one") => team[1] += event.duration as f64,
                _ => {}
            }
        }
        let mut players = [0.0f64; 2];
        for event in player_possession.events() {
            players[if event.is_team_0 { 0 } else { 1 }] += event.duration as f64;
        }
        println!(
            "{fixture}: control team0={:.1}s team1={:.1}s, player sums team0={:.1}s team1={:.1}s",
            team[0], team[1], players[0], players[1]
        );
        for side in 0..2 {
            // Small tolerance: player spans open one frame before their first
            // touch (crediting that frame's dt), which the resolver does not.
            assert!(
                players[side] <= team[side] + 1.0,
                "{fixture}: team{side} control {:.1}s < summed player possession {:.1}s",
                team[side],
                players[side],
            );
        }
    }
}
