use anyhow::{Context, Result};
use reqwest::blocking::Client;
use subtr_actor::ballchasing::{parse_replay_bytes, parse_replay_file};
use subtr_actor::{Collector, ReplayProcessor, TimeAdvance};

use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
struct LooseFlipResetCandidate {
    time: f32,
    frame: usize,
    player_id: subtr_actor::PlayerId,
    player_name: String,
    is_team_0: bool,
    score: f32,
    closest_approach_distance: f32,
    local_ball_position: glam::Vec3,
    underside_alignment: f32,
    player_height: f32,
    ball_height: f32,
}

#[derive(Debug, Clone)]
struct LoosePostWallDodgeCandidate {
    time: f32,
    frame: usize,
    player_id: subtr_actor::PlayerId,
    player_name: String,
    is_team_0: bool,
    score: f32,
    wall_contact_time: f32,
    time_since_wall_contact: f32,
}

#[derive(Default)]
struct CandidateScanner {
    strict_tracker: subtr_actor::FlipResetTracker,
    exact_dodge_refreshed_events: Vec<subtr_actor::DodgeRefreshedEvent>,
    loose_flip_reset_candidates: Vec<LooseFlipResetCandidate>,
    loose_post_wall_dodge_candidates: Vec<LoosePostWallDodgeCandidate>,
    recent_wall_contact_time: HashMap<subtr_actor::PlayerId, f32>,
    previous_dodge_active: HashMap<subtr_actor::PlayerId, bool>,
}

impl CandidateScanner {
    fn estimated_scale_factor(
        ball_body: &boxcars::RigidBody,
        player_body: &boxcars::RigidBody,
    ) -> f32 {
        let max_horizontal = [
            ball_body.location.x.abs(),
            ball_body.location.y.abs(),
            player_body.location.x.abs(),
            player_body.location.y.abs(),
        ]
        .into_iter()
        .fold(0.0, f32::max);

        if max_horizontal < 200.0 {
            100.0
        } else {
            1.0
        }
    }

    fn scaled_vec3(location: &boxcars::Vector3f, scale_factor: f32) -> glam::Vec3 {
        glam::Vec3::new(location.x, location.y, location.z) * scale_factor
    }

    fn glam_quat(rotation: &boxcars::Quaternion) -> glam::Quat {
        glam::Quat::from_xyzw(rotation.x, rotation.y, rotation.z, rotation.w)
    }

    fn loose_flip_reset_candidate(
        processor: &ReplayProcessor,
        touch_event: &subtr_actor::TouchEvent,
    ) -> Option<LooseFlipResetCandidate> {
        let player_id = touch_event.player.as_ref()?.clone();
        let closest_approach_distance = touch_event.closest_approach_distance?;
        let ball_body = processor.get_ball_rigid_body().ok()?;
        let player_body = processor.get_player_rigid_body(&player_id).ok()?;
        let scale_factor = Self::estimated_scale_factor(ball_body, player_body);
        let player_position = Self::scaled_vec3(&player_body.location, scale_factor);
        let ball_position = Self::scaled_vec3(&ball_body.location, scale_factor);
        let relative_ball_position = ball_position - player_position;
        let center_distance = relative_ball_position.length();
        if !center_distance.is_finite() || center_distance <= 30.0 || center_distance >= 550.0 {
            return None;
        }

        let player_rotation = Self::glam_quat(&player_body.rotation);
        let local_ball_position = player_rotation.inverse() * relative_ball_position;
        let car_up = (player_rotation * glam::Vec3::Z).normalize_or_zero();
        let underside_alignment = (-car_up).dot(relative_ball_position.normalize_or_zero());
        let scaled_touch_distance = closest_approach_distance * scale_factor;

        let below_car_score = (-local_ball_position.z / 180.0).clamp(0.0, 1.0);
        let alignment_score = ((underside_alignment - 0.45) / 0.50).clamp(0.0, 1.0);
        let touch_score = (1.0 - ((scaled_touch_distance - 20.0) / 220.0)).clamp(0.0, 1.0);
        let height_score = ((player_position.z - 70.0) / 500.0).clamp(0.0, 1.0);
        let footprint_score = (1.0
            - (local_ball_position.x.abs() / 260.0).clamp(0.0, 1.0) * 0.5
            - (local_ball_position.y.abs() / 260.0).clamp(0.0, 1.0) * 0.5)
            .clamp(0.0, 1.0);
        let score = 0.28 * below_car_score
            + 0.26 * alignment_score
            + 0.20 * touch_score
            + 0.14 * height_score
            + 0.12 * footprint_score;

        if score < 0.45 || local_ball_position.z >= 20.0 || underside_alignment < 0.25 {
            return None;
        }

        Some(LooseFlipResetCandidate {
            time: touch_event.time,
            frame: touch_event.frame,
            player_name: processor
                .get_player_name(&player_id)
                .unwrap_or_else(|_| "<unknown>".to_owned()),
            is_team_0: touch_event.team_is_team_0,
            player_id,
            score,
            closest_approach_distance: scaled_touch_distance,
            local_ball_position,
            underside_alignment,
            player_height: player_position.z,
            ball_height: ball_position.z,
        })
    }

