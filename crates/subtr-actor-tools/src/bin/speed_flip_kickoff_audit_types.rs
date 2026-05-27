use clap::Parser;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct Audit {
    pub(crate) replays: Vec<ReplayAudit>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReplayAudit {
    pub(crate) path: String,
    pub(crate) kickoff_count: usize,
    pub(crate) team_kickoff_opportunities: usize,
    pub(crate) detected_team_kickoffs: usize,
    pub(crate) speed_flip_event_count: usize,
    pub(crate) kickoffs: Vec<KickoffAudit>,
}

#[derive(Debug, Serialize)]
pub(crate) struct KickoffAudit {
    pub(crate) index: usize,
    pub(crate) start_time: f32,
    pub(crate) start_frame: usize,
    pub(crate) blue_front_players: Vec<String>,
    pub(crate) orange_front_players: Vec<String>,
    pub(crate) blue_detected: Vec<DetectedSpeedFlip>,
    pub(crate) orange_detected: Vec<DetectedSpeedFlip>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DetectedSpeedFlip {
    pub(crate) player: String,
    pub(crate) time: f32,
    pub(crate) time_since_kickoff_start: f32,
    pub(crate) confidence: f32,
    pub(crate) diagonal_score: f32,
    pub(crate) cancel_score: f32,
    pub(crate) speed_score: f32,
    pub(crate) max_speed: f32,
}

#[derive(Debug, Parser)]
#[command(about = "Audit speed-flip detections during replay kickoffs.")]
pub(crate) struct Args {
    /// Replay paths to audit.
    #[arg(value_name = "replay", num_args = 1..)]
    pub(crate) paths: Vec<String>,
}
