//! Audit WallAerial detection for a replay.
//!
//! Dumps every WallAerial event with its setup/takeoff/touch geometry and
//! timing so we can see why a given aerial was (mis)classified as a wall aerial.

use std::collections::HashMap;

use anyhow::Context;
use clap::Parser;
use subtr_actor::{
    EventPayload, PlayerFrame, PlayerId, ReplayDataCollector, ReplayMeta, StatsTimelineCollector,
};

#[derive(Debug, Parser)]
#[command(about = "Audit WallAerial detection for a replay.")]
struct Args {
    replay: String,
    /// Optionally dump a player's x/z trajectory across this inclusive frame range, e.g. 770-820.
    #[arg(long)]
    traj: Option<String>,
    /// Player name substring to match for --traj.
    #[arg(long)]
    player: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let Args {
        replay,
        traj,
        player,
    } = Args::parse();
    let parsed = parse_replay(&replay)?;

    if let Some(range) = traj.as_deref() {
        return dump_trajectory(&parsed, range, player.as_deref());
    }
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&parsed)
        .map_err(|error| anyhow::anyhow!("failed to build stats timeline: {error:?}"))?;
    let names = player_name_map(&timeline.replay_meta);
    let clock_zero = timeline
        .frames
        .first()
        .map(|frame| frame.time)
        .unwrap_or(0.0);
    let clock = |t: f32| format_clock(t - clock_zero);

    let mut idx = 0;
    println!("==== WALL AERIAL events ====");
    for event in &timeline.events.events {
        if let EventPayload::WallAerial(wa) = &event.payload {
            idx += 1;
            println!(
                "wall_aerial {idx}: clk {} abs {:.3} frame {} player={} team0={} wall={:?} conf={:.3}",
                clock(wa.time),
                wa.time,
                wa.frame,
                player_name(&names, &wa.player),
                wa.is_team_0,
                wa.wall,
                wa.confidence,
            );
            println!(
                "    setup: start clk {} abs {:.3} dur={:.3}s",
                clock(wa.setup_start_time),
                wa.setup_start_time,
                wa.setup_duration,
            );
            println!(
                "    wall_contact: clk {} abs {:.3} frame {} pos=[{:.0},{:.0},{:.0}]",
                clock(wa.wall_contact_time),
                wa.wall_contact_time,
                wa.wall_contact_frame,
                wa.wall_contact_position[0],
                wa.wall_contact_position[1],
                wa.wall_contact_position[2],
            );
            println!(
                "    takeoff: clk {} abs {:.3} frame {} pos=[{:.0},{:.0},{:.0}] time_since_takeoff={:.3}",
                clock(wa.takeoff_time),
                wa.takeoff_time,
                wa.takeoff_frame,
                wa.takeoff_position[0],
                wa.takeoff_position[1],
                wa.takeoff_position[2],
                wa.time_since_takeoff,
            );
            println!(
                "    touch: player_pos=[{:.0},{:.0},{:.0}] ball_pos=[{:.0},{:.0},{:.0}]",
                wa.player_position[0],
                wa.player_position[1],
                wa.player_position[2],
                wa.ball_position[0],
                wa.ball_position[1],
                wa.ball_position[2],
            );
            println!(
                "    ball: speed={:.0} speed_change={:.0} goal_alignment={:.3}",
                wa.ball_speed, wa.ball_speed_change, wa.goal_alignment,
            );
        }
    }
    if idx == 0 {
        println!("(no wall aerial events)");
    }

    Ok(())
}

fn dump_trajectory(
    parsed: &boxcars::Replay,
    range: &str,
    player_substr: Option<&str>,
) -> anyhow::Result<()> {
    const SIDE_WALL_CONTACT_ABS_X: f32 = 3600.0;
    const SETUP_SIDE_WALL_START_ABS_X: f32 = 3200.0;

    let (lo, hi) = range.split_once('-').context("range must be LO-HI")?;
    let lo: usize = lo.trim().parse().context("bad LO")?;
    let hi: usize = hi.trim().parse().context("bad HI")?;

    let data = ReplayDataCollector::new()
        .get_replay_data(parsed)
        .map_err(|e| anyhow::anyhow!("collect replay data: {e:?}"))?;
    let names = player_name_map(&data.meta);
    let ball_frames = data.frame_data.ball_data.frames();
    let ball_at = |f: usize| -> Option<glam::Vec3> {
        match ball_frames.get(f) {
            Some(subtr_actor::BallFrame::Data { rigid_body }) => {
                let l = rigid_body.location;
                Some(glam::Vec3::new(l.x, l.y, l.z))
            }
            _ => None,
        }
    };

    for (player_id, player_data) in &data.frame_data.players {
        let name = player_name(&names, player_id);
        if let Some(sub) = player_substr {
            if !name.to_lowercase().contains(&sub.to_lowercase()) {
                continue;
            }
        }
        println!("== trajectory for {name} (frames {lo}-{hi}) ==");
        let frames = player_data.frames();
        for f in lo..=hi.min(frames.len().saturating_sub(1)) {
            if let Some(PlayerFrame::Data { rigid_body, .. }) = frames.get(f) {
                let loc = rigid_body.location;
                let side_on = loc.x.abs() >= SIDE_WALL_CONTACT_ABS_X;
                let back_on = loc.y.abs() >= 5000.0 && loc.x.abs() > 900.0;
                let on_wall = (side_on || back_on) && loc.z >= 120.0;
                let side_setup = loc.x.abs() >= SETUP_SIDE_WALL_START_ABS_X;
                let back_setup = loc.y.abs() >= 4600.0 && loc.x.abs() > 900.0;
                let in_setup = (side_setup || back_setup) && loc.z >= 120.0;
                let gv = glam::Vec3::new(loc.x, loc.y, loc.z);
                let (ball_dist, control) = match ball_at(f) {
                    Some(b) => {
                        let d = gv.distance(b);
                        (format!("{d:.0}"), d <= 380.0)
                    }
                    None => ("?".to_string(), false),
                };
                println!(
                    "  frame {f}: pos=[{:.0},{:.0},{:.0}] |x|={:.0} |y|={:.0} on_wall={} setup_band={} ball_dist={ball_dist} ctrl={control} ON_WALL_CTRL={}",
                    loc.x,
                    loc.y,
                    loc.z,
                    loc.x.abs(),
                    loc.y.abs(),
                    on_wall,
                    in_setup,
                    on_wall && control,
                );
            } else {
                println!("  frame {f}: <no data>");
            }
        }
    }
    Ok(())
}

fn format_clock(seconds: f32) -> String {
    let seconds = seconds.max(0.0);
    let minutes = (seconds / 60.0).floor() as u32;
    let rem = seconds - (minutes as f32) * 60.0;
    format!("{minutes}:{rem:04.1}")
}

fn parse_replay(path: &str) -> anyhow::Result<boxcars::Replay> {
    let data = std::fs::read(path).with_context(|| format!("reading replay {path}"))?;
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .with_context(|| format!("parsing replay {path}"))
}

fn player_name_map(meta: &ReplayMeta) -> HashMap<String, String> {
    meta.team_zero
        .iter()
        .chain(meta.team_one.iter())
        .map(|player| (player_id_string(&player.remote_id), player.name.clone()))
        .collect()
}

fn player_name(names: &HashMap<String, String>, player_id: &PlayerId) -> String {
    names
        .get(&player_id_string(player_id))
        .cloned()
        .unwrap_or_else(|| player_id_string(player_id))
}

fn player_id_string(player_id: &PlayerId) -> String {
    format!("{player_id:?}")
}
