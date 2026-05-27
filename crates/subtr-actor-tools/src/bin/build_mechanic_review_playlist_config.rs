use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Context};
use clap::Parser;

use super::args::Args;
use super::constants::ALL_MECHANICS;

#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) ids: Vec<String>,
    pub(crate) replay_paths: Vec<PathBuf>,
    pub(crate) ids_file: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
    pub(crate) cache_dir: PathBuf,
    pub(crate) count: usize,
    pub(crate) playlist: String,
    pub(crate) sort_by: String,
    pub(crate) sort_dir: String,
    pub(crate) query_params: Vec<(String, String)>,
    pub(crate) min_confidence: f32,
    pub(crate) before_seconds: f32,
    pub(crate) after_seconds: f32,
    pub(crate) goal_lookahead_seconds: f32,
    pub(crate) goal_tail_seconds: f32,
    pub(crate) min_clip_seconds: f32,
    pub(crate) max_items: Option<usize>,
    pub(crate) download_delay: Duration,
    pub(crate) mechanics: Vec<String>,
}

impl Config {
    fn from_args(args: Args) -> anyhow::Result<Self> {
        if args.list_mechanics {
            println!("{}", ALL_MECHANICS.join("\n"));
            std::process::exit(0);
        }

        let mut mechanics = args.mechanic;
        mechanics.extend(args.mechanics);

        let config = Self {
            ids: args.ids,
            replay_paths: args.replay_paths,
            ids_file: args.ids_file,
            output: args.output,
            cache_dir: args.cache_dir,
            count: args.count,
            playlist: args.playlist,
            sort_by: args.sort_by,
            sort_dir: args.sort_dir,
            query_params: args.query_params,
            min_confidence: args.min_confidence,
            before_seconds: args.before_seconds,
            after_seconds: args.after_seconds,
            goal_lookahead_seconds: args.goal_lookahead_seconds,
            goal_tail_seconds: args.goal_tail_seconds,
            min_clip_seconds: args.min_clip_seconds,
            max_items: args.max_items,
            download_delay: Duration::from_millis(args.download_delay_ms),
            mechanics,
        };

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> anyhow::Result<()> {
        if self.count == 0 {
            bail!("--count must be at least 1");
        }
        if self.before_seconds < 0.0
            || self.after_seconds < 0.0
            || self.goal_lookahead_seconds < 0.0
            || self.goal_tail_seconds < 0.0
            || self.min_clip_seconds < 0.0
        {
            bail!("clip padding must be non-negative");
        }
        Ok(())
    }
}

pub(crate) fn parse_args() -> anyhow::Result<Config> {
    Config::from_args(Args::parse()).context("failed to parse mechanic playlist arguments")
}
