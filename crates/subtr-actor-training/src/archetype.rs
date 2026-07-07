//! Structured parsing and regeneration of per-round `SerializedArchetypes`
//! strings.
//!
//! Each string in a round's `SerializedArchetypes` array is a standalone JSON
//! object written by the in-game training editor. Three shapes exist in the
//! wild:
//!
//! * the ball ([`BallSpawn`], `ObjectArchetype` =
//!   `Archetypes.Ball.Ball_GameEditor`),
//! * a car spawn point ([`CarSpawn`], `ObjectArchetype` =
//!   `Archetypes.GameEditor.DynamicSpawnPointMesh`),
//! * the player car ([`PlayerCarSpawn`], no `ObjectArchetype` key, identified
//!   by the presence of `IsPC`).
//!
//! # Fidelity model
//!
//! Rounds keep their original strings; parsing is on demand and editing
//! regenerates only the specific archetype string being modified, so
//! untouched rounds/archetypes stay byte-identical through a full `.tem`
//! round trip. The game's original float formatting (e.g. `1554.78` vs
//! `1554.7800`) would not survive re-serialization, which is why unmodified
//! strings are never rewritten.
//!
//! For a *modified* archetype the regenerated string matches observed game
//! output: fixed key order (as listed on each struct), floats with exactly
//! four decimal places, rotator components as bare integers, and lowercase
//! booleans. The equality standard for a regenerated string is *semantic*
//! (parsed-JSON numeric equality, so `0` == `0.0000`), not byte equality.
//!
//! Unknown keys are preserved in each struct's `extras` map and re-emitted
//! after the known keys. Because `serde_json::Map` is sorted (this crate does
//! not enable serde_json's `preserve_order` feature), extras serialize in
//! alphabetical key order, with serde_json's default value formatting.
//!
//! Parsing is conservative: any string that is not a JSON object, has an
//! unrecognized `ObjectArchetype`, or is missing (or has a mistyped) required
//! key falls back to [`Archetype::Unknown`] with the raw string preserved
//! verbatim.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::container::TrainingFile;
use crate::error::{Error, Result};
use crate::io::UeString;
use crate::property::{ArrayValue, Property, PropertyList, PropertyValue};

/// `ObjectArchetype` value of the ball entry.
pub const BALL_OBJECT_ARCHETYPE: &str = "Archetypes.Ball.Ball_GameEditor";
/// `ObjectArchetype` value of a car spawn point entry.
pub const CAR_SPAWN_OBJECT_ARCHETYPE: &str = "Archetypes.GameEditor.DynamicSpawnPointMesh";

/// The ball entry of a round.
///
/// Key order in the serialized string: `ObjectArchetype`, `StartLocationX/Y/Z`
/// (floats, uu), `VelocityStartRotationP/Y/R` (integer UE rotator units,
/// 65536 = 360 degrees), `VelocityStartSpeed` (float, uu/s), then extras.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BallSpawn {
    pub start_location_x: f64,
    pub start_location_y: f64,
    pub start_location_z: f64,
    pub velocity_start_rotation_p: i32,
    pub velocity_start_rotation_y: i32,
    pub velocity_start_rotation_r: i32,
    pub velocity_start_speed: f64,
    /// Unknown keys, preserved through parse/serialize (alphabetical on
    /// output).
    #[serde(default)]
    #[ts(type = "Record<string, unknown>")]
    pub extras: Map<String, Value>,
}

impl Default for BallSpawn {
    /// The values a freshly created editor round uses for its ball.
    fn default() -> Self {
        BallSpawn {
            start_location_x: 0.0,
            start_location_y: 4120.0,
            start_location_z: 100.4872,
            velocity_start_rotation_p: 8191,
            velocity_start_rotation_y: -16384,
            velocity_start_rotation_r: 0,
            velocity_start_speed: 1500.0,
            extras: Map::new(),
        }
    }
}

