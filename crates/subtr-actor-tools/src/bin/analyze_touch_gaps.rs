use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use subtr_actor::{
    Collector, PlayerId, ProcessorView, TimeAdvance, TouchCandidateScoring,
    ball_trajectory_deviation_with_gravity, touch_candidate_contact_gap_rank_with_hitbox,
    vec_to_glam,
};

const FIELD_HALF_WIDTH: f32 = 4096.0;
const FIELD_HALF_LENGTH: f32 = 5120.0;
const CEILING_Z: f32 = 2044.0;
const HIGH_CONFIDENCE_SURFACE_MARGIN: f32 = 1400.0;
const HIGH_CONFIDENCE_MIN_BALL_Z: f32 = 250.0;
const HIGH_CONFIDENCE_IMPULSE_COOLDOWN_SECONDS: f32 = 0.25;

#[derive(Debug, Parser)]
struct Args {
    #[arg(required = true)]
    replay_paths: Vec<PathBuf>,
    #[arg(long, default_value_t = 4)]
    window_frames: usize,
    #[arg(long, alias = "gap-threshold", default_value_t = 5.0)]
    strict_gap_threshold: f32,
    #[arg(long, default_value_t = 25.0)]
    relaxed_gap_threshold: f32,
    #[arg(long, alias = "velocity-threshold", default_value_t = 50.0)]
    strict_velocity_threshold: f32,
    #[arg(long, default_value_t = 1000.0)]
    relaxed_velocity_threshold: f32,
    #[arg(long, default_value_t = 1000.0)]
    high_confidence_velocity_threshold: f32,
    #[arg(long, default_value_t = HIGH_CONFIDENCE_SURFACE_MARGIN)]
    high_confidence_surface_margin: f32,
}

impl From<&Args> for TouchCandidateScoring {
    fn from(args: &Args) -> Self {
        TouchCandidateScoring {
            strict_contact_gap_threshold: args.strict_gap_threshold,
            relaxed_contact_gap_threshold: args.relaxed_gap_threshold,
            strict_contact_min_velocity_deviation: args.strict_velocity_threshold,
            relaxed_contact_min_velocity_deviation: args.relaxed_velocity_threshold,
            ..TouchCandidateScoring::DEFAULT
        }
    }
}

#[derive(Debug, Clone)]
struct TouchGapSample {
    replay: String,
    frame: usize,
    time: f32,
    team_is_team_0: bool,
    player_id: Option<PlayerId>,
    player_name: Option<String>,
    closest_contact_gap: Option<f32>,
    current_contact_gap: Option<f32>,
    trajectory_velocity_deviation: Option<f32>,
    window_closest_contact_gap: Option<f32>,
    window_trajectory_velocity_deviation: Option<f32>,
}

#[derive(Debug, Clone)]
struct FrameTeamGapSample {
    replay: String,
    frame: usize,
    time: f32,
    ball_position: glam::Vec3,
    team_is_team_0: bool,
    closest_contact_gap: Option<f32>,
    current_contact_gap: Option<f32>,
    player_id: Option<PlayerId>,
    player_name: Option<String>,
    trajectory_velocity_deviation: Option<f32>,
}

#[derive(Debug, Default)]
struct TouchGapCollector {
    replay: String,
    samples: Vec<TouchGapSample>,
    frame_team_samples: Vec<FrameTeamGapSample>,
    previous_ball_body: Option<(boxcars::RigidBody, f32)>,
}

impl TouchGapCollector {
    fn new(replay: String) -> Self {
        Self {
            replay,
            samples: Vec::new(),
            frame_team_samples: Vec::new(),
            previous_ball_body: None,
        }
    }

