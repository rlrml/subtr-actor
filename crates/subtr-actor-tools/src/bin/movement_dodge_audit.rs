//! Dump every dodge ("movement dodge") in a replay as a labelable sheet.
//!
//! One row per `DodgeEvent` (emitted by the flip-impulse detector for every
//! dodge start), with a viewer-clock timestamp, the player, ground/air height,
//! speed, the estimated dodge impulse + direction, and a column flagging whether
//! the current speed-flip detector classified it as a speed flip. A trailing
//! `true_label` column is left blank for hand-labeling against the replay, so we
//! can use the corrected labels as ground truth for the classifier.

use std::collections::HashMap;

use anyhow::Context;
use clap::Parser;
use subtr_actor::{
    DodgeEvent, EventPayload, PlayerId, ReplayMeta, SpeedFlipEvent, StatsTimelineCollector,
};

#[derive(Debug, Parser)]
#[command(about = "Dump every dodge in a replay as a labelable sheet (TSV).")]
struct Args {
    /// Replay path to audit.
    replay: String,
    /// Only include dodges whose start speed is at least this (UU/s). Default 0
    /// includes everything; pass e.g. 500 to focus on movement dodges.
    #[arg(long, default_value_t = 0.0)]
    min_speed: f32,
}

fn main() -> anyhow::Result<()> {
    let Args { replay, min_speed } = Args::parse();
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

    let mut speed_flips: Vec<&SpeedFlipEvent> = Vec::new();
    let mut dodges: Vec<&DodgeEvent> = Vec::new();
    for event in &timeline.events.events {
        match &event.payload {
            EventPayload::SpeedFlip(event) => speed_flips.push(event),
            EventPayload::Dodge(event) => dodges.push(event),
            _ => {}
        }
    }

    let matching_speed_flip = |dodge: &DodgeEvent| -> Option<&SpeedFlipEvent> {
        speed_flips
            .iter()
            .copied()
            .find(|sf| sf.player == dodge.player && (sf.time - dodge.time).abs() <= 0.15)
    };

    println!(
        "idx\tclock\tabs_time\tframe\tplayer\tteam\tz\tspeed\tdspeed\timpulse_mag\tfwd\tright\tup\tdir_label\tconf\tonset_pitch\tonset_roll\tmin_fwd_z\tmax_fwd_dev\tmax_up_dev\tmin_up_z\ttq_x\ttq_y\ttq_hmag\tcurrent\tsf_conf\tsf_align\tsf_nose_sweep\tsf_roll_sweep\ttrue_label"
    );
    let mut idx = 0;
    for dodge in &dodges {
        let speed = dodge
            .dodge_impulse
            .as_ref()
            .map(|impulse| impulse.start_speed)
            .unwrap_or(0.0);
        if speed < min_speed {
            continue;
        }
        idx += 1;
        let clock = format_clock(dodge.time - clock_zero);
        let z = dodge
            .dodge_impulse
            .as_ref()
            .map(|impulse| impulse.start_position[2]);
        let (dspeed, mag, fwd, right, up, dir, conf) = match &dodge.dodge_impulse {
            Some(i) => (
                Some(i.end_speed - i.start_speed),
                Some(i.estimated_horizontal_impulse_magnitude),
                Some(i.local_forward_component),
                Some(i.local_right_component),
                Some(i.local_up_component),
                i.direction_label.clone(),
                Some(i.confidence),
            ),
            None => (None, None, None, None, None, "no_impulse".to_owned(), None),
        };
        let (onset_pitch, onset_roll, min_fwd_z, max_fwd_dev, max_up_dev, min_up_z) =
            match &dodge.dodge_rotation {
                Some(r) => (
                    Some(r.onset_pitch_rate),
                    Some(r.onset_roll_rate),
                    Some(r.min_forward_z),
                    Some(r.max_forward_deviation_degrees),
                    Some(r.max_up_deviation_degrees),
                    Some(r.min_up_z),
                ),
                None => (None, None, None, None, None, None),
            };
        let (tq_x, tq_y, tq_hmag) = match dodge.dodge_torque {
            Some([x, y, _]) => (Some(x), Some(y), Some((x * x + y * y).sqrt())),
            None => (None, None, None),
        };
        let speed_flip = matching_speed_flip(dodge);
        println!(
            "{idx}\t{clock}\t{:.3}\t{}\t{}\t{}\t{}\t{:.0}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t",
            dodge.time,
            dodge.frame,
            player_name(&names, &dodge.player),
            if dodge.is_team_0 { 0 } else { 1 },
            opt(z, 1),
            speed,
            opt(dspeed, 0),
            opt(mag, 1),
            opt(fwd, 3),
            opt(right, 3),
            opt(up, 3),
            dir,
            opt(conf, 2),
            opt(onset_pitch, 2),
            opt(onset_roll, 2),
            opt(min_fwd_z, 3),
            opt(max_fwd_dev, 1),
            opt(max_up_dev, 1),
            opt(min_up_z, 3),
            opt(tq_x, 2),
            opt(tq_y, 2),
            opt(tq_hmag, 2),
            if speed_flip.is_some() {
                "SPEED_FLIP"
            } else {
                ""
            },
            opt(speed_flip.map(|event| event.confidence), 3),
            opt(speed_flip.map(|event| event.min_travel_alignment), 3),
            opt(
                speed_flip.map(|event| event.max_forward_deviation_degrees),
                1,
            ),
            opt(speed_flip.map(|event| event.roll_sweep_degrees), 1),
        );
    }
    eprintln!(
        "{} dodges ({} currently classified speed_flip) over {:.1}s",
        idx,
        speed_flips.len(),
        timeline
            .frames
            .last()
            .map(|f| f.time - clock_zero)
            .unwrap_or(0.0),
    );
    Ok(())
}

fn opt(value: Option<f32>, precision: usize) -> String {
    match value {
        Some(value) => format!("{value:.precision$}"),
        None => String::new(),
    }
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