/// A car spawn point entry (`DynamicSpawnPointMesh`).
///
/// Key order in the serialized string: `ObjectArchetype`, `LocationX/Y/Z`
/// (floats, uu), `RotationP/Y/R` (integer UE rotator units),
/// `VelocityStartSpeed` (float, omitted when `None` — Psyonix-made packs do
/// not write it), then extras.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CarSpawn {
    pub location_x: f64,
    pub location_y: f64,
    pub location_z: f64,
    pub rotation_p: i32,
    pub rotation_y: i32,
    pub rotation_r: i32,
    #[serde(default)]
    pub velocity_start_speed: Option<f64>,
    /// Unknown keys, preserved through parse/serialize (alphabetical on
    /// output).
    #[serde(default)]
    #[ts(type = "Record<string, unknown>")]
    pub extras: Map<String, Value>,
}

impl Default for CarSpawn {
    /// The values a freshly created editor round uses for its spawn point.
    fn default() -> Self {
        CarSpawn {
            location_x: 0.0,
            location_y: 0.0,
            location_z: 30.0,
            rotation_p: 0,
            rotation_y: 16384,
            rotation_r: 0,
            velocity_start_speed: Some(0.0),
            extras: Map::new(),
        }
    }
}

/// The player car entry (no `ObjectArchetype` key; identified by `IsPC`).
///
/// The transform keys are individually optional: real game output includes
/// bare `{"IsPC":true}` entries with no transform at all.
///
/// Key order in the serialized string: `IsPC`, `LocationX/Y/Z` (floats, uu),
/// `RotationP/Y/R` (integer UE rotator units) — each written only when
/// `Some` — then extras.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerCarSpawn {
    pub is_pc: bool,
    #[serde(default)]
    pub location_x: Option<f64>,
    #[serde(default)]
    pub location_y: Option<f64>,
    #[serde(default)]
    pub location_z: Option<f64>,
    #[serde(default)]
    pub rotation_p: Option<i32>,
    #[serde(default)]
    pub rotation_y: Option<i32>,
    #[serde(default)]
    pub rotation_r: Option<i32>,
    /// Unknown keys, preserved through parse/serialize (alphabetical on
    /// output).
    #[serde(default)]
    #[ts(type = "Record<string, unknown>")]
    pub extras: Map<String, Value>,
}

impl Default for PlayerCarSpawn {
    /// A player car entry with a zeroed transform, as observed in
    /// Psyonix-made packs.
    fn default() -> Self {
        PlayerCarSpawn {
            is_pc: true,
            location_x: Some(0.0),
            location_y: Some(0.0),
            location_z: Some(0.0),
            rotation_p: Some(0),
            rotation_y: Some(0),
            rotation_r: Some(0),
            extras: Map::new(),
        }
    }
}

/// One parsed `SerializedArchetypes` entry.
///
/// Serde/TS representation: internally tagged on `kind`, so the generated
/// TypeScript type is a discriminated union like
/// `{ kind: "Ball" } & BallSpawn`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(tag = "kind")]
pub enum Archetype {
    Ball(BallSpawn),
    CarSpawnPoint(CarSpawn),
    PlayerCar(PlayerCarSpawn),
    /// Anything this crate does not recognize; the raw string is preserved
    /// verbatim and round-trips byte-identically.
    Unknown {
        raw: String,
    },
}

fn number_to_i32(value: &Value) -> Option<i32> {
    if let Some(int) = value.as_i64() {
        return i32::try_from(int).ok();
    }
    // Tolerate integral floats (`16384.0`); the game writes bare integers,
    // but numeric JSON equality treats them as the same value.
    let float = value.as_f64()?;
    (float.fract() == 0.0 && float >= f64::from(i32::MIN) && float <= f64::from(i32::MAX))
        .then_some(float as i32)
}

fn take_f64(map: &mut Map<String, Value>, key: &str) -> Option<f64> {
    map.remove(key)?.as_f64()
}

