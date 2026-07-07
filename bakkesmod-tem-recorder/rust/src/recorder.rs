//! Safe pack-recording layer the FFI functions are thin wrappers over.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use subtr_actor_training::{Difficulty, Guid, Round, TrainingFile, TrainingPack, TrainingType};

use crate::abi::{TrBallState, TrCarState};
use crate::archetypes::build_round_archetypes;

/// Default pack name for freshly created packs.
pub const DEFAULT_PACK_NAME: &str = "TEM Recorder Pack";
/// Default map for freshly created packs, matching the corpus fixtures.
/// TODO(phase-3): confirm which map names custom training accepts.
pub const DEFAULT_MAP_NAME: &str = "Park_P";

/// An in-memory training pack being assembled by the plugin.
///
/// Wraps a [`TrainingFile`] (authoritative, so packs opened from disk keep
/// every property this crate does not model) plus the last error message
/// for ABI error reporting.
pub struct RecorderPack {
    file: TrainingFile,
    last_error: String,
}

/// Extracts a top-level numeric field from a flat single-line archetype
/// JSON string without a JSON dependency (the strings are machine-written
/// `"key":value` pairs with no nesting or whitespace).
fn json_number_field(archetype: &str, key: &str) -> Option<f64> {
    let marker = format!("\"{key}\":");
    let start = archetype.find(&marker)? + marker.len();
    let rest = &archetype[start..];
    let end = rest
        .find(|character: char| character == ',' || character == '}')
        .unwrap_or(rest.len());
    rest[..end].trim().parse().ok()
}

fn unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// Generates a pseudo-random GUID from the clock and process id via
/// splitmix64 (no `rand` dependency; uniqueness, not unpredictability, is
/// what matters here).
fn generate_guid() -> Guid {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);
    let mut state = nanos ^ (u64::from(std::process::id()) << 32);
    let mut next = move || {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    };
    let first = next();
    let second = next();
    Guid {
        a: (first >> 32) as i32,
        b: first as i32,
        c: (second >> 32) as i32,
        d: second as i32,
    }
}

impl RecorderPack {
    /// Creates a fresh pack with a generated GUID, current timestamps, and
    /// corpus-matching defaults.
    pub fn new() -> RecorderPack {
        let now = unix_time();
        let pack = TrainingPack {
            guid: generate_guid(),
            name: Some(DEFAULT_PACK_NAME.to_string()),
            training_type: TrainingType::Striker,
            difficulty: Difficulty::Medium,
            map_name: Some(DEFAULT_MAP_NAME.to_string()),
            created_at: now,
            updated_at: now,
            ..TrainingPack::default()
        };
        let file = TrainingFile::from_pack(&pack)
            .expect("constructing a fresh training file from a typed pack cannot fail");
        RecorderPack {
            file,
            last_error: String::new(),
        }
    }

    /// Opens an existing `.tem` file so new shots append to it.
    pub fn open(path: &Path) -> Result<RecorderPack, String> {
        let bytes = std::fs::read(path)
            .map_err(|error| format!("could not read {}: {error}", path.display()))?;
        let file = TrainingFile::from_bytes(&bytes)
            .map_err(|error| format!("could not parse {}: {error}", path.display()))?;
        // Fail early if the file has no usable training data object.
        file.pack()
            .map_err(|error| format!("not a training pack: {error}"))?;
        Ok(RecorderPack {
            file,
            last_error: String::new(),
        })
    }

    /// The typed view of the underlying pack.
    pub fn pack(&self) -> Result<TrainingPack, String> {
        self.file.pack().map_err(|error| error.to_string())
    }

    /// Records `error` as the pack's last error and returns it.
    pub fn record_error(&mut self, error: String) -> &str {
        self.last_error = error;
        &self.last_error
    }

    /// The last error message recorded on this pack (empty when none).
    pub fn last_error(&self) -> &str {
        &self.last_error
    }

