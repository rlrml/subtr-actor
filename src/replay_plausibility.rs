use crate::collector::replay_data::{BallFrame, PlayerFrame, ReplayData};
use crate::geometry::{quat_to_glam, vec_to_glam};

#[path = "replay_plausibility_accumulator.rs"]
mod replay_plausibility_accumulator;
#[path = "replay_plausibility_constants.rs"]
mod replay_plausibility_constants;
#[path = "replay_plausibility_report.rs"]
mod replay_plausibility_report;
#[path = "replay_plausibility_stats.rs"]
mod replay_plausibility_stats;

use replay_plausibility_accumulator::RigidBodyPlausibilityAccumulator;
pub use replay_plausibility_report::{ReplayPlausibilityReport, RigidBodyPlausibilityReport};

pub fn evaluate_replay_plausibility(replay_data: &ReplayData) -> ReplayPlausibilityReport {
    let mut ball = RigidBodyPlausibilityAccumulator::default();
    let mut players = RigidBodyPlausibilityAccumulator::default();
    let times: Vec<f32> = replay_data
        .frame_data
        .metadata_frames
        .iter()
        .map(|frame| frame.time)
        .collect();

    add_ball_samples(replay_data, &times, &mut ball);
    add_player_samples(replay_data, &times, &mut players);

    ReplayPlausibilityReport {
        ball: ball.finish(),
        players: players.finish(),
    }
}

fn add_ball_samples(
    replay_data: &ReplayData,
    times: &[f32],
    ball: &mut RigidBodyPlausibilityAccumulator,
) {
    let mut previous_ball: Option<(f32, &boxcars::RigidBody)> = None;
    for (time, frame) in times
        .iter()
        .copied()
        .zip(replay_data.frame_data.ball_data.frames())
    {
        if let BallFrame::Data { rigid_body } = frame {
            ball.add_sample(rigid_body);
            if let Some((previous_time, previous_rigid_body)) = previous_ball {
                ball.add_pair(previous_time, previous_rigid_body, time, rigid_body);
            }
            previous_ball = Some((time, rigid_body));
        }
    }
}

fn add_player_samples(
    replay_data: &ReplayData,
    times: &[f32],
    players: &mut RigidBodyPlausibilityAccumulator,
) {
    for (_, player_data) in &replay_data.frame_data.players {
        let mut previous_player: Option<(f32, &boxcars::RigidBody)> = None;
        for (time, frame) in times.iter().copied().zip(player_data.frames()) {
            if let PlayerFrame::Data { rigid_body, .. } = frame {
                players.add_sample(rigid_body);
                if let Some((previous_time, previous_rigid_body)) = previous_player {
                    players.add_pair(previous_time, previous_rigid_body, time, rigid_body);
                }
                previous_player = Some((time, rigid_body));
            }
        }
    }
}