    fn best_team_gap(
        processor: &dyn ProcessorView,
        ball_body: &boxcars::RigidBody,
        team_is_team_0: bool,
        current_time: f32,
    ) -> Option<(f32, f32, PlayerId, Option<String>)> {
        processor
            .iter_player_ids_in_order()
            .filter(|player_id| {
                processor.get_player_is_team_0(player_id).ok() == Some(team_is_team_0)
            })
            .filter_map(|player_id| {
                let player_body = processor
                    .get_velocity_applied_player_rigid_body(player_id, current_time)
                    .ok()?;
                let (closest_gap, current_gap) = touch_candidate_contact_gap_rank_with_hitbox(
                    ball_body,
                    &player_body,
                    processor.get_player_car_hitbox(player_id),
                )?;
                Some((
                    closest_gap,
                    current_gap,
                    player_id.clone(),
                    processor.get_player_name(player_id).ok(),
                ))
            })
            .min_by(|left, right| left.0.total_cmp(&right.0))
    }
}

impl Collector for TouchGapCollector {
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> subtr_actor::SubtrActorResult<TimeAdvance> {
        let ball_body = processor
            .get_velocity_applied_ball_rigid_body(current_time)
            .ok();
        let trajectory_velocity_deviation = ball_body.as_ref().and_then(|ball_body| {
            self.previous_ball_body
                .as_ref()
                .and_then(|(previous_ball_body, previous_time)| {
                    ball_trajectory_deviation_with_gravity(
                        previous_ball_body,
                        *previous_time,
                        ball_body,
                        current_time,
                        -650.0,
                    )
                })
                .map(|deviation| deviation.velocity_deviation)
        });
        if let Some(ball_body) = ball_body.as_ref() {
            for team_is_team_0 in [true, false] {
                let best = Self::best_team_gap(processor, ball_body, team_is_team_0, current_time);
                let (closest_contact_gap, current_contact_gap, player_id, player_name) = best
                    .map(|(closest_gap, current_gap, player_id, player_name)| {
                        (
                            Some(closest_gap),
                            Some(current_gap),
                            Some(player_id),
                            player_name,
                        )
                    })
                    .unwrap_or((None, None, None, None));
                self.frame_team_samples.push(FrameTeamGapSample {
                    replay: self.replay.clone(),
                    frame: frame_number,
                    time: current_time,
                    ball_position: vec_to_glam(&ball_body.location),
                    team_is_team_0,
                    closest_contact_gap,
                    current_contact_gap,
                    player_id,
                    player_name,
                    trajectory_velocity_deviation,
                });
            }
        }

        let touch_events = processor.current_frame_touch_events();
        if touch_events.is_empty() {
            self.previous_ball_body = ball_body.map(|ball_body| (ball_body, current_time));
            return Ok(TimeAdvance::NextFrame);
        }

        for event in touch_events {
            let best = ball_body.as_ref().and_then(|ball_body| {
                Self::best_team_gap(processor, ball_body, event.team_is_team_0, current_time)
            });
            let (closest_contact_gap, current_contact_gap, player_id, player_name) = best
                .map(|(closest_gap, current_gap, player_id, player_name)| {
                    (
                        Some(closest_gap),
                        Some(current_gap),
                        Some(player_id),
                        player_name,
                    )
                })
                .unwrap_or((None, None, None, None));
            self.samples.push(TouchGapSample {
                replay: self.replay.clone(),
                frame: frame_number,
                time: current_time,
                team_is_team_0: event.team_is_team_0,
                player_id,
                player_name,
                closest_contact_gap,
                current_contact_gap,
                trajectory_velocity_deviation,
                window_closest_contact_gap: None,
                window_trajectory_velocity_deviation: None,
            });
        }

        self.previous_ball_body = ball_body.map(|ball_body| (ball_body, current_time));
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

fn percentile(sorted: &[f32], percentile: f32) -> Option<f32> {
    if sorted.is_empty() {
        return None;
    }
    let index = ((sorted.len() - 1) as f32 * percentile).round() as usize;
    sorted.get(index).copied()
}

fn enrich_window_samples(
    samples: &mut [TouchGapSample],
    frames: &[FrameTeamGapSample],
    window_frames: usize,
) {
    for sample in samples {
        let start_frame = sample.frame.saturating_sub(window_frames);
        let end_frame = sample.frame.saturating_add(window_frames);
        let mut best_gap_frame: Option<&FrameTeamGapSample> = None;
        let mut best_velocity_deviation: Option<f32> = None;

        for frame in frames.iter().filter(|frame| {
            frame.team_is_team_0 == sample.team_is_team_0
                && frame.frame >= start_frame
                && frame.frame <= end_frame
        }) {
            if frame.closest_contact_gap.is_some_and(|gap| {
                best_gap_frame
                    .and_then(|best| best.closest_contact_gap)
                    .is_none_or(|best_gap| gap < best_gap)
            }) {
                best_gap_frame = Some(frame);
            }
            if let Some(velocity_deviation) = frame.trajectory_velocity_deviation {
                best_velocity_deviation = Some(
                    best_velocity_deviation
                        .map(|best| best.max(velocity_deviation))
                        .unwrap_or(velocity_deviation),
                );
            }
        }

        if let Some(best_gap_frame) = best_gap_frame {
            sample.window_closest_contact_gap = best_gap_frame.closest_contact_gap;
            sample.player_id = best_gap_frame.player_id.clone();
            sample.player_name = best_gap_frame.player_name.clone();
            sample.current_contact_gap = best_gap_frame.current_contact_gap;
        }
        sample.window_trajectory_velocity_deviation = best_velocity_deviation;
    }
}

fn print_summary(samples: &[TouchGapSample]) {
    let mut gaps = samples
        .iter()
        .filter_map(|sample| {
            sample
                .window_closest_contact_gap
                .or(sample.closest_contact_gap)
        })
        .collect::<Vec<_>>();
    gaps.sort_by(f32::total_cmp);
    let unresolved_same_frame = samples
        .iter()
        .filter(|sample| sample.closest_contact_gap.is_none())
        .count();
    let unresolved_window = samples
        .iter()
        .filter(|sample| {
            sample
                .window_closest_contact_gap
                .or(sample.closest_contact_gap)
                .is_none()
        })
        .count();
    let touching = gaps.iter().filter(|gap| **gap <= 0.0).count();
    let within_1 = gaps.iter().filter(|gap| **gap <= 1.0).count();
    let within_5 = gaps.iter().filter(|gap| **gap <= 5.0).count();
    let within_10 = gaps.iter().filter(|gap| **gap <= 10.0).count();
    let within_25 = gaps.iter().filter(|gap| **gap <= 25.0).count();
    let with_velocity_change = samples
        .iter()
        .filter(|sample| {
            sample
                .window_trajectory_velocity_deviation
                .or(sample.trajectory_velocity_deviation)
                .is_some_and(|deviation| deviation >= 1.0)
        })
        .count();

    println!("touches: {}", samples.len());
    println!("resolved window: {}", gaps.len());
    println!("unresolved window: {unresolved_window}");
    println!("unresolved same-frame: {unresolved_same_frame}");
    println!(
        "trajectory_velocity_deviation>=1: {with_velocity_change}/{}",
        samples.len()
    );
    if gaps.is_empty() {
        return;
    }
    println!("gap<=0: {touching}/{}", gaps.len());
    println!("gap<=1: {within_1}/{}", gaps.len());
    println!("gap<=5: {within_5}/{}", gaps.len());
    println!("gap<=10: {within_10}/{}", gaps.len());
    println!("gap<=25: {within_25}/{}", gaps.len());
    for velocity_threshold in [1.0, 25.0, 50.0, 100.0] {
        let row = [0.0, 1.0, 5.0, 10.0, 25.0]
            .into_iter()
            .map(|gap_threshold| {
                samples
                    .iter()
                    .filter(|sample| {
                        sample
                            .window_closest_contact_gap
                            .or(sample.closest_contact_gap)
                            .is_some_and(|gap| gap <= gap_threshold)
                            && sample
                                .window_trajectory_velocity_deviation
                                .or(sample.trajectory_velocity_deviation)
                                .is_some_and(|deviation| deviation >= velocity_threshold)
                    })
                    .count()
            })
            .collect::<Vec<_>>();
        println!(
            "gap/vel matrix vel>={velocity_threshold:.0}: <=0 {} <=1 {} <=5 {} <=10 {} <=25 {}",
            row[0], row[1], row[2], row[3], row[4],
        );
    }
    println!(
        "gap percentiles: p50={:.2} p75={:.2} p90={:.2} p95={:.2} p99={:.2} max={:.2}",
        percentile(&gaps, 0.50).unwrap_or_default(),
        percentile(&gaps, 0.75).unwrap_or_default(),
        percentile(&gaps, 0.90).unwrap_or_default(),
        percentile(&gaps, 0.95).unwrap_or_default(),
        percentile(&gaps, 0.99).unwrap_or_default(),
        gaps.last().copied().unwrap_or_default(),
    );
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum PhysicsCooldownKey {
    Player(PlayerId),
    Team(bool),
}

fn physics_cooldown_key(sample: &FrameTeamGapSample) -> PhysicsCooldownKey {
    sample
        .player_id
        .clone()
        .map(PhysicsCooldownKey::Player)
        .unwrap_or(PhysicsCooldownKey::Team(sample.team_is_team_0))
}

fn is_physics_candidate(
    sample: &FrameTeamGapSample,
    scoring_config: TouchCandidateScoring,
) -> bool {
    let Some(gap) = sample.closest_contact_gap else {
        return false;
    };
    let Some(velocity_deviation) = sample.trajectory_velocity_deviation else {
        return false;
    };
    scoring_config.accepts_contact_gap(gap, 0.0, velocity_deviation)
}

fn primary_physics_candidates(
    frame_samples: &[FrameTeamGapSample],
    scoring_config: TouchCandidateScoring,
) -> Vec<&FrameTeamGapSample> {
    let mut best_by_frame = HashMap::<(&str, usize), &FrameTeamGapSample>::new();
    for sample in frame_samples
        .iter()
        .filter(|sample| is_physics_candidate(sample, scoring_config))
    {
        let key = (sample.replay.as_str(), sample.frame);
        if best_by_frame.get(&key).is_none_or(|best| {
            scoring_config
                .score_contact_gap(sample.closest_contact_gap.unwrap_or(f32::INFINITY), false)
                < scoring_config
                    .score_contact_gap(best.closest_contact_gap.unwrap_or(f32::INFINITY), false)
        }) {
            best_by_frame.insert(key, sample);
        }
    }

    let mut candidates = best_by_frame.into_values().collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        left.time
            .total_cmp(&right.time)
            .then_with(|| left.frame.cmp(&right.frame))
            .then_with(|| left.team_is_team_0.cmp(&right.team_is_team_0))
    });
    candidates
}

fn cooldown_physics_candidates(
    frame_samples: &[FrameTeamGapSample],
    scoring_config: TouchCandidateScoring,
) -> Vec<&FrameTeamGapSample> {
    const TOUCH_RATE_LIMIT_SECONDS: f32 = 0.25;
    const FLOAT_EPSILON: f32 = 0.0001;

    let mut last_touch_times = HashMap::new();
    let mut accepted = Vec::new();
    for candidate in primary_physics_candidates(frame_samples, scoring_config) {
        let key = physics_cooldown_key(candidate);
        let allowed = last_touch_times.get(&key).is_none_or(|last_time| {
            candidate.time - last_time + FLOAT_EPSILON >= TOUCH_RATE_LIMIT_SECONDS
        });
        if allowed {
            last_touch_times.insert(key, candidate.time);
            accepted.push(candidate);
        }
    }
    accepted
}

fn explicit_event_near_candidate(
    candidate: &FrameTeamGapSample,
    samples: &[TouchGapSample],
    window_frames: usize,
) -> bool {
    let start_frame = candidate.frame.saturating_sub(window_frames);
    let end_frame = candidate.frame.saturating_add(window_frames);
    samples.iter().any(|sample| {
        sample.replay == candidate.replay
            && sample.team_is_team_0 == candidate.team_is_team_0
            && sample.frame >= start_frame
            && sample.frame <= end_frame
    })
}

fn print_physics_summary(
    samples: &[TouchGapSample],
    frame_samples: &[FrameTeamGapSample],
    window_frames: usize,
    scoring_config: TouchCandidateScoring,
) {
    let primary_physics_candidates = primary_physics_candidates(frame_samples, scoring_config);
    let cooldowned_physics_candidates = cooldown_physics_candidates(frame_samples, scoring_config);
    let physics_with_explicit_nearby = cooldowned_physics_candidates
        .iter()
        .filter(|candidate| explicit_event_near_candidate(candidate, samples, window_frames))
        .count();
    let explicit_with_physics_nearby = samples
        .iter()
        .filter(|sample| {
            cooldowned_physics_candidates.iter().any(|candidate| {
                let start_frame = sample.frame.saturating_sub(window_frames);
                let end_frame = sample.frame.saturating_add(window_frames);
                candidate.replay == sample.replay
                    && candidate.team_is_team_0 == sample.team_is_team_0
                    && candidate.frame >= start_frame
                    && candidate.frame <= end_frame
            })
        })
        .count();

    println!(
        "physics candidates strict_gap<={:.1} strict_vel>={:.1} relaxed_gap<={:.1} relaxed_vel>={:.1}: primary_frames={} cooldowned={}",
        scoring_config.strict_contact_gap_threshold,
        scoring_config.strict_contact_min_velocity_deviation,
        scoring_config.relaxed_contact_gap_threshold,
        scoring_config.relaxed_contact_min_velocity_deviation,
        primary_physics_candidates.len(),
        cooldowned_physics_candidates.len()
    );
    println!(
        "physics candidates with explicit team update within ±{window_frames} frames: {physics_with_explicit_nearby}/{}",
        cooldowned_physics_candidates.len()
    );
    println!(
        "explicit team updates with physics candidate within ±{window_frames} frames: {explicit_with_physics_nearby}/{}",
        samples.len()
    );
}

#[derive(Debug, Clone)]
struct HighConfidenceImpulseSample<'a> {
    sample: &'a FrameTeamGapSample,
    window_best_gap: Option<f32>,
    window_best_player_name: Option<String>,
    window_best_replay: String,
    window_best_frame: usize,
}

