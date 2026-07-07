//! WASM bindings for Rocket League custom training pack (`.tem`) files,
//! backed by the `subtr-actor-training` crate.
//!
//! # Lossless interchange representation
//!
//! Editing entry points exchange a *lossless* representation of a training
//! file: the JSON string form of [`subtr_actor_training::TrainingFile`]
//! (`TrainingFile::to_json` / `TrainingFile::from_json`). This keeps every
//! unknown property and object type from the original file, so a
//! parse -> edit -> serialize round trip only changes the fields that were
//! explicitly edited. A JSON string was chosen over a structured `JsValue`
//! because it crosses the WASM boundary on every mutation: it is cheap to
//! pass, needs no bigint/number massaging, and is stable to hold on the JS
//! side as an opaque token.
//!
//! # How typed edits preserve unknown data
//!
//! * [`update_training_pack_metadata`] applies only the scalar/metadata
//!   fields of the typed pack through the crate's in-place setters, which
//!   write into the existing property tree; unknown file-level and
//!   round-level properties are untouched. It deliberately does **not**
//!   rebuild the `Rounds` array (converting typed rounds back to property
//!   lists would drop unknown per-round properties) and does not write
//!   `CreatorPlayerID` (read-only in the underlying crate; its unknown
//!   subfields such as `NpId` survive in the tree).
//! * The round operations ([`training_pack_remove_round`],
//!   [`training_pack_move_round`], [`training_pack_duplicate_round`],
//!   [`training_pack_append_rounds`]) move whole round property lists, so
//!   unknown per-round properties survive. Only [`training_pack_add_round`]
//!   and [`training_pack_insert_round`] build a round from the typed
//!   representation — which is lossless for them because a *new* round has
//!   no unknown data to lose.
//!
//! 64-bit integer fields (`created_at`, `updated_at`, `creator_player_id.uid`)
//! are exposed to JS as `BigInt`, matching the generated TypeScript types
//! (Steam ids exceed `Number.MAX_SAFE_INTEGER`).

use serde::Serialize;
use subtr_actor_training::{Archetype, BallSpawn, Round, TrainingFile, TrainingPack};
use wasm_bindgen::prelude::*;