fn take_i32(map: &mut Map<String, Value>, key: &str) -> Option<i32> {
    number_to_i32(&map.remove(key)?)
}

/// `Some(None)` when the key is absent, `None` when present but mistyped.
fn take_opt_f64(map: &mut Map<String, Value>, key: &str) -> Option<Option<f64>> {
    match map.remove(key) {
        None => Some(None),
        Some(value) => value.as_f64().map(Some),
    }
}

fn take_opt_i32(map: &mut Map<String, Value>, key: &str) -> Option<Option<i32>> {
    match map.remove(key) {
        None => Some(None),
        Some(value) => number_to_i32(&value).map(Some),
    }
}

fn parse_ball(mut map: Map<String, Value>) -> Option<BallSpawn> {
    Some(BallSpawn {
        start_location_x: take_f64(&mut map, "StartLocationX")?,
        start_location_y: take_f64(&mut map, "StartLocationY")?,
        start_location_z: take_f64(&mut map, "StartLocationZ")?,
        velocity_start_rotation_p: take_i32(&mut map, "VelocityStartRotationP")?,
        velocity_start_rotation_y: take_i32(&mut map, "VelocityStartRotationY")?,
        velocity_start_rotation_r: take_i32(&mut map, "VelocityStartRotationR")?,
        velocity_start_speed: take_f64(&mut map, "VelocityStartSpeed")?,
        extras: map,
    })
}

fn parse_car_spawn(mut map: Map<String, Value>) -> Option<CarSpawn> {
    Some(CarSpawn {
        location_x: take_f64(&mut map, "LocationX")?,
        location_y: take_f64(&mut map, "LocationY")?,
        location_z: take_f64(&mut map, "LocationZ")?,
        rotation_p: take_i32(&mut map, "RotationP")?,
        rotation_y: take_i32(&mut map, "RotationY")?,
        rotation_r: take_i32(&mut map, "RotationR")?,
        velocity_start_speed: take_opt_f64(&mut map, "VelocityStartSpeed")?,
        extras: map,
    })
}

fn parse_player_car(mut map: Map<String, Value>) -> Option<PlayerCarSpawn> {
    Some(PlayerCarSpawn {
        is_pc: map.remove("IsPC")?.as_bool()?,
        location_x: take_opt_f64(&mut map, "LocationX")?,
        location_y: take_opt_f64(&mut map, "LocationY")?,
        location_z: take_opt_f64(&mut map, "LocationZ")?,
        rotation_p: take_opt_i32(&mut map, "RotationP")?,
        rotation_y: take_opt_i32(&mut map, "RotationY")?,
        rotation_r: take_opt_i32(&mut map, "RotationR")?,
        extras: map,
    })
}

fn parse_known(raw: &str) -> Option<Archetype> {
    let Value::Object(mut map) = serde_json::from_str(raw).ok()? else {
        return None;
    };
    match map.get("ObjectArchetype").and_then(Value::as_str) {
        Some(BALL_OBJECT_ARCHETYPE) => {
            map.remove("ObjectArchetype");
            parse_ball(map).map(Archetype::Ball)
        }
        Some(CAR_SPAWN_OBJECT_ARCHETYPE) => {
            map.remove("ObjectArchetype");
            parse_car_spawn(map).map(Archetype::CarSpawnPoint)
        }
        Some(_) => None,
        None if map.contains_key("IsPC") => parse_player_car(map).map(Archetype::PlayerCar),
        None => None,
    }
}

/// A tiny ordered JSON object writer matching observed game output: fixed
/// key order, floats as `{:.4}`, integers bare, booleans lowercase.
struct FieldWriter {
    out: String,
}

impl FieldWriter {
    fn new() -> FieldWriter {
        FieldWriter {
            out: String::from("{"),
        }
    }

    fn key(&mut self, key: &str) {
        if self.out.len() > 1 {
            self.out.push(',');
        }
        // Known keys are plain identifiers, but extras keys may need JSON
        // escaping.
        let _ = write!(self.out, "{}:", Value::String(key.to_string()));
    }