fn ball_is_far_from_arena_surfaces(position: glam::Vec3, margin: f32) -> bool {
    position.x.abs() <= FIELD_HALF_WIDTH - margin
        && position.y.abs() <= FIELD_HALF_LENGTH - margin
        && position.z >= HIGH_CONFIDENCE_MIN_BALL_Z
        && position.z <= CEILING_Z - margin
}

fn best_gap_near_frame<'a>(
    frame_samples: &'a [FrameTeamGapSample],
    replay: &str,
    frame: usize,
    window_frames: usize,
) -> Option<&'a FrameTeamGapSample> {
    let start_frame = frame.saturating_sub(window_frames);
    let end_frame = frame.saturating_add(window_frames);
    frame_samples
        .iter()
        .filter(|sample| {
            sample.replay == replay && sample.frame >= start_frame && sample.frame <= end_frame
        })
        .filter(|sample| sample.closest_contact_gap.is_some())
        .min_by(|left, right| {
            left.closest_contact_gap
                .unwrap_or(f32::INFINITY)
                .total_cmp(&right.closest_contact_gap.unwrap_or(f32::INFINITY))
        })
}

fn high_confidence_impulse_samples(
    frame_samples: &[FrameTeamGapSample],
    window_frames: usize,
    velocity_threshold: f32,
    surface_margin: f32,
) -> Vec<HighConfidenceImpulseSample<'_>> {
    let mut primary_impulse_frames = HashMap::<(&str, usize), &FrameTeamGapSample>::new();
    for sample in frame_samples.iter().filter(|sample| {
        sample
            .trajectory_velocity_deviation
            .is_some_and(|deviation| deviation >= velocity_threshold)
            && ball_is_far_from_arena_surfaces(sample.ball_position, surface_margin)
    }) {
        let key = (sample.replay.as_str(), sample.frame);
        if primary_impulse_frames.get(&key).is_none_or(|best| {
            sample.closest_contact_gap.unwrap_or(f32::INFINITY)
                < best.closest_contact_gap.unwrap_or(f32::INFINITY)
        }) {
            primary_impulse_frames.insert(key, sample);
        }
    }

    let mut impulse_frames = primary_impulse_frames.into_values().collect::<Vec<_>>();
    impulse_frames.sort_by(|left, right| {
        left.replay
            .cmp(&right.replay)
            .then_with(|| left.time.total_cmp(&right.time))
            .then_with(|| left.frame.cmp(&right.frame))
    });

    let mut last_impulse_times = HashMap::<&str, f32>::new();
    let mut accepted = Vec::new();
    for sample in impulse_frames {
        let allowed = last_impulse_times
            .get(sample.replay.as_str())
            .is_none_or(|last_time| {
                sample.time - last_time >= HIGH_CONFIDENCE_IMPULSE_COOLDOWN_SECONDS
            });
        if !allowed {
            continue;
        }
        last_impulse_times.insert(sample.replay.as_str(), sample.time);
        let best_gap_sample = best_gap_near_frame(
            frame_samples,
            sample.replay.as_str(),
            sample.frame,
            window_frames,
        );
        accepted.push(HighConfidenceImpulseSample {
            sample,
            window_best_gap: best_gap_sample.and_then(|sample| sample.closest_contact_gap),
            window_best_player_name: best_gap_sample.and_then(|sample| sample.player_name.clone()),
            window_best_replay: best_gap_sample
                .map(|sample| sample.replay.clone())
                .unwrap_or_else(|| sample.replay.clone()),
            window_best_frame: best_gap_sample
                .map(|sample| sample.frame)
                .unwrap_or(sample.frame),
        });
    }
    accepted
}

