//! Audit kickoffs for the "contested 50/50 but only one player credited a touch" bug.
//!
//! For each kickoff we measure, frame by frame, how close each team's closest car
//! gets to the ball (contact gap). If both teams physically reach the ball
//! (small contact gap) around the kickoff contact, but the attribution pipeline
//! only credits a touch to one team, we flag the kickoff as suspicious.

use anyhow::{Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use subtr_actor::{
    Collector, EventPayload, PlayerId, ProcessorView, StatsTimelineCollector, TimeAdvance,
    ball_trajectory_deviation_with_gravity, touch_candidate_contact_gap_rank_with_hitbox,
    vec_to_glam,
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(required = true)]
    replay_paths: Vec<PathBuf>,
    /// Contact-gap threshold (uu) below which a team is considered to have physically reached the ball.
    #[arg(long, default_value_t = 40.0)]
    contact_gap_threshold: f32,
    /// Frames before/after the kickoff contact frame to inspect.
    #[arg(long, default_value_t = 6)]
    window_frames: usize,
    /// Print a line for every kickoff, not just suspicious ones.
    #[arg(long, default_value_t = false)]
    verbose: bool,
    /// Dump per-frame both-team gap/deviation/marker detail around this frame (first replay only).
    #[arg(long)]
    detail_around: Option<usize>,
}

#[derive(Debug, Clone)]
struct FrameGap {
    frame: usize,
    time: f32,
    ball_speed: f32,
    ball_pos_deviation: Option<f32>,
    ball_vel_deviation: Option<f32>,
    marker_teams: Vec<bool>,
    team0: Option<(f32, PlayerId, Option<String>)>,
    team1: Option<(f32, PlayerId, Option<String>)>,
}

#[derive(Debug, Default)]
struct GapCollector {
    frames: Vec<FrameGap>,
    previous_ball: Option<(boxcars::RigidBody, f32)>,
    previous_time: Option<f32>,
}

impl GapCollector {
    fn best_team_gap(
        processor: &dyn ProcessorView,
        ball_body: &boxcars::RigidBody,
        team_is_team_0: bool,
        current_time: f32,
    ) -> Option<(f32, PlayerId, Option<String>)> {
        processor
            .iter_player_ids_in_order()
            .filter(|player_id| {
                processor.get_player_is_team_0(player_id).ok() == Some(team_is_team_0)
            })
            .filter_map(|player_id| {
                let player_body = processor
                    .get_velocity_applied_player_rigid_body(player_id, current_time)
                    .ok()?;
                let (closest_gap, _current_gap) = touch_candidate_contact_gap_rank_with_hitbox(
                    ball_body,
                    &player_body,
                    processor.get_player_car_hitbox(player_id),
                )?;
                Some((
                    closest_gap,
                    player_id.clone(),
                    processor.get_player_name(player_id).ok(),
                ))
            })
            .min_by(|left, right| left.0.total_cmp(&right.0))
    }
}

impl Collector for GapCollector {
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> subtr_actor::SubtrActorResult<TimeAdvance> {
        if let Ok(ball_body) = processor.get_velocity_applied_ball_rigid_body(current_time) {
            let ball_speed = ball_body
                .linear_velocity
                .map(|v| vec_to_glam(&v).length())
                .unwrap_or(0.0);
            let deviation = self
                .previous_ball
                .as_ref()
                .zip(self.previous_time)
                .and_then(|((prev, _), prev_time)| {
                    ball_trajectory_deviation_with_gravity(
                        prev,
                        prev_time,
                        &ball_body,
                        current_time,
                        -650.0,
                    )
                });
            let marker_teams = processor
                .current_frame_touch_events()
                .iter()
                .map(|event| event.team_is_team_0)
                .collect();
            self.frames.push(FrameGap {
                frame: frame_number,
                time: current_time,
                ball_speed,
                ball_pos_deviation: deviation.map(|d| d.position_deviation),
                ball_vel_deviation: deviation.map(|d| d.velocity_deviation),
                marker_teams,
                team0: Self::best_team_gap(processor, &ball_body, true, current_time),
                team1: Self::best_team_gap(processor, &ball_body, false, current_time),
            });
            self.previous_ball = Some((ball_body, current_time));
            self.previous_time = Some(current_time);
        }
        Ok(TimeAdvance::NextFrame)
    }
}

fn parse_replay(path: &Path) -> Result<boxcars::Replay> {
    let data = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    boxcars::ParserBuilder::new(&data)
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()
        .with_context(|| format!("parsing {}", path.display()))
}

struct KickoffInfo {
    start_time: f32,
    first_touch_frame: Option<usize>,
    first_touch_time: Option<f32>,
    kickoff_type: String,
    kickoff_direction: String,
    team0_taker_touch_time: Option<f32>,
    team1_taker_touch_time: Option<f32>,
    team0_taker: Option<(String, bool)>, // (player name-ish, has_first_touch)
    team1_taker: Option<(String, bool)>,
}

