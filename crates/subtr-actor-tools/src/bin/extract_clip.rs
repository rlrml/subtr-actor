//! Extract a serializable [`subtr_actor::ReplayClip`] from a replay file.
//!
//! The clip spans a region of interest given by frame or time bounds, padded
//! with warm-up lead-in frames so delta-based detectors behave identically to a
//! full-replay run (see `subtr_actor::clip`). The output JSON can be committed
//! as a test fixture and loaded with `ReplayClip::from_json`.
//!
//! Examples:
//!
//! ```sh
//! cargo run --bin extract_clip -- replay.replay --start-frame 1788 --end-frame 1910 -o clip.json
//! cargo run --bin extract_clip -- replay.replay --start-time 88.5 --end-time 93.0 -o clip.json
//! ```

use clap::Parser;
use subtr_actor::clip_replay_around;

/// Default warm-up frames before the region of interest. Differential fidelity
/// tests show processor state converges within ~30 frames; 90 (~3 seconds) also
/// covers slower mechanics trackers with margin.
const DEFAULT_LEAD_IN: usize = 90;
const DEFAULT_TAIL: usize = 30;

#[derive(Parser, Debug)]
#[command(about = "Extract a self-contained ReplayClip JSON fixture from a replay")]
struct Args {
    /// Path to the source .replay file
    replay: String,

    /// First frame of the region of interest (source replay frame index)
    #[arg(long, conflicts_with = "start_time")]
    start_frame: Option<usize>,

    /// Last frame of the region of interest, inclusive
    #[arg(long, conflicts_with = "end_time")]
    end_frame: Option<usize>,

    /// Start of the region of interest in replay seconds
    #[arg(long)]
    start_time: Option<f32>,

    /// End of the region of interest in replay seconds
    #[arg(long)]
    end_time: Option<f32>,

    /// Warm-up frames included before the region of interest
    #[arg(long, default_value_t = DEFAULT_LEAD_IN)]
    lead_in: usize,

    /// Frames included after the region of interest
    #[arg(long, default_value_t = DEFAULT_TAIL)]
    tail: usize,

    /// Output path for the clip JSON (defaults to stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Emit compact JSON instead of pretty-printed
    #[arg(long)]
    compact: bool,
}

fn frame_at_time(frames: &[boxcars::Frame], time: f32) -> usize {
    frames
        .iter()
        .position(|frame| frame.time >= time)
        .unwrap_or(frames.len().saturating_sub(1))
}

fn main() {
    let args = Args::parse();

    let data = std::fs::read(&args.replay)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", args.replay));
    let replay = boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", args.replay));

    let frames = &replay
        .network_frames
        .as_ref()
        .expect("replay should have network frames")
        .frames;

    let region_start = match (args.start_frame, args.start_time) {
        (Some(frame), _) => frame,
        (None, Some(time)) => frame_at_time(frames, time),
        (None, None) => panic!("provide --start-frame or --start-time"),
    };
    let region_end = match (args.end_frame, args.end_time) {
        (Some(frame), _) => frame,
        (None, Some(time)) => frame_at_time(frames, time),
        (None, None) => panic!("provide --end-frame or --end-time"),
    };

    let clip = clip_replay_around(&replay, region_start, region_end, args.lead_in, args.tail)
        .unwrap_or_else(|error| panic!("failed to build clip: {error:?}"));

    let json = if args.compact {
        serde_json::to_string(&clip)
    } else {
        clip.to_json()
    }
    .expect("clip should serialize");

    match &args.output {
        Some(path) => {
            std::fs::write(path, &json)
                .unwrap_or_else(|error| panic!("failed to write {path}: {error}"));
            eprintln!(
                "wrote {} ({} frames, source frames {}..={}, {} bytes)",
                path,
                clip.frames.len(),
                clip.provenance.source_first_real_frame,
                clip.provenance.source_last_real_frame,
                json.len(),
            );
        }
        None => println!("{json}"),
    }
}