fn training_file_from_bytes(data: &[u8]) -> Result<TrainingFile, JsValue> {
    TrainingFile::from_bytes(data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse training pack: {e}")))
}

fn training_file_from_lossless(lossless: &str) -> Result<TrainingFile, JsValue> {
    TrainingFile::from_json(lossless)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse lossless training pack: {e}")))
}

fn lossless_from_training_file(file: &TrainingFile) -> Result<String, JsValue> {
    file.to_json(false)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize training file: {e}")))
}

fn typed_pack_to_js(pack: &TrainingPack) -> Result<JsValue, JsValue> {
    // `serialize_missing_as_null` keeps `Option::None` as `null` (not
    // `undefined`) so the runtime shape matches the generated TS types.
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_large_number_types_as_bigints(true)
        .serialize_missing_as_null(true);
    pack.serialize(&serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

fn typed_pack_from_js(typed_pack: JsValue) -> Result<TrainingPack, JsValue> {
    serde_wasm_bindgen::from_value(typed_pack)
        .map_err(|e| JsValue::from_str(&format!("Failed to read typed training pack: {e}")))
}

fn round_from_js(round: JsValue) -> Result<Round, JsValue> {
    serde_wasm_bindgen::from_value(round)
        .map_err(|e| JsValue::from_str(&format!("Failed to read training round: {e}")))
}

fn archetypes_to_js(archetypes: &[Archetype]) -> Result<JsValue, JsValue> {
    // `serialize_maps_as_objects` keeps each archetype's `extras` map a plain
    // JS object (matching the generated `Record<string, unknown>` type)
    // instead of an ES `Map`; `serialize_missing_as_null` keeps `Option`
    // fields as `null` to match the generated TS types.
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true)
        .serialize_missing_as_null(true);
    archetypes
        .serialize(&serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert archetypes to JS: {e}")))
}

fn archetype_from_js(archetype: JsValue) -> Result<Archetype, JsValue> {
    serde_wasm_bindgen::from_value(archetype)
        .map_err(|e| JsValue::from_str(&format!("Failed to read archetype: {e}")))
}

fn ball_from_js(ball: JsValue) -> Result<BallSpawn, JsValue> {
    serde_wasm_bindgen::from_value(ball)
        .map_err(|e| JsValue::from_str(&format!("Failed to read ball spawn: {e}")))
}

/// Parse an encrypted `.tem` file into the typed `TrainingPack` view.
#[wasm_bindgen]
pub fn parse_training_pack(data: &[u8]) -> Result<JsValue, JsValue> {
    let file = training_file_from_bytes(data)?;
    let pack = file
        .pack()
        .map_err(|e| JsValue::from_str(&format!("Failed to read training pack: {e}")))?;
    typed_pack_to_js(&pack)
}

/// Parse an encrypted `.tem` file into the lossless JSON representation
/// (see the module docs). This is the value every editing entry point takes
/// and returns.
#[wasm_bindgen]
pub fn parse_training_pack_lossless(data: &[u8]) -> Result<String, JsValue> {
    let file = training_file_from_bytes(data)?;
    lossless_from_training_file(&file)
}

/// Serialize a lossless representation back into encrypted `.tem` bytes.
#[wasm_bindgen]
pub fn serialize_training_pack(lossless: &str) -> Result<Vec<u8>, JsValue> {
    let file = training_file_from_lossless(lossless)?;
    file.to_bytes()
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize training pack: {e}")))
}

/// Build the typed `TrainingPack` view of a lossless representation.
#[wasm_bindgen]
pub fn training_pack_from_lossless(lossless: &str) -> Result<JsValue, JsValue> {
    let file = training_file_from_lossless(lossless)?;
    let pack = file
        .pack()
        .map_err(|e| JsValue::from_str(&format!("Failed to read training pack: {e}")))?;
    typed_pack_to_js(&pack)
}

/// Create a fresh lossless representation from a typed `TrainingPack`
/// (create-from-scratch; nothing unknown exists yet).
#[wasm_bindgen]
pub fn new_training_pack(typed_pack: JsValue) -> Result<String, JsValue> {
    let pack = typed_pack_from_js(typed_pack)?;
    let file = TrainingFile::from_pack(&pack)
        .map_err(|e| JsValue::from_str(&format!("Failed to build training pack: {e}")))?;
    lossless_from_training_file(&file)
}

/// Apply the metadata fields of a typed `TrainingPack` onto a lossless
/// representation, preserving unknown properties.
///
/// Applies: guid, code, name, training type, difficulty, creator name,
/// description, tags, map name (only when non-null; the underlying crate
/// cannot unset it), created/updated timestamps, player team number,
/// unowned, perfect-completed, and shots-completed.
///
/// Does **not** apply `rounds` (use the round operations, which preserve
/// per-round unknown properties) or `creator_player_id` (read-only).
#[wasm_bindgen]
pub fn update_training_pack_metadata(
    lossless: &str,
    typed_pack: JsValue,
) -> Result<String, JsValue> {
    let mut file = training_file_from_lossless(lossless)?;
    let pack = typed_pack_from_js(typed_pack)?;
    (|| -> subtr_actor_training::Result<()> {
        file.set_guid(pack.guid)?;
        file.set_code(pack.code.as_deref())?;
        file.set_name(pack.name.as_deref())?;
        file.set_training_type(&pack.training_type)?;
        file.set_difficulty(&pack.difficulty)?;
        file.set_creator_name(pack.creator_name.as_deref())?;
        file.set_description(pack.description.as_deref())?;
        file.set_tags(pack.tags.clone())?;
        if let Some(map_name) = &pack.map_name {
            file.set_map_name(map_name)?;
        }
        file.set_created_at(pack.created_at)?;
        file.set_updated_at(pack.updated_at)?;
        file.set_player_team_number(pack.player_team_number);
        file.set_unowned(pack.unowned);
        file.set_perfect_completed(pack.perfect_completed);
        file.set_shots_completed(pack.shots_completed);
        Ok(())
    })()
    .map_err(|e| JsValue::from_str(&format!("Failed to update training pack: {e}")))?;
    lossless_from_training_file(&file)
}

fn edit_training_file(
    lossless: &str,
    edit: impl FnOnce(&mut TrainingFile) -> subtr_actor_training::Result<()>,
) -> Result<String, JsValue> {
    let mut file = training_file_from_lossless(lossless)?;
    edit(&mut file)
        .map_err(|e| JsValue::from_str(&format!("Failed to edit training pack rounds: {e}")))?;
    lossless_from_training_file(&file)
}

/// Append a typed round to a lossless representation.
#[wasm_bindgen]
pub fn training_pack_add_round(lossless: &str, round: JsValue) -> Result<String, JsValue> {
    let round = round_from_js(round)?;
    edit_training_file(lossless, |file| file.add_round(&round))
}

/// Insert a typed round at `index` (clamped to the round count).
#[wasm_bindgen]
pub fn training_pack_insert_round(
    lossless: &str,
    index: usize,
    round: JsValue,
) -> Result<String, JsValue> {
    let round = round_from_js(round)?;
    edit_training_file(lossless, |file| file.insert_round(index, &round))
}

/// Remove the round at `index`.
#[wasm_bindgen]
pub fn training_pack_remove_round(lossless: &str, index: usize) -> Result<String, JsValue> {
    edit_training_file(lossless, |file| file.remove_round(index).map(|_| ()))
}

/// Move the round at `from` to position `to`.
#[wasm_bindgen]
pub fn training_pack_move_round(lossless: &str, from: usize, to: usize) -> Result<String, JsValue> {
    edit_training_file(lossless, |file| file.move_round(from, to))
}

/// Duplicate the round at `index`, inserting the copy right after it.
#[wasm_bindgen]
pub fn training_pack_duplicate_round(lossless: &str, index: usize) -> Result<String, JsValue> {
    edit_training_file(lossless, |file| file.duplicate_round(index))
}

/// Append every round of `other_lossless` to `lossless` (whole property
/// lists are copied, including unknown per-round properties).
#[wasm_bindgen]
pub fn training_pack_append_rounds(
    lossless: &str,
    other_lossless: &str,
) -> Result<String, JsValue> {
    let other = training_file_from_lossless(other_lossless)?;
    edit_training_file(lossless, |file| file.append_rounds_from(&other).map(|_| ()))
}

// --- round archetype editing ---
//
// Parsing is on demand and edits regenerate only the archetype string being
// modified, so untouched archetype strings stay byte-identical (see the
// `subtr_actor_training::archetype` module docs for the fidelity model).

/// Parse the archetypes of the round at `round_index` into an array of
/// typed `Archetype` values (a `kind`-tagged union; unrecognized strings
/// come back as `kind: "Unknown"` carrying the raw string verbatim).
#[wasm_bindgen]
pub fn training_pack_round_archetypes(
    lossless: &str,
    round_index: usize,
) -> Result<JsValue, JsValue> {
    let file = training_file_from_lossless(lossless)?;
    let archetypes = file
        .round_archetypes(round_index)
        .map_err(|e| JsValue::from_str(&format!("Failed to read round archetypes: {e}")))?;
    archetypes_to_js(&archetypes)
}

/// Replace the archetype at `archetype_index` of round `round_index`.
#[wasm_bindgen]
pub fn training_pack_set_round_archetype(
    lossless: &str,
    round_index: usize,
    archetype_index: usize,
    archetype: JsValue,
) -> Result<String, JsValue> {
    let archetype = archetype_from_js(archetype)?;
    edit_training_file(lossless, |file| {
        file.set_round_archetype(round_index, archetype_index, &archetype)
    })
}

/// Append an archetype to the round at `round_index`.
#[wasm_bindgen]
pub fn training_pack_add_round_archetype(
    lossless: &str,
    round_index: usize,
    archetype: JsValue,
) -> Result<String, JsValue> {
    let archetype = archetype_from_js(archetype)?;
    edit_training_file(lossless, |file| {
        file.add_round_archetype(round_index, &archetype)
    })
}

/// Remove the archetype at `archetype_index` of round `round_index`.
#[wasm_bindgen]
pub fn training_pack_remove_round_archetype(
    lossless: &str,
    round_index: usize,
    archetype_index: usize,
) -> Result<String, JsValue> {
    edit_training_file(lossless, |file| {
        file.remove_round_archetype(round_index, archetype_index)
            .map(|_| ())
    })
}

/// Set the ball of the round at `round_index`: replaces the round's first
/// ball archetype, or inserts one at position 0 if the round has none.
#[wasm_bindgen]
pub fn training_pack_set_round_ball(
    lossless: &str,
    round_index: usize,
    ball: JsValue,
) -> Result<String, JsValue> {
    let ball = ball_from_js(ball)?;
    edit_training_file(lossless, |file| file.set_round_ball(round_index, &ball))
}

/// Set the time limit of the round at `round_index` in place (0 removes the
/// property, matching the game's omit-default convention).
#[wasm_bindgen]
pub fn training_pack_set_round_time_limit(
    lossless: &str,
    round_index: usize,
    time_limit: f32,
) -> Result<String, JsValue> {
    edit_training_file(lossless, |file| {
        file.set_round_time_limit(round_index, time_limit)
    })
}