fn print_high_confidence_impulse_summary(
    frame_samples: &[FrameTeamGapSample],
    window_frames: usize,
    velocity_threshold: f32,
    surface_margin: f32,
) {
    let impulse_samples = high_confidence_impulse_samples(
        frame_samples,
        window_frames,
        velocity_threshold,
        surface_margin,
    );
    let mut gaps = impulse_samples
        .iter()
        .filter_map(|sample| sample.window_best_gap)
        .collect::<Vec<_>>();
    gaps.sort_by(f32::total_cmp);
    let within_0 = gaps.iter().filter(|gap| **gap <= 0.0).count();
    let within_1 = gaps.iter().filter(|gap| **gap <= 1.0).count();
    let within_5 = gaps.iter().filter(|gap| **gap <= 5.0).count();
    let within_10 = gaps.iter().filter(|gap| **gap <= 10.0).count();
    let within_25 = gaps.iter().filter(|gap| **gap <= 25.0).count();
    let within_50 = gaps.iter().filter(|gap| **gap <= 50.0).count();
    let within_100 = gaps.iter().filter(|gap| **gap <= 100.0).count();

    println!(
        "high-confidence impulses vel>={velocity_threshold:.0}, surface_margin={surface_margin:.0}, window=±{window_frames}: {}",
        impulse_samples.len()
    );
    if gaps.is_empty() {
        return;
    }
    println!(
        "high-confidence impulse gaps: <=0 {within_0}/{} <=1 {within_1}/{} <=5 {within_5}/{} <=10 {within_10}/{} <=25 {within_25}/{} <=50 {within_50}/{} <=100 {within_100}/{}",
        gaps.len(),
        gaps.len(),
        gaps.len(),
        gaps.len(),
        gaps.len(),
        gaps.len(),
        gaps.len(),
    );
    println!(
        "high-confidence gap percentiles: min={:.2} p50={:.2} p75={:.2} p90={:.2} p95={:.2} p99={:.2} max={:.2}",
        gaps.first().copied().unwrap_or_default(),
        percentile(&gaps, 0.50).unwrap_or_default(),
        percentile(&gaps, 0.75).unwrap_or_default(),
        percentile(&gaps, 0.90).unwrap_or_default(),
        percentile(&gaps, 0.95).unwrap_or_default(),
        percentile(&gaps, 0.99).unwrap_or_default(),
        gaps.last().copied().unwrap_or_default(),
    );

    let mut worst = impulse_samples
        .iter()
        .filter(|sample| sample.window_best_gap.is_some())
        .collect::<Vec<_>>();
    worst.sort_by(|left, right| {
        right
            .window_best_gap
            .unwrap_or_default()
            .total_cmp(&left.window_best_gap.unwrap_or_default())
    });
    println!("worst high-confidence impulse gaps:");
    for sample in worst.into_iter().take(12) {
        println!(
            "{:>7.2}uu vel_dev={:>7.2} impulse_frame={:<6} best_frame={:<6} time={:>8.3} ball=({:>7.1},{:>7.1},{:>6.1}) player={} replay={}",
            sample.window_best_gap.unwrap_or_default(),
            sample
                .sample
                .trajectory_velocity_deviation
                .unwrap_or_default(),
            sample.sample.frame,
            sample.window_best_frame,
            sample.sample.time,
            sample.sample.ball_position.x,
            sample.sample.ball_position.y,
            sample.sample.ball_position.z,
            sample
                .window_best_player_name
                .as_deref()
                .unwrap_or("<unknown>"),
            sample.window_best_replay,
        );
    }
}

