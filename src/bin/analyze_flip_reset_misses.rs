use anyhow::{Context, Result};
use std::path::PathBuf;
use subtr_actor::ballchasing::parse_replay_file;
use subtr_actor::{
    Collector, DodgeRefreshedEvent, FlipResetEvent, FlipResetTracker, ReplayProcessor, TimeAdvance,
};

#[derive(Debug, Clone)]
struct TouchCandidate {
    time: f32,
    frame: usize,
    player: Option<subtr_actor::PlayerId>,
    player_name: String,
    is_team_0: bool,
    closest_approach_distance: Option<f32>,
    strict_event: Option<FlipResetEvent>,
}

#[derive(Debug, Clone)]
struct ExactCandidate {
    event: DodgeRefreshedEvent,
    player_name: String,
    player_ball_distance: Option<f32>,
}

#[derive(Default)]
struct MissScanner {
    tracker: FlipResetTracker,
    touch_candidates: Vec<TouchCandidate>,
    exact_events: Vec<ExactCandidate>,
}

impl MissScanner {
    fn new() -> Self {
        Self::default()
    }
}

impl Collector for MissScanner {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
        _current_time: f32,
    ) -> subtr_actor::SubtrActorResult<TimeAdvance> {
        self.tracker.on_frame(processor, frame, frame_number)?;
        let ball_body = processor.get_ball_rigid_body().ok().copied();
        for event in processor.current_frame_dodge_refreshed_events() {
            let player_name = processor
                .get_player_name(&event.player)
                .unwrap_or_else(|_| format!("{:?}", event.player));
            let player_ball_distance = ball_body.and_then(|ball_body| {
                let player_body = processor
                    .get_player_rigid_body(&event.player)
                    .ok()
                    .copied()?;
                Some(
                    (subtr_actor::vec_to_glam(&ball_body.location)
                        - subtr_actor::vec_to_glam(&player_body.location))
                    .length(),
                )
            });
            self.exact_events.push(ExactCandidate {
                event: event.clone(),
                player_name,
                player_ball_distance,
            });
        }
        for touch_event in processor.current_frame_touch_events() {
            let strict_event =
                self.tracker
                    .current_frame_flip_reset_events()
                    .iter()
                    .find(|event| {
                        (event.time - touch_event.time).abs() < 0.0001
                            && touch_event.player.as_ref() == Some(&event.player)
                    });
            self.touch_candidates.push(TouchCandidate {
                time: touch_event.time,
                frame: touch_event.frame,
                player: touch_event.player.clone(),
                player_name: touch_event
                    .player
                    .as_ref()
                    .and_then(|player| processor.get_player_name(player).ok())
                    .unwrap_or_else(|| "<unknown>".to_owned()),
                is_team_0: touch_event.team_is_team_0,
                closest_approach_distance: touch_event.closest_approach_distance,
                strict_event: strict_event.cloned(),
            });
        }
        Ok(TimeAdvance::NextFrame)
    }
}

fn main() -> Result<()> {
    let replay_path = PathBuf::from(
        std::env::args()
            .nth(1)
            .context("Usage: cargo run --bin analyze_flip_reset_misses -- <replay-path>")?,
    );
    let replay = parse_replay_file(&replay_path)?;
    let mut scanner = MissScanner::new();
    let mut processor = ReplayProcessor::new(&replay)
        .map_err(|error| anyhow::Error::new(error.variant))
        .context("Failed to initialize replay processor")?;
    processor
        .process(&mut scanner)
        .map_err(|error| anyhow::Error::new(error.variant))
        .context("Failed to process replay")?;

    let player_names: Vec<(subtr_actor::PlayerId, String)> = processor
        .iter_player_ids_in_order()
        .map(|player| {
            (
                player.clone(),
                processor
                    .get_player_name(player)
                    .unwrap_or_else(|_| format!("{:?}", player)),
            )
        })
        .collect();

    println!("players:");
    for (player_id, name) in &player_names {
        println!("  {} => {:?}", name, player_id);
    }
    println!(
        "exact={} strict={}",
        scanner.exact_events.len(),
        scanner.tracker.flip_reset_events().len()
    );

    for exact_candidate in &scanner.exact_events {
        let exact_event = &exact_candidate.event;
        let matched = scanner
            .tracker
            .flip_reset_events()
            .iter()
            .any(|heuristic_event| {
                heuristic_event.player == exact_event.player
                    && (heuristic_event.time - exact_event.time) >= -0.20
                    && (heuristic_event.time - exact_event.time) <= 0.05
            });
        if matched {
            continue;
        }

        println!(
            "missed exact t={:.2} frame={} player={} counter={} ball_dist={:?}",
            exact_event.time,
            exact_event.frame,
            exact_candidate.player_name,
            exact_event.counter_value,
            exact_candidate.player_ball_distance
        );
        let mut same_player_touches: Vec<_> = scanner
            .touch_candidates
            .iter()
            .filter(|candidate| {
                candidate
                    .player
                    .as_ref()
                    .map(|player| player == &exact_event.player)
                    .unwrap_or(false)
            })
            .collect();
        same_player_touches.sort_by(|left, right| {
            (left.time - exact_event.time)
                .abs()
                .partial_cmp(&(right.time - exact_event.time).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let nearby_touches: Vec<_> = same_player_touches
            .iter()
            .copied()
            .filter(|candidate| (candidate.time - exact_event.time).abs() <= 0.50)
            .collect();
        if nearby_touches.is_empty() {
            println!("  no same-player touches within 0.50s");
            for candidate in same_player_touches.into_iter().take(3) {
                let strict = candidate
                    .strict_event
                    .as_ref()
                    .map(|event| format!("yes conf={:.3}", event.confidence))
                    .unwrap_or_else(|| "no".to_owned());
                println!(
                    "  nearest touch t={:.2} dt={:+.3}s frame={} player={} team={} dist={:?} strict={}",
                    candidate.time,
                    candidate.time - exact_event.time,
                    candidate.frame,
                    candidate.player_name,
                    if candidate.is_team_0 { 0 } else { 1 },
                    candidate.closest_approach_distance,
                    strict
                );
            }
            continue;
        }
        for candidate in nearby_touches.into_iter().take(5) {
            let strict = candidate
                .strict_event
                .as_ref()
                .map(|event| format!("yes conf={:.3}", event.confidence))
                .unwrap_or_else(|| "no".to_owned());
            println!(
                "  touch t={:.2} dt={:+.3}s frame={} player={} team={} dist={:?} strict={}",
                candidate.time,
                candidate.time - exact_event.time,
                candidate.frame,
                candidate.player_name,
                if candidate.is_team_0 { 0 } else { 1 },
                candidate.closest_approach_distance,
                strict
            );
        }
    }

    Ok(())
}
