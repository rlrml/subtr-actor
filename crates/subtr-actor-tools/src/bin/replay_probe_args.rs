use clap::{Parser, ValueEnum};

use super::constants::{DEFAULT_DEMOLITION_REPLAY_PATH, DEFAULT_REPLAY_PATH};

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub(crate) enum ProbeCommand {
    Metadata,
    Plausibility,
    LegacyRotation,
    Demolition,
    VectorRanges,
    Mechanics,
}

impl ProbeCommand {
    fn default_path(self) -> &'static str {
        match self {
            Self::Demolition => DEFAULT_DEMOLITION_REPLAY_PATH,
            Self::Metadata
            | Self::Plausibility
            | Self::LegacyRotation
            | Self::VectorRanges
            | Self::Mechanics => DEFAULT_REPLAY_PATH,
        }
    }
}

#[derive(Debug, Parser)]
#[command(about = "Probe replay metadata, plausibility, rotation, demolition, and vector ranges.")]
struct Args {
    /// Probe to run.
    command: ProbeCommand,

    /// Replay path. Defaults to a built-in fixture for the selected probe.
    replay_path: Option<String>,
}

pub(crate) fn parse_args() -> (ProbeCommand, String) {
    let Args {
        command,
        replay_path,
    } = Args::parse();
    let path = replay_path.unwrap_or_else(|| command.default_path().to_string());
    (command, path)
}