fn print_worst(samples: &[TouchGapSample]) {
    let mut worst = samples
        .iter()
        .filter(|sample| {
            sample
                .window_closest_contact_gap
                .or(sample.closest_contact_gap)
                .is_some()
        })
        .collect::<Vec<_>>();
    worst.sort_by(|left, right| {
        right
            .window_closest_contact_gap
            .or(right.closest_contact_gap)
            .unwrap_or_default()
            .total_cmp(
                &left
                    .window_closest_contact_gap
                    .or(left.closest_contact_gap)
                    .unwrap_or_default(),
            )
    });
    println!("worst resolved explicit touches:");
    for sample in worst.into_iter().take(20) {
        println!(
            "{:>7.2}uu same_frame={:>7.2}uu vel_dev={:>7.2} frame={:<6} time={:>8.3} team={} player={} replay={}",
            sample
                .window_closest_contact_gap
                .or(sample.closest_contact_gap)
                .unwrap_or_default(),
            sample.closest_contact_gap.unwrap_or_default(),
            sample
                .window_trajectory_velocity_deviation
                .or(sample.trajectory_velocity_deviation)
                .unwrap_or_default(),
            sample.frame,
            sample.time,
            if sample.team_is_team_0 { 0 } else { 1 },
            sample.player_name.as_deref().unwrap_or("<unknown>"),
            sample.replay,
        );
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let scoring_config = TouchCandidateScoring::from(&args);
    let mut all_samples = Vec::new();
    let mut all_frame_samples = Vec::new();

    for path in &args.replay_paths {
        let replay = parse_replay(path)?;
        let label = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<replay>")
            .to_owned();
        let mut collector = TouchGapCollector::new(label.clone())
            .process_replay(&replay)
            .map_err(|error| anyhow::anyhow!("processing {}: {error:?}", path.display()))?;
        enrich_window_samples(
            &mut collector.samples,
            &collector.frame_team_samples,
            args.window_frames,
        );
        println!("\n== {label} ==");
        print_summary(&collector.samples);
        print_physics_summary(
            &collector.samples,
            &collector.frame_team_samples,
            args.window_frames,
            scoring_config,
        );
        print_high_confidence_impulse_summary(
            &collector.frame_team_samples,
            args.window_frames,
            args.high_confidence_velocity_threshold,
            args.high_confidence_surface_margin,
        );
        all_samples.extend(collector.samples);
        all_frame_samples.extend(collector.frame_team_samples);
    }

    println!("\n== all replays ==");
    print_summary(&all_samples);
    print_physics_summary(
        &all_samples,
        &all_frame_samples,
        args.window_frames,
        scoring_config,
    );
    print_high_confidence_impulse_summary(
        &all_frame_samples,
        args.window_frames,
        args.high_confidence_velocity_threshold,
        args.high_confidence_surface_margin,
    );
    print_worst(&all_samples);

    Ok(())
}
