use anyhow::Context;
use clap::Parser;
use subtr_actor::{EventPayload, StatsTimelineCollector};

#[derive(Debug, Parser)]
#[command(
    about = "Dump per-kickoff win strength and band to evaluate the band calibration."
)]
struct Args {
    /// Replay paths to dump.
    #[arg(value_name = "replay", num_args = 1..)]
    paths: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let Args { paths } = Args::parse();
    let mut strengths: Vec<f32> = Vec::new();
    let mut band_counts: std::collections::BTreeMap<&'static str, u32> = Default::default();
    let mut outcome_counts: std::collections::BTreeMap<&'static str, u32> = Default::default();

    // TSV header
    println!("replay\tstart_time\toutcome\twin_strength\tband\texit_speed\texit_y_velocity");
    for path in &paths {
        if let Err(error) = dump_replay(
            path,
            &mut strengths,
            &mut band_counts,
            &mut outcome_counts,
        ) {
            eprintln!("skip {path}: {error:?}");
        }
    }

    strengths.sort_by(|left, right| left.total_cmp(right));
    eprintln!(
        "\n=== summary over {} kickoffs with strength ===",
        strengths.len()
    );
    eprintln!("bands: {band_counts:?}");
    eprintln!("outcomes: {outcome_counts:?}");
    if !strengths.is_empty() {
        let percentile = |q: f32| strengths[((strengths.len() - 1) as f32 * q) as usize];
        eprintln!(
            "strength min={:.2} p10={:.2} p25={:.2} p50={:.2} p75={:.2} p90={:.2} max={:.2}",
            strengths[0],
            percentile(0.10),
            percentile(0.25),
            percentile(0.50),
            percentile(0.75),
            percentile(0.90),
            strengths[strengths.len() - 1]
        );
    }
    Ok(())
}

fn dump_replay(
    path: &str,
    strengths: &mut Vec<f32>,
    band_counts: &mut std::collections::BTreeMap<&'static str, u32>,
    outcome_counts: &mut std::collections::BTreeMap<&'static str, u32>,
) -> anyhow::Result<()> {
    let data = std::fs::read(path).with_context(|| format!("failed to read {path}"))?;
    let replay = boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .with_context(|| format!("failed to parse {path}"))?;
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .map_err(|error| anyhow::anyhow!("failed to collect stats for {path}: {error:?}"))?;
    for event in &timeline.events.events {
        let EventPayload::Kickoff(kickoff) = &event.payload else {
            continue;
        };
        *outcome_counts
            .entry(kickoff.outcome.as_label_value())
            .or_insert(0) += 1;
        *band_counts
            .entry(kickoff.win_strength_band.as_label_value())
            .or_insert(0) += 1;
        if let Some(strength) = kickoff.win_strength {
            strengths.push(strength);
        }
        println!(
            "{path}\t{:.1}\t{}\t{:?}\t{}\t{:?}\t{:?}",
            kickoff.start_time,
            kickoff.outcome.as_label_value(),
            kickoff.win_strength,
            kickoff.win_strength_band.as_label_value(),
            kickoff.exit_speed,
            kickoff.exit_y_velocity,
        );
    }
    Ok(())
}
