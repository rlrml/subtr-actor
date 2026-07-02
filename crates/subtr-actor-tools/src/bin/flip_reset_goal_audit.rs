//! Audit why goals do/do not receive a FlipResetGoal tag.
//!
//! Dumps every goal (GoalContext) with its tags, every DodgeReset (on_ball,
//! used, outcome), and every confirmed FlipReset event, with viewer-clock and
//! absolute times, so we can see which gate a given goal fails.

use std::collections::HashMap;

use anyhow::Context;
use clap::Parser;
use subtr_actor::{
    BallCarryKind, EventPayload, GoalTag, PlayerId, ReplayMeta, StatsTimelineCollector,
};

#[derive(Debug, Parser)]
#[command(about = "Audit FlipResetGoal tagging for a replay.")]
struct Args {
    replay: String,
}

fn main() -> anyhow::Result<()> {
    let Args { replay } = Args::parse();
    let parsed = parse_replay(&replay)?;
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

    let mut goal_idx = 0;
    println!("==== GOALS (GoalContext events) ====");
    for event in &timeline.events.events {
        if let EventPayload::GoalContext(goal) = &event.payload {
            goal_idx += 1;
            let scorer = goal
                .scorer
                .as_ref()
                .map(|p| player_name(&names, p))
                .unwrap_or_else(|| "<none>".into());
            let last_touch = goal
                .scorer_last_touch
                .as_ref()
                .map(|t| format!("{} (clk {})", t.time, clock(t.time)))
                .unwrap_or_else(|| "<none>".into());
            let tag_names: Vec<String> = goal.tags.iter().map(tag_name).collect();
            println!(
                "goal {goal_idx}: clk {} abs {:.3} frame {} team0={} scorer={scorer} last_touch={last_touch}",
                clock(goal.time),
                goal.time,
                goal.frame,
                goal.scoring_team_is_team_0,
            );
            println!("         tags: {}", tag_names.join(", "));
            let has_flip = goal
                .tags
                .iter()
                .any(|t| matches!(t, GoalTag::FlipResetGoal(_)));
            let has_air_dribble = goal
                .tags
                .iter()
                .any(|t| matches!(t, GoalTag::AirDribbleGoal(_)));
            println!("         FlipResetGoal: {has_flip}");
            println!("         AirDribbleGoal: {has_air_dribble}");
        }
    }

    println!("\n==== AIR DRIBBLE events ====");
    for event in &timeline.events.events {
        if let EventPayload::BallCarry(carry) = &event.payload {
            if carry.kind != BallCarryKind::AirDribble {
                continue;
            }
            println!(
                "clk {} abs {:.3}-{:.3} frames {}-{} player={} team0={} touches={} air_touches={} origin={:?}",
                clock(carry.start_time),
                carry.start_time,
                carry.end_time,
                carry.start_frame,
                carry.end_frame,
                player_name(&names, &carry.player_id),
                carry.is_team_0,
                carry.touch_count,
                carry.air_touch_count,
                carry.air_dribble_origin,
            );
        }
    }

    println!("\n==== TOUCH events ====");
    for event in &timeline.events.events {
        if let EventPayload::Touch(touch) = &event.payload {
            let kind = touch.tag("kind").unwrap_or("<none>");
            let surface = touch.tag("surface").unwrap_or("<none>");
            println!(
                "clk {} abs {:.3} frame {} player={} team0={} kind={} surface={} player_z={} ball_z={} speed_change={:.1}",
                clock(touch.time),
                touch.time,
                touch.frame,
                player_name(&names, &touch.player),
                touch.is_team_0,
                kind,
                surface,
                touch
                    .player_position
                    .map(|position| format!("{:.0}", position[2]))
                    .unwrap_or_else(|| "<none>".to_owned()),
                touch
                    .ball_position
                    .map(|position| format!("{:.0}", position[2]))
                    .unwrap_or_else(|| "<none>".to_owned()),
                touch.ball_speed_change,
            );
        }
    }

    println!("\n==== DODGE RESET events (on_ball=flip reset candidate) ====");
    for event in &timeline.events.events {
        if let EventPayload::DodgeReset(dr) = &event.payload {
            println!(
                "clk {} abs {:.3} frame {} player={} team0={} on_ball={} used={} outcome={:?} time_to_use={:?}",
                clock(dr.time),
                dr.time,
                dr.frame,
                player_name(&names, &dr.player),
                dr.is_team_0,
                dr.on_ball,
                dr.used,
                dr.outcome,
                dr.time_to_use,
            );
        }
    }

    println!("\n==== CONFIRMED FLIP RESET events ====");
    for event in &timeline.events.events {
        if let EventPayload::FlipReset(fr) = &event.payload {
            println!(
                "clk {} abs {:.3} frame {} player={} team0={} reset_time={:.3} (clk {}) time_since_reset={:.3}",
                clock(fr.time),
                fr.time,
                fr.frame,
                player_name(&names, &fr.player),
                fr.is_team_0,
                fr.reset_time,
                clock(fr.reset_time),
                fr.time_since_reset,
            );
        }
    }

    Ok(())
}

fn tag_name(tag: &GoalTag) -> String {
    format!("{:?}", tag.kind())
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
