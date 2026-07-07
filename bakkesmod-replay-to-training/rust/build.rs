//! Embeds build identification (git hash, dirty flag, commit date) into the
//! Rust core DLL so `replay_to_training_version` can report which build is
//! loaded, mirroring the CMake-side plumbing for the C++ plugin DLL.
//!
//! Sources, in priority order:
//! 1. `REPLAY_TO_TRAINING_GIT_HASH` / `REPLAY_TO_TRAINING_GIT_DIRTY` /
//!    `REPLAY_TO_TRAINING_COMMIT_DATE` environment variables (exported by
//!    the nix build, whose sandbox has no `.git`),
//! 2. `git` at build time (local and CI builds from a checkout),
//! 3. `"unknown"` / `"0"` fallbacks, so the build never fails.

use std::process::Command;

fn env_override(name: &str) -> Option<String> {
    println!("cargo:rerun-if-env-changed={name}");
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

fn git_output(args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let text = text.trim().to_string();
    (!text.is_empty()).then_some(text)
}

fn git_dirty() -> Option<String> {
    let status = Command::new("git")
        .args(["diff", "--quiet", "--ignore-submodules", "HEAD", "--"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .ok()?;
    match status.code() {
        Some(0) => Some("0".to_string()),
        Some(1) => Some("1".to_string()),
        _ => None,
    }
}

fn main() {
    let hash = env_override("REPLAY_TO_TRAINING_GIT_HASH")
        .or_else(|| git_output(&["rev-parse", "--short", "HEAD"]))
        .unwrap_or_else(|| "unknown".to_string());
    let dirty = env_override("REPLAY_TO_TRAINING_GIT_DIRTY")
        .or_else(git_dirty)
        .unwrap_or_else(|| "0".to_string());
    let commit_date = env_override("REPLAY_TO_TRAINING_COMMIT_DATE")
        .or_else(|| git_output(&["show", "-s", "--format=%cI", "HEAD"]))
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=REPLAY_TO_TRAINING_GIT_HASH={hash}");
    println!("cargo:rustc-env=REPLAY_TO_TRAINING_GIT_DIRTY={dirty}");
    println!("cargo:rustc-env=REPLAY_TO_TRAINING_COMMIT_DATE={commit_date}");
}