    fn is_grounded_for_wall_sequence(player_body: &boxcars::RigidBody, scale_factor: f32) -> bool {
        player_body.location.z * scale_factor <= 80.0
    }

    fn is_touching_wall(player_body: &boxcars::RigidBody, scale_factor: f32) -> bool {
        let x = player_body.location.x.abs() * scale_factor;
        let y = player_body.location.y.abs() * scale_factor;
        let z = player_body.location.z * scale_factor;
        z >= 120.0 && (x >= 3600.0 || y >= 5000.0)
    }

    fn consider_loose_post_wall_dodge(
        &mut self,
        processor: &ReplayProcessor,
        frame_index: usize,
        current_time: f32,
    ) {
        for player_id in processor.iter_player_ids_in_order() {
            let Ok(player_body) = processor.get_player_rigid_body(player_id) else {
                self.previous_dodge_active.remove(player_id);
                self.recent_wall_contact_time.remove(player_id);
                continue;
            };
            let scale_factor = Self::estimated_scale_factor(
                processor.get_ball_rigid_body().ok().unwrap_or(player_body),
                player_body,
            );
            let is_grounded = Self::is_grounded_for_wall_sequence(player_body, scale_factor);
            let is_touching_wall = Self::is_touching_wall(player_body, scale_factor);

            if is_grounded {
                self.recent_wall_contact_time.remove(player_id);
            } else if is_touching_wall {
                self.recent_wall_contact_time
                    .insert(player_id.clone(), current_time);
            }

            let dodge_active = processor.get_dodge_active(player_id).unwrap_or(0) % 2 == 1;
            let was_dodge_active = self
                .previous_dodge_active
                .insert(player_id.clone(), dodge_active)
                .unwrap_or(false);
            if !dodge_active || was_dodge_active || is_grounded || is_touching_wall {
                continue;
            }

            let Some(wall_contact_time) = self.recent_wall_contact_time.get(player_id).copied()
            else {
                continue;
            };
            let time_since_wall_contact = current_time - wall_contact_time;
            if !(0.12..=1.35).contains(&time_since_wall_contact) {
                continue;
            }

            let timing_score = if time_since_wall_contact < 0.30 {
                (time_since_wall_contact - 0.12) / 0.18
            } else if time_since_wall_contact <= 0.95 {
                1.0
            } else {
                1.0 - ((time_since_wall_contact - 0.95) / 0.40)
            }
            .clamp(0.0, 1.0);
            let height_score =
                ((player_body.location.z * scale_factor - 120.0) / 500.0).clamp(0.0, 1.0);
            let score = 0.65 * timing_score + 0.35 * height_score;
            if score < 0.45 {
                continue;
            }

            self.loose_post_wall_dodge_candidates
                .push(LoosePostWallDodgeCandidate {
                    time: current_time,
                    frame: frame_index,
                    player_id: player_id.clone(),
                    player_name: processor
                        .get_player_name(player_id)
                        .unwrap_or_else(|_| "<unknown>".to_owned()),
                    is_team_0: processor.get_player_is_team_0(player_id).unwrap_or(false),
                    score,
                    wall_contact_time,
                    time_since_wall_contact,
                });
        }
    }
}

impl Collector for CandidateScanner {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_index: usize,
        current_time: f32,
    ) -> subtr_actor::SubtrActorResult<TimeAdvance> {
        self.strict_tracker
            .on_frame(processor, frame, frame_index)?;
        self.exact_dodge_refreshed_events.extend(
            processor
                .current_frame_dodge_refreshed_events()
                .iter()
                .cloned(),
        );

        for touch_event in processor.current_frame_touch_events() {
            let Some(candidate) = Self::loose_flip_reset_candidate(processor, touch_event) else {
                continue;
            };
            self.loose_flip_reset_candidates.push(candidate);
        }

        self.consider_loose_post_wall_dodge(processor, frame_index, current_time);
        Ok(TimeAdvance::NextFrame)
    }
}