    fn float(&mut self, key: &str, value: f64) {
        self.key(key);
        // Non-finite floats have no JSON representation; the game never
        // produces them, so clamp rather than emit invalid JSON.
        let value = if value.is_finite() { value } else { 0.0 };
        let _ = write!(self.out, "{value:.4}");
    }

    fn opt_float(&mut self, key: &str, value: Option<f64>) {
        if let Some(value) = value {
            self.float(key, value);
        }
    }

    fn opt_int(&mut self, key: &str, value: Option<i32>) {
        if let Some(value) = value {
            self.int(key, value);
        }
    }

    fn int(&mut self, key: &str, value: i32) {
        self.key(key);
        let _ = write!(self.out, "{value}");
    }

    fn bool(&mut self, key: &str, value: bool) {
        self.key(key);
        let _ = write!(self.out, "{value}");
    }

    fn string(&mut self, key: &str, value: &str) {
        self.key(key);
        let _ = write!(self.out, "{}", Value::String(value.to_string()));
    }

    fn extras(&mut self, extras: &Map<String, Value>) {
        for (key, value) in extras {
            self.key(key);
            let _ = write!(self.out, "{value}");
        }
    }

    fn finish(mut self) -> String {
        self.out.push('}');
        self.out
    }
}

impl BallSpawn {
    fn write_archetype_string(&self) -> String {
        let mut writer = FieldWriter::new();
        writer.string("ObjectArchetype", BALL_OBJECT_ARCHETYPE);
        writer.float("StartLocationX", self.start_location_x);
        writer.float("StartLocationY", self.start_location_y);
        writer.float("StartLocationZ", self.start_location_z);
        writer.int("VelocityStartRotationP", self.velocity_start_rotation_p);
        writer.int("VelocityStartRotationY", self.velocity_start_rotation_y);
        writer.int("VelocityStartRotationR", self.velocity_start_rotation_r);
        writer.float("VelocityStartSpeed", self.velocity_start_speed);
        writer.extras(&self.extras);
        writer.finish()
    }
}

impl CarSpawn {
    fn write_archetype_string(&self) -> String {
        let mut writer = FieldWriter::new();
        writer.string("ObjectArchetype", CAR_SPAWN_OBJECT_ARCHETYPE);
        writer.float("LocationX", self.location_x);
        writer.float("LocationY", self.location_y);
        writer.float("LocationZ", self.location_z);
        writer.int("RotationP", self.rotation_p);
        writer.int("RotationY", self.rotation_y);
        writer.int("RotationR", self.rotation_r);
        writer.opt_float("VelocityStartSpeed", self.velocity_start_speed);
        writer.extras(&self.extras);
        writer.finish()
    }
}

impl PlayerCarSpawn {
    fn write_archetype_string(&self) -> String {
        let mut writer = FieldWriter::new();
        writer.bool("IsPC", self.is_pc);
        writer.opt_float("LocationX", self.location_x);
        writer.opt_float("LocationY", self.location_y);
        writer.opt_float("LocationZ", self.location_z);
        writer.opt_int("RotationP", self.rotation_p);
        writer.opt_int("RotationY", self.rotation_y);
        writer.opt_int("RotationR", self.rotation_r);
        writer.extras(&self.extras);
        writer.finish()
    }
}

impl Archetype {
    /// Parse one `SerializedArchetypes` string. Infallible: anything
    /// unrecognized becomes [`Archetype::Unknown`] with the raw string
    /// preserved verbatim.
    pub fn parse(raw: &str) -> Archetype {
        parse_known(raw).unwrap_or_else(|| Archetype::Unknown {
            raw: raw.to_string(),
        })
    }