    pub fn set_name(&mut self, name: Option<&str>) -> Result<(), String> {
        self.file.set_name(name).map_err(|error| error.to_string())
    }

    pub fn set_code(&mut self, code: Option<&str>) -> Result<(), String> {
        self.file.set_code(code).map_err(|error| error.to_string())
    }

    pub fn set_creator_name(&mut self, creator_name: Option<&str>) -> Result<(), String> {
        self.file
            .set_creator_name(creator_name)
            .map_err(|error| error.to_string())
    }

    pub fn set_map_name(&mut self, map_name: &str) -> Result<(), String> {
        self.file
            .set_map_name(map_name)
            .map_err(|error| error.to_string())
    }

    pub fn set_difficulty(&mut self, difficulty: &Difficulty) -> Result<(), String> {
        self.file
            .set_difficulty(difficulty)
            .map_err(|error| error.to_string())
    }

    /// Appends a captured shot as a new round.
    pub fn add_shot(
        &mut self,
        ball: &TrBallState,
        cars: &[TrCarState],
        time_limit: f32,
    ) -> Result<(), String> {
        let round = Round {
            time_limit,
            serialized_archetypes: build_round_archetypes(ball, cars),
        };
        self.file
            .add_round(&round)
            .map_err(|error| error.to_string())
    }

    /// Removes the shot (round) at `index`.
    pub fn remove_shot(&mut self, index: usize) -> Result<(), String> {
        self.file
            .remove_round(index)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    /// Number of shots (rounds) currently in the pack, including any that
    /// were already present in an opened file.
    pub fn shot_count(&self) -> usize {
        self.file.rounds().map(|rounds| rounds.len()).unwrap_or(0)
    }

    /// A short human-readable summary of the shot at `index` for the
    /// settings-window list, e.g. `ball (24, 4269, 224), 1 car, 10s`.
    pub fn shot_summary(&self, index: usize) -> Option<String> {
        let rounds = self.file.rounds().ok()?;
        let round = rounds.get(index)?;
        let mut ball_location = None;
        let mut car_count = 0usize;
        for archetype in &round.serialized_archetypes {
            if archetype.contains("\"IsPC\"") {
                car_count += 1;
            } else if archetype.contains("Ball_GameEditor") {
                if let (Some(x), Some(y), Some(z)) = (
                    json_number_field(archetype, "StartLocationX"),
                    json_number_field(archetype, "StartLocationY"),
                    json_number_field(archetype, "StartLocationZ"),
                ) {
                    ball_location = Some((x, y, z));
                }
            }
        }
        let ball_text = match ball_location {
            Some((x, y, z)) => format!("ball ({x:.0}, {y:.0}, {z:.0})"),
            None => "no ball".to_string(),
        };
        let car_text = if car_count == 1 { "car" } else { "cars" };
        Some(format!(
            "{ball_text}, {car_count} {car_text}, {:.0}s",
            round.time_limit
        ))
    }

    /// The pack GUID as 32 uppercase hex characters, matching the game's
    /// `.Tem` filename convention.
    pub fn guid_hex(&self) -> String {
        let guid = self.file.pack().map(|pack| pack.guid).unwrap_or_default();
        format!(
            "{:08X}{:08X}{:08X}{:08X}",
            guid.a as u32, guid.b as u32, guid.c as u32, guid.d as u32
        )
    }

    /// Serializes, encrypts, and writes the pack to `path`, creating parent
    /// directories and refreshing `UpdatedAt`.
    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        self.file
            .set_updated_at(unix_time())
            .map_err(|error| error.to_string())?;
        let bytes = self
            .file
            .to_bytes()
            .map_err(|error| format!("could not serialize pack: {error}"))?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
            }
        }
        std::fs::write(path, bytes)
            .map_err(|error| format!("could not write {}: {error}", path.display()))
    }
}

impl Default for RecorderPack {
    fn default() -> Self {
        RecorderPack::new()
    }
}

#[cfg(test)]
#[path = "recorder_tests.rs"]
mod tests;