fn normalize_replay_id(input: &str) -> &str {
    input
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(input)
        .split('?')
        .next()
        .unwrap_or(input)
}

fn fetch_public_ballchasing_replay(input: &str) -> Result<(String, Vec<u8>)> {
    let replay_id = normalize_replay_id(input).to_owned();
    let url = format!("https://ballchasing.com/dl/replay/{replay_id}");
    let client = Client::builder().build()?;
    let response = client
        .post(&url)
        .send()
        .with_context(|| format!("Failed to fetch {url}"))?
        .error_for_status()
        .with_context(|| format!("Ballchasing returned an error for {url}"))?;
    let replay_bytes = response
        .bytes()
        .with_context(|| format!("Failed to read replay bytes from {url}"))?;
    Ok((replay_id, replay_bytes.to_vec()))
}

fn replay_name_from_path(path: &Path) -> String {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "replay.replay")
    {
        if let Some(parent_name) = path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
        {
            return parent_name.to_owned();
        }
    }
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("replay")
        .to_owned()
}

fn scan_replay(label: &str, replay: &boxcars::Replay) -> Result<()> {
    let scanner = CandidateScanner::default()
        .process_replay(replay)
        .map_err(|error| anyhow::Error::new(error.variant))?;
    println!("== {label} ==");
    println!(
        "exact dodge refreshes: {}, strict flip resets: {}, strict post-wall dodges: {}, strict followup dodges: {}",
        scanner.exact_dodge_refreshed_events.len(),
        scanner.strict_tracker.flip_reset_events().len(),
        scanner.strict_tracker.post_wall_dodge_events().len(),
        scanner
            .strict_tracker
            .flip_reset_followup_dodge_events()
            .len()
    );

    let mut loose_flip_candidates = scanner.loose_flip_reset_candidates;
    loose_flip_candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.time.partial_cmp(&b.time).unwrap_or(Ordering::Equal))
    });
    println!("top loose flip-reset candidates:");
    for candidate in loose_flip_candidates.iter().take(5) {
        println!(
            "  t={:.2} frame={} player={} team={} score={:.3} touch={:.1} align={:.3} local=({:.1},{:.1},{:.1}) heights=({:.1},{:.1}) id={:?}",
            candidate.time,
            candidate.frame,
            candidate.player_name,
            if candidate.is_team_0 { 0 } else { 1 },
            candidate.score,
            candidate.closest_approach_distance,
            candidate.underside_alignment,
            candidate.local_ball_position.x,
            candidate.local_ball_position.y,
            candidate.local_ball_position.z,
            candidate.player_height,
            candidate.ball_height,
            candidate.player_id
        );
    }

    let mut loose_post_wall_candidates = scanner.loose_post_wall_dodge_candidates;
    loose_post_wall_candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.time.partial_cmp(&b.time).unwrap_or(Ordering::Equal))
    });
    println!("top loose post-wall dodge candidates:");
    for candidate in loose_post_wall_candidates.iter().take(5) {
        println!(
            "  t={:.2} frame={} player={} team={} score={:.3} dt={:.3} wall_t={:.2} id={:?}",
            candidate.time,
            candidate.frame,
            candidate.player_name,
            if candidate.is_team_0 { 0 } else { 1 },
            candidate.score,
            candidate.time_since_wall_contact,
            candidate.wall_contact_time,
            candidate.player_id
        );
    }
    println!();
    Ok(())
}

fn main() -> Result<()> {
    let inputs: Vec<String> = std::env::args().skip(1).collect();
    if inputs.is_empty() {
        anyhow::bail!(
            "Usage: cargo run --bin scan_flip_reset_candidates -- <replay-path-or-ballchasing-id-or-url>..."
        );
    }

    for input in &inputs {
        if Path::new(input).exists() {
            let replay = parse_replay_file(input)?;
            scan_replay(&replay_name_from_path(Path::new(input)), &replay)?;
            continue;
        }

        let (replay_id, replay_bytes) = fetch_public_ballchasing_replay(input)?;
        let replay = parse_replay_bytes(&replay_bytes)?;
        scan_replay(&format!("ballchasing:{replay_id}"), &replay)?;
    }

    Ok(())
}