    /// Regenerate the serialized string (see the module docs for the
    /// formatting rules). [`Archetype::Unknown`] returns its raw string
    /// unchanged.
    pub fn to_archetype_string(&self) -> String {
        match self {
            Archetype::Ball(ball) => ball.write_archetype_string(),
            Archetype::CarSpawnPoint(car) => car.write_archetype_string(),
            Archetype::PlayerCar(player_car) => player_car.write_archetype_string(),
            Archetype::Unknown { raw } => raw.clone(),
        }
    }
}

// --- TrainingFile archetype editing ---

const ARCHETYPES_PROPERTY: &str = "SerializedArchetypes";

fn round_out_of_range(index: usize, count: usize) -> Error {
    Error::UnexpectedPropertyShape {
        name: "Rounds".to_string(),
        reason: format!("index {index} out of range ({count} rounds)"),
    }
}

fn archetype_out_of_range(index: usize, count: usize) -> Error {
    Error::UnexpectedPropertyShape {
        name: ARCHETYPES_PROPERTY.to_string(),
        reason: format!("index {index} out of range ({count} archetypes)"),
    }
}

/// Mutable access to the archetype strings of a round, creating the property
/// (as an empty string array) when `create_if_absent`. Clears any recorded
/// declared-length quirk since the value is about to change.
fn archetype_strings_mut(
    round: &mut PropertyList,
    create_if_absent: bool,
) -> Result<Option<&mut Vec<UeString>>> {
    if round.get(ARCHETYPES_PROPERTY).is_none() {
        if !create_if_absent {
            return Ok(None);
        }
        round.set(
            ARCHETYPES_PROPERTY,
            PropertyValue::Array(ArrayValue::Strings(vec![])),
        );
    }
    let property = round.get_mut(ARCHETYPES_PROPERTY).unwrap();
    // An empty array parsed without a usable element interpretation is
    // represented as `Raw`; coerce it so it can be edited.
    if matches!(
        &property.value,
        PropertyValue::Array(ArrayValue::Raw { count: 0, data })
            if data.is_empty()
    ) {
        property.value = PropertyValue::Array(ArrayValue::Strings(vec![]));
    }
    property.declared_length = None;
    match &mut property.value {
        PropertyValue::Array(ArrayValue::Strings(items)) => Ok(Some(items)),
        _ => Err(Error::UnexpectedPropertyShape {
            name: ARCHETYPES_PROPERTY.to_string(),
            reason: "not an array of strings".to_string(),
        }),
    }
}

impl TrainingFile {
    /// The raw property list of the round at `round_index`.
    fn round_properties(&self, round_index: usize) -> Result<&PropertyList> {
        let rounds: &[PropertyList] = match self
            .training_data()?
            .get("Rounds")
            .map(|property| &property.value)
        {
            Some(PropertyValue::Array(ArrayValue::Structs(items))) => items,
            _ => &[],
        };
        rounds
            .get(round_index)
            .ok_or_else(|| round_out_of_range(round_index, rounds.len()))
    }

    /// Mutable access to the raw property list of the round at `round_index`.
    fn round_properties_mut(&mut self, round_index: usize) -> Result<&mut PropertyList> {
        let rounds = self.rounds_mut()?;
        let count = rounds.len();
        rounds
            .get_mut(round_index)
            .ok_or_else(|| round_out_of_range(round_index, count))
    }

    /// Parse every archetype string of the round at `round_index`.
    pub fn round_archetypes(&self, round_index: usize) -> Result<Vec<Archetype>> {
        let round = self.round_properties(round_index)?;
        let strings: &[UeString] = match round
            .get(ARCHETYPES_PROPERTY)
            .map(|property| &property.value)
        {
            Some(PropertyValue::Array(ArrayValue::Strings(items))) => items,
            _ => &[],
        };
        Ok(strings
            .iter()
            .map(|string| Archetype::parse(string.as_str().unwrap_or_default()))
            .collect())
    }

