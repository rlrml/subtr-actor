use clap::Parser;
use subtr_actor::StatsTimelineCollector;

#[path = "speed_flip_kickoff_audit_helpers.rs"]
mod helpers;
#[path = "speed_flip_kickoff_audit_types.rs"]
mod types;

use helpers::{
    detected_speed_flips, front_players, kickoff_start_indices, parse_replay, player_name_map,
};
use types::{Args, Audit, KickoffAudit, ReplayAudit};

const DETECTION_WINDOW_SECONDS: f32 = 1.5;

fn main() -> anyhow::Result<()> {
    let Args { paths } = Args::parse();

    let mut replays = Vec::new();
    for path in paths {
        replays.push(audit_replay(&path)?);
    }

    println!("{}", serde_json::to_string_pretty(&Audit { replays })?);
    Ok(())
}

#[allow(deprecated)]
fn audit_replay(path: &str) -> anyhow::Result<ReplayAudit> {
    let replay = parse_replay(path)?;
    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .map_err(|error| anyhow::anyhow!("failed to build stats timeline for {path}: {error:?}"))?;
    let player_names = player_name_map(&timeline.replay_meta);
    let kickoff_start_indices = kickoff_start_indices(&timeline.frames);
    let mut kickoffs = Vec::new();

    for (index, frame_index) in kickoff_start_indices.into_iter().enumerate() {
        let frame = &timeline.frames[frame_index];
        let start_time = frame.time;
        let end_time = start_time + DETECTION_WINDOW_SECONDS;
        let blue_detected = detected_speed_flips(
            &timeline.events.speed_flip,
            true,
            start_time,
            end_time,
            &player_names,
        );
        let orange_detected = detected_speed_flips(
            &timeline.events.speed_flip,
            false,
            start_time,
            end_time,
            &player_names,
        );

        kickoffs.push(KickoffAudit {
            index: index + 1,
            start_time,
            start_frame: frame.frame_number,
            blue_front_players: front_players(frame, true, &player_names),
            orange_front_players: front_players(frame, false, &player_names),
            blue_detected,
            orange_detected,
        });
    }

    let team_kickoff_opportunities = kickoffs
        .iter()
        .map(|kickoff| {
            usize::from(!kickoff.blue_front_players.is_empty())
                + usize::from(!kickoff.orange_front_players.is_empty())
        })
        .sum();
    let detected_team_kickoffs = kickoffs
        .iter()
        .map(|kickoff| {
            usize::from(!kickoff.blue_detected.is_empty())
                + usize::from(!kickoff.orange_detected.is_empty())
        })
        .sum();

    Ok(ReplayAudit {
        path: path.to_owned(),
        kickoff_count: kickoffs.len(),
        team_kickoff_opportunities,
        detected_team_kickoffs,
        speed_flip_event_count: timeline.events.speed_flip.len(),
        kickoffs,
    })
}
