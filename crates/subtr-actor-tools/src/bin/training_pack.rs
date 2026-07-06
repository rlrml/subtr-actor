//! CLI for Rocket League custom training pack (`.tem`) files.
//!
//! Subcommands:
//! * `decode <file.tem> [-o out.json] [--pretty] [--no-decrypt]` — decode to
//!   the lossless JSON representation.
//! * `encode <file.json> [-o out.tem] [--no-encrypt]` — re-encode JSON back
//!   to a `.tem`.
//! * `info <file.tem>` — print name/code/creator/round count.
//! * `roundtrip <file.tem>` — decode and re-encode, byte-comparing both the
//!   decrypted payloads and the full file.

use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};
use subtr_actor_training::TrainingFile;

#[derive(Debug, Parser)]
#[command(
    about = "Decode, encode, inspect, and round-trip Rocket League training pack (.tem) files."
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Decode a .tem file to lossless JSON.
    Decode {
        /// Path to the .tem (or already-decrypted payload) file.
        input: PathBuf,
        /// Output JSON path (defaults to stdout).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Pretty-print the JSON.
        #[arg(long)]
        pretty: bool,
        /// Treat the input as an already-decrypted payload (no envelope).
        #[arg(long)]
        no_decrypt: bool,
        /// Output the typed TrainingPack view instead of the lossless
        /// container JSON (not accepted by `encode`).
        #[arg(long)]
        typed: bool,
    },
    /// Encode a JSON file (as produced by `decode`) back to a .tem file.
    Encode {
        /// Path to the JSON file.
        input: PathBuf,
        /// Output .tem path (defaults to the input with a .tem extension).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Write the decrypted payload instead of the encrypted envelope.
        #[arg(long)]
        no_encrypt: bool,
    },
    /// Print a short summary of a training pack.
    Info {
        /// Path to the .tem file.
        input: PathBuf,
        /// Treat the input as an already-decrypted payload (no envelope).
        #[arg(long)]
        no_decrypt: bool,
    },
    /// Decode and re-encode, verifying byte-for-byte fidelity.
    Roundtrip {
        /// Path to the .tem or .save file.
        input: PathBuf,
        /// Treat the input as an already-decrypted payload (no envelope).
        #[arg(long)]
        no_decrypt: bool,
    },
}

fn load(path: &Path, no_decrypt: bool) -> anyhow::Result<TrainingFile> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let file = if no_decrypt {
        TrainingFile::from_decrypted_payload(&bytes)
    } else {
        TrainingFile::from_bytes(&bytes)
    };
    file.with_context(|| format!("parsing {}", path.display()))
}

fn first_difference(a: &[u8], b: &[u8]) -> Option<usize> {
    if a == b {
        return None;
    }
    Some(
        a.iter()
            .zip(b.iter())
            .position(|(x, y)| x != y)
            .unwrap_or_else(|| a.len().min(b.len())),
    )
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Decode {
            input,
            output,
            pretty,
            no_decrypt,
            typed,
        } => {
            let file = load(&input, no_decrypt)?;
            let json = if typed {
                let pack = file.pack()?;
                if pretty {
                    serde_json::to_string_pretty(&pack)?
                } else {
                    serde_json::to_string(&pack)?
                }
            } else {
                file.to_json(pretty)?
            };
            match output {
                Some(path) => std::fs::write(&path, json)
                    .with_context(|| format!("writing {}", path.display()))?,
                None => println!("{json}"),
            }
        }
        Command::Encode {
            input,
            output,
            no_encrypt,
        } => {
            let json = std::fs::read_to_string(&input)
                .with_context(|| format!("reading {}", input.display()))?;
            let file = TrainingFile::from_json(&json)?;
            let bytes = if no_encrypt {
                file.to_decrypted_payload()?
            } else {
                file.to_bytes()?
            };
            let path = output.unwrap_or_else(|| input.with_extension("tem"));
            std::fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
            println!("wrote {}", path.display());
        }
        Command::Info { input, no_decrypt } => {
            let file = load(&input, no_decrypt)?;
            let pack = file.pack()?;
            println!("name:       {}", pack.name.as_deref().unwrap_or("<none>"));
            println!("code:       {}", pack.code.as_deref().unwrap_or("<none>"));
            println!(
                "creator:    {}",
                pack.creator_name.as_deref().unwrap_or("<none>")
            );
            println!(
                "map:        {}",
                pack.map_name.as_deref().unwrap_or("<none>")
            );
            println!("type:       {}", pack.training_type.as_name());
            println!("difficulty: {}", pack.difficulty.as_name());
            println!("rounds:     {}", pack.rounds.len());
        }
        Command::Roundtrip { input, no_decrypt } => {
            let original =
                std::fs::read(&input).with_context(|| format!("reading {}", input.display()))?;
            let file = if no_decrypt {
                TrainingFile::from_decrypted_payload(&original)
            } else {
                TrainingFile::from_bytes(&original)
            }
            .with_context(|| format!("parsing {}", input.display()))?;

            if no_decrypt {
                let reencoded = file.to_decrypted_payload()?;
                // The stored payload may carry AES zero padding; compare up
                // to the structural length and require zero padding beyond.
                let structural = &original[..reencoded.len().min(original.len())];
                match first_difference(structural, &reencoded) {
                    None if original[reencoded.len()..].iter().all(|&byte| byte == 0) => {
                        println!("payload roundtrip OK ({} bytes)", reencoded.len());
                    }
                    None => bail!("payload matches but trailing bytes are not zero padding"),
                    Some(offset) => bail!(
                        "payload mismatch at offset {offset} (original {} bytes, reencoded {} bytes)",
                        original.len(),
                        reencoded.len()
                    ),
                }
                return Ok(());
            }

            // Compare decrypted payloads (padded) and the full file.
            let ciphertext = &original[8..];
            let decrypted = subtr_actor_training::crypto::decrypt(ciphertext)?;
            let mut reencoded_payload = file.to_decrypted_payload()?;
            reencoded_payload.resize(decrypted.len(), 0);
            match first_difference(&decrypted, &reencoded_payload) {
                None => println!("decrypted payload roundtrip OK ({} bytes)", decrypted.len()),
                Some(offset) => bail!(
                    "decrypted payload mismatch at offset {offset} of {}",
                    decrypted.len()
                ),
            }

            let reencoded = file.to_bytes()?;
            match first_difference(&original, &reencoded) {
                None => {
                    println!("full file roundtrip OK ({} bytes)", original.len());
                }
                Some(offset) => bail!(
                    "full file mismatch at offset {offset} (original {} bytes, reencoded {} bytes)",
                    original.len(),
                    reencoded.len()
                ),
            }
        }
    }
    Ok(())
}