fn audit(label: &str, args: &Args, replay: &boxcars::Replay) -> Result<()> {
    // Pass 1: geometric gaps per frame.
    let gaps = GapCollector::default()
        .process_replay(replay)
        .map_err(|e| anyhow::anyhow!("gap pass {label}: {e:?}"))?
        .frames;

    // Pass 2: high-level timeline kickoff events + touch events.
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(replay)
        .map_err(|e| anyhow::anyhow!("timeline {label}: {e:?}"))?;

    // Collect touch events (frame, team).
    let touches: Vec<(usize, f32, Option<bool>)> = timeline
        .events
        .events
        .iter()
        .filter(|event| matches!(event.payload, EventPayload::Touch(_)))
        .map(|event| {
            let (frame, time) = event.meta.timing.start();
            (frame, time, event.meta.team_is_team_0)
        })
        .collect();

    let kickoffs: Vec<KickoffInfo> = timeline
        .events
        .events
        .iter()
        .filter_map(|event| match &event.payload {
            EventPayload::Kickoff(kickoff) => Some(KickoffInfo {
                start_time: kickoff.start_time,
                first_touch_frame: kickoff.first_touch_frame,
                first_touch_time: kickoff.first_touch_time,
                kickoff_type: format!("{:?}", kickoff.kickoff_type),
                kickoff_direction: format!("{:?}", kickoff.kickoff_direction),
                team0_taker_touch_time: kickoff.team_zero_taker_touch_time,
                team1_taker_touch_time: kickoff.team_one_taker_touch_time,
                team0_taker: kickoff
                    .team_zero_taker
                    .as_ref()
                    .map(|t| (format!("{:?}", t.player), t.first_touch_time.is_some())),
                team1_taker: kickoff
                    .team_one_taker
                    .as_ref()
                    .map(|t| (format!("{:?}", t.player), t.first_touch_time.is_some())),
            }),
            _ => None,
        })
        .collect();

    if let Some(center) = args.detail_around {
        let lo = center.saturating_sub(12);
        let hi = center + 12;
        println!("== detail around frame {center} ({label}) ==");
        println!(
            "frame time | t0_gap t0_who | t1_gap t1_who | ball_speed pos_dev vel_dev | markers"
        );
        for f in gaps.iter().filter(|f| f.frame >= lo && f.frame <= hi) {
            let fmt = |t: &Option<(f32, PlayerId, Option<String>)>| match t {
                Some((g, _, name)) => {
                    format!("{:>6.1} {:<14}", g, name.clone().unwrap_or_default())
                }
                None => format!("{:>6} {:<14}", "-", ""),
            };
            println!(
                "{:>6} {:>6.2} | {} | {} | {:>7.0} {:>6.0} {:>7.0} | {:?}",
                f.frame,
                f.time,
                fmt(&f.team0),
                fmt(&f.team1),
                f.ball_speed,
                f.ball_pos_deviation.unwrap_or(0.0),
                f.ball_vel_deviation.unwrap_or(0.0),
                f.marker_teams,
            );
        }
        println!();
    }

    let mut flagged = 0usize;
    for (i, k) in kickoffs.iter().enumerate() {
        let Some(ft_frame) = k.first_touch_frame else {
            continue;
        };
        // Window of frames around the contact.
        let lo = ft_frame.saturating_sub(args.window_frames);
        let hi = ft_frame + args.window_frames;
        let mut team0_min = f32::INFINITY;
        let mut team1_min = f32::INFINITY;
        let mut team0_who = None;
        let mut team1_who = None;
        for f in gaps.iter().filter(|f| f.frame >= lo && f.frame <= hi) {
            if let Some((g, _, name)) = &f.team0 {
                if *g < team0_min {
                    team0_min = *g;
                    team0_who = name.clone();
                }
            }
            if let Some((g, _, name)) = &f.team1 {
                if *g < team1_min {
                    team1_min = *g;
                    team1_who = name.clone();
                }
            }
        }

        let both_reached =
            team0_min <= args.contact_gap_threshold && team1_min <= args.contact_gap_threshold;

        // Which teams got credited a touch in the contest window?
        let ft_time = k.first_touch_time.unwrap_or(k.start_time);
        let window_end = ft_time + 0.8;
        let mut t0_touch = false;
        let mut t1_touch = false;
        for (tf, tt, team) in &touches {
            if *tt >= k.start_time - 0.1 && *tt <= window_end && tf.abs_diff(ft_frame) <= 60 {
                match team {
                    Some(true) => t0_touch = true,
                    Some(false) => t1_touch = true,
                    None => {}
                }
            }
        }
        let credited_teams = t0_touch as u8 + t1_touch as u8;

        let suspicious = both_reached && credited_teams < 2;

        if suspicious {
            flagged += 1;
        }
        if suspicious || args.verbose {
            let tag = if suspicious { "** SUSPICIOUS **" } else { "" };
            println!(
                "  kickoff#{i} t={:.1} type={}/{} ft_frame={ft_frame} | gap_min team0={:.0}({:?}) team1={:.0}({:?}) both_reached={both_reached} | touch_credited team0={t0_touch} team1={t1_touch} | taker_touch_time t0={:?} t1={:?} | taker_has_touch t0={:?} t1={:?} {tag}",
                k.start_time,
                k.kickoff_type,
                k.kickoff_direction,
                team0_min,
                team0_who,
                team1_min,
                team1_who,
                k.team0_taker_touch_time,
                k.team1_taker_touch_time,
                k.team0_taker.as_ref().map(|t| t.1),
                k.team1_taker.as_ref().map(|t| t.1),
            );
        }
    }

    println!(
        "== {label}: {} kickoffs, {flagged} suspicious ==",
        kickoffs.len()
    );
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    for path in &args.replay_paths {
        let label = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<replay>")
            .to_owned();
        match parse_replay(path) {
            Ok(replay) => {
                let audited = audit(&label, &args, &replay);
                if let Err(e) = audited {
                    eprintln!("error {label}: {e:?}");
                }
            }
            Err(e) => eprintln!("parse error {label}: {e:?}"),
        }
    }
    Ok(())
}