    /// Replace the archetype at `archetype_index` of round `round_index`
    /// with the regenerated string of `archetype`. Only this one string is
    /// rewritten; every other string in the file is left byte-identical.
    pub fn set_round_archetype(
        &mut self,
        round_index: usize,
        archetype_index: usize,
        archetype: &Archetype,
    ) -> Result<()> {
        let round = self.round_properties_mut(round_index)?;
        let strings = archetype_strings_mut(round, false)?
            .ok_or_else(|| archetype_out_of_range(archetype_index, 0))?;
        let count = strings.len();
        let slot = strings
            .get_mut(archetype_index)
            .ok_or_else(|| archetype_out_of_range(archetype_index, count))?;
        *slot = UeString::new(&archetype.to_archetype_string());
        Ok(())
    }

    /// Append an archetype to round `round_index`, creating the
    /// `SerializedArchetypes` property if the round has none.
    pub fn add_round_archetype(&mut self, round_index: usize, archetype: &Archetype) -> Result<()> {
        let round = self.round_properties_mut(round_index)?;
        let strings = archetype_strings_mut(round, true)?.expect("created when absent");
        strings.push(UeString::new(&archetype.to_archetype_string()));
        Ok(())
    }

    /// Remove and return (parsed) the archetype at `archetype_index` of round
    /// `round_index`. When the last archetype is removed the now-empty
    /// `SerializedArchetypes` property is dropped, matching the game's
    /// omit-empty convention.
    pub fn remove_round_archetype(
        &mut self,
        round_index: usize,
        archetype_index: usize,
    ) -> Result<Archetype> {
        let round = self.round_properties_mut(round_index)?;
        let strings = archetype_strings_mut(round, false)?
            .ok_or_else(|| archetype_out_of_range(archetype_index, 0))?;
        if archetype_index >= strings.len() {
            return Err(archetype_out_of_range(archetype_index, strings.len()));
        }
        let removed = strings.remove(archetype_index);
        let now_empty = strings.is_empty();
        if now_empty {
            round.remove(ARCHETYPES_PROPERTY);
        }
        Ok(Archetype::parse(removed.as_str().unwrap_or_default()))
    }

    /// Replace the first [`Archetype::Ball`] of round `round_index`, or
    /// insert one at position 0 if the round has no ball.
    pub fn set_round_ball(&mut self, round_index: usize, ball: &BallSpawn) -> Result<()> {
        let new_string = Archetype::Ball(ball.clone()).to_archetype_string();
        let round = self.round_properties_mut(round_index)?;
        let strings = archetype_strings_mut(round, true)?.expect("created when absent");
        let ball_position = strings.iter().position(|string| {
            matches!(
                Archetype::parse(string.as_str().unwrap_or_default()),
                Archetype::Ball(_)
            )
        });
        match ball_position {
            Some(index) => strings[index] = UeString::new(&new_string),
            None => strings.insert(0, UeString::new(&new_string)),
        }
        Ok(())
    }

    /// Set the time limit of round `round_index` in place. Following the
    /// game's omit-default convention (see `round_to_properties`), a value of
    /// `0.0` removes the `TimeLimit` property.
    pub fn set_round_time_limit(&mut self, round_index: usize, time_limit: f32) -> Result<()> {
        let round = self.round_properties_mut(round_index)?;
        if time_limit == 0.0 {
            round.remove("TimeLimit");
        } else if round.get("TimeLimit").is_some() {
            round.set("TimeLimit", PropertyValue::Float(time_limit));
        } else {
            // Match the game's property order: TimeLimit precedes
            // SerializedArchetypes.
            let position = round
                .properties
                .iter()
                .position(|property| property.name.is_ignore_case(ARCHETYPES_PROPERTY))
                .unwrap_or(round.properties.len());
            round.properties.insert(
                position,
                Property {
                    name: UeString::new("TimeLimit"),
                    index: 0,
                    declared_length: None,
                    value: PropertyValue::Float(time_limit),
                },
            );
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "archetype_tests.rs"]
mod tests;
