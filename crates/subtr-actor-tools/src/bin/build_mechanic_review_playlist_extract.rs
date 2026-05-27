use super::candidate::{include_candidate, MechanicCandidate};
use super::config::Config;
use super::extract_ceiling::push_ceiling_shot_candidates;
use super::extract_flick::{push_flick_candidates, push_musty_flick_candidates};
use super::extract_flip_reset::push_flip_reset_candidates;
use super::extract_movement::{
    push_half_flip_candidates, push_speed_flip_candidates, push_wavedash_candidates,
};
use super::extract_touch::{
    push_air_dribble_candidates, push_double_tap_candidates, push_one_timer_candidates,
};

pub(crate) fn extract_candidates(
    replay: &boxcars::Replay,
    graph: &subtr_actor::stats::analysis_graph::AnalysisGraph,
    mechanics: &[&str],
    config: &Config,
) -> anyhow::Result<Vec<MechanicCandidate>> {
    let mut candidates = Vec::new();

    for mechanic in mechanics {
        match *mechanic {
            "flick" => {
                push_flick_candidates(graph, &mut candidates);
            }
            "musty_flick" => {
                push_musty_flick_candidates(graph, &mut candidates);
            }
            "one_timer" => {
                push_one_timer_candidates(graph, &mut candidates);
            }
            "air_dribble" => {
                push_air_dribble_candidates(graph, &mut candidates);
            }
            "flip_reset" => {
                push_flip_reset_candidates(replay, graph, &mut candidates)?;
            }
            "ceiling_shot" => {
                push_ceiling_shot_candidates(graph, &mut candidates);
            }
            "double_tap" => {
                push_double_tap_candidates(graph, &mut candidates);
            }
            "speed_flip" => {
                push_speed_flip_candidates(graph, &mut candidates);
            }
            "half_flip" => {
                push_half_flip_candidates(graph, &mut candidates);
            }
            "wavedash" => {
                push_wavedash_candidates(graph, &mut candidates);
            }
            _ => {}
        }
    }

    candidates.retain(|candidate| include_candidate(candidate, config));
    candidates.sort_by(|left, right| {
        left.start_time
            .total_cmp(&right.start_time)
            .then_with(|| left.mechanic.cmp(right.mechanic))
            .then_with(|| left.event_frame.cmp(&right.event_frame))
    });
    Ok(candidates)
}
