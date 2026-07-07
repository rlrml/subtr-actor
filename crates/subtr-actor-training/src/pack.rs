//! Typed view of a training pack on top of the generic property tree.
//!
//! [`TrainingPack`] mirrors `TAGame.TrainingEditorData_TA` plus the root
//! `SaveData_GameEditor_Training_TA` fields. It is a *view*: reading builds
//! it from the tree, and the edit methods on [`TrainingFile`] write back
//! into the tree in place, so unknown properties are retained in position.
//! `SerializedArchetypes` strings are preserved exactly here; structured
//! parsing/editing of them lives in [`crate::archetype`].

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::container::{SaveObject, TrainingFile, VersionInfo};
use crate::error::{Error, Result};
use crate::io::UeString;
use crate::property::{
    ArrayValue, ByteValue, PropertyList, PropertyValue, StructBody, StructValue,
};

const TRAINING_DATA_TYPE: &str = "TAGame.TrainingEditorData_TA";

/// A UE3 GUID: four raw little-endian `i32`s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct Guid {
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
}

impl Guid {
    fn from_raw(bytes: &[u8]) -> Option<Guid> {
        if bytes.len() != 16 {
            return None;
        }
        let field = |i: usize| i32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap());
        Some(Guid {
            a: field(0),
            b: field(1),
            c: field(2),
            d: field(3),
        })
    }

    fn to_raw(self) -> Vec<u8> {
        [self.a, self.b, self.c, self.d]
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }
}

/// `ETrainingType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrainingType {
    None,
    Aerial,
    Goalie,
    Striker,
    End,
    /// An enum value name this crate does not know.
    Other(String),
}

impl TrainingType {
    pub fn as_name(&self) -> &str {
        match self {
            TrainingType::None => "Training_None",
            TrainingType::Aerial => "Training_Aerial",
            TrainingType::Goalie => "Training_Goalie",
            TrainingType::Striker => "Training_Striker",
            TrainingType::End => "Training_END",
            TrainingType::Other(name) => name,
        }
    }

    pub fn from_name(name: &str) -> TrainingType {
        match name {
            "Training_None" => TrainingType::None,
            "Training_Aerial" => TrainingType::Aerial,
            "Training_Goalie" => TrainingType::Goalie,
            "Training_Striker" => TrainingType::Striker,
            "Training_END" => TrainingType::End,
            other => TrainingType::Other(other.to_string()),
        }
    }
}

/// `EDifficulty`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    End,
    /// An enum value name this crate does not know.
    Other(String),
}

impl Difficulty {
    pub fn as_name(&self) -> &str {
        match self {
            Difficulty::Easy => "D_Easy",
            Difficulty::Medium => "D_Medium",
            Difficulty::Hard => "D_Hard",
            Difficulty::End => "D_END",
            Difficulty::Other(name) => name,
        }
    }

    pub fn from_name(name: &str) -> Difficulty {
        match name {
            "D_Easy" => Difficulty::Easy,
            "D_Medium" => Difficulty::Medium,
            "D_Hard" => Difficulty::Hard,
            "D_END" => Difficulty::End,
            other => Difficulty::Other(other.to_string()),
        }
    }
}

macro_rules! string_backed_serde {
    ($type:ty) => {
        impl Serialize for $type {
            fn serialize<S: Serializer>(
                &self,
                serializer: S,
            ) -> std::result::Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_name())
            }
        }

        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D: Deserializer<'de>>(
                deserializer: D,
            ) -> std::result::Result<Self, D::Error> {
                Ok(Self::from_name(&String::deserialize(deserializer)?))
            }
        }
    };
}

string_backed_serde!(TrainingType);
string_backed_serde!(Difficulty);

/// Typed view of `CreatorPlayerID` (a `UniqueNetId` struct serialized as
/// nested tagged properties). Read-only in this phase; unknown subfields
/// (e.g. `NpId`) survive in the underlying tree even though they are not
/// surfaced here.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerId {
    pub uid: u64,
    pub epic_account_id: Option<String>,
    /// `OnlinePlatform` enum value name, e.g. `OnlinePlatform_Steam`.
    pub platform: Option<String>,
    pub splitscreen_id: u8,
}

/// One training round.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct Round {
    pub time_limit: f32,
    /// Raw archetype strings, preserved exactly. Parse them with
    /// [`crate::archetype::Archetype::parse`].
    pub serialized_archetypes: Vec<String>,
}

/// Typed training pack model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct TrainingPack {
    pub guid: Guid,
    pub code: Option<String>,
    pub name: Option<String>,
    #[ts(type = "string")]
    pub training_type: TrainingType,
    #[ts(type = "string")]
    pub difficulty: Difficulty,
    pub creator_name: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<i32>,
    pub map_name: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub creator_player_id: PlayerId,
    pub rounds: Vec<Round>,
    pub player_team_number: i32,
    pub unowned: bool,
    pub perfect_completed: bool,
    pub shots_completed: i32,
}

impl Default for TrainingPack {
    fn default() -> Self {
        TrainingPack {
            guid: Guid::default(),
            code: None,
            name: None,
            training_type: TrainingType::None,
            difficulty: Difficulty::Easy,
            creator_name: None,
            description: None,
            tags: Vec::new(),
            map_name: None,
            created_at: 0,
            updated_at: 0,
            creator_player_id: PlayerId::default(),
            rounds: Vec::new(),
            player_team_number: 0,
            unowned: false,
            perfect_completed: false,
            shots_completed: 0,
        }
    }
}

// --- reading helpers over property lists ---

fn opt_string(list: &PropertyList, name: &str) -> Option<String> {
    match &list.get(name)?.value {
        PropertyValue::Str(value) | PropertyValue::Name(value) => {
            value.as_str().map(str::to_string)
        }
        _ => None,
    }
}

fn int_or(list: &PropertyList, name: &str, default: i32) -> i32 {
    match list.get(name).map(|property| &property.value) {
        Some(PropertyValue::Int(value)) => *value,
        _ => default,
    }
}

fn qword_or(list: &PropertyList, name: &str, default: u64) -> u64 {
    match list.get(name).map(|property| &property.value) {
        Some(PropertyValue::QWord(value)) => *value,
        _ => default,
    }
}

fn bool_or(list: &PropertyList, name: &str, default: bool) -> bool {
    match list.get(name).map(|property| &property.value) {
        Some(PropertyValue::Bool(value)) => *value != 0,
        _ => default,
    }
}

fn enum_name(list: &PropertyList, name: &str) -> Option<String> {
    match &list.get(name)?.value {
        PropertyValue::Byte(ByteValue::Enum { value, .. }) => value.as_str().map(str::to_string),
        _ => None,
    }
}

fn byte_or(list: &PropertyList, name: &str, default: u8) -> u8 {
    match list.get(name).map(|property| &property.value) {
        Some(PropertyValue::Byte(ByteValue::Raw(value))) => *value,
        _ => default,
    }
}

fn round_from_properties(list: &PropertyList) -> Round {
    let time_limit = match list.get("TimeLimit").map(|property| &property.value) {
        Some(PropertyValue::Float(value)) => *value,
        _ => 0.0,
    };
    let serialized_archetypes = match list
        .get("SerializedArchetypes")
        .map(|property| &property.value)
    {
        Some(PropertyValue::Array(ArrayValue::Strings(items))) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    };
    Round {
        time_limit,
        serialized_archetypes,
    }
}

fn round_to_properties(round: &Round) -> PropertyList {
    let mut list = PropertyList::default();
    // Mirror the game's writer: properties equal to their type default are
    // omitted.
    if round.time_limit != 0.0 {
        list.set("TimeLimit", PropertyValue::Float(round.time_limit));
    }
    if !round.serialized_archetypes.is_empty() {
        list.set(
            "SerializedArchetypes",
            PropertyValue::Array(ArrayValue::Strings(
                round
                    .serialized_archetypes
                    .iter()
                    .map(|value| UeString::new(value))
                    .collect(),
            )),
        );
    }
    list
}

fn enum_property(enum_type: &str, value: &str) -> PropertyValue {
    PropertyValue::Byte(ByteValue::Enum {
        enum_type: UeString::new(enum_type),
        value: UeString::new(value),
    })
}

fn str_property(value: Option<&str>) -> PropertyValue {
    PropertyValue::Str(UeString::from_option(value))
}

impl TrainingFile {
    /// Index (into [`TrainingFile::objects`]) of the training data object,
    /// resolved through the root's `TrainingData` object reference with a
    /// fallback to a type-name search.
    pub fn training_data_index(&self) -> Result<usize> {
        if let Some(property) = self.root.get("TrainingData") {
            if let PropertyValue::Object(index) = &property.value {
                return usize::try_from(*index)
                    .ok()
                    .filter(|&index| index < self.objects.len())
                    .ok_or(Error::TrainingDataIndexOutOfRange {
                        index: *index,
                        count: self.objects.len(),
                    });
            }
        }
        self.objects
            .iter()
            .position(|object| object.type_name.is(TRAINING_DATA_TYPE))
            .ok_or(Error::NoTrainingData)
    }

    /// The training data object's property list.
    pub fn training_data(&self) -> Result<&PropertyList> {
        Ok(&self.objects[self.training_data_index()?].properties)
    }

    /// Mutable access to the training data object's property list.
    pub fn training_data_mut(&mut self) -> Result<&mut PropertyList> {
        let index = self.training_data_index()?;
        Ok(&mut self.objects[index].properties)
    }

    /// Build the typed view of this file.
    pub fn pack(&self) -> Result<TrainingPack> {
        let data = self.training_data()?;

        let guid = match data.get("TM_Guid").map(|property| &property.value) {
            Some(PropertyValue::Struct(StructValue {
                body: StructBody::Raw(bytes),
                ..
            })) => Guid::from_raw(bytes).unwrap_or_default(),
            _ => Guid::default(),
        };

        let creator_player_id = match data.get("CreatorPlayerID").map(|property| &property.value) {
            Some(PropertyValue::Struct(StructValue {
                body: StructBody::Properties(fields),
                ..
            })) => PlayerId {
                uid: qword_or(fields, "Uid", 0),
                epic_account_id: opt_string(fields, "EpicAccountId"),
                platform: enum_name(fields, "Platform"),
                splitscreen_id: byte_or(fields, "SplitscreenID", 0),
            },
            _ => PlayerId::default(),
        };

        let tags = match data.get("Tags").map(|property| &property.value) {
            Some(PropertyValue::Array(ArrayValue::Ints(items))) => items.clone(),
            _ => Vec::new(),
        };

        let rounds = match data.get("Rounds").map(|property| &property.value) {
            Some(PropertyValue::Array(ArrayValue::Structs(items))) => {
                items.iter().map(round_from_properties).collect()
            }
            _ => Vec::new(),
        };

        Ok(TrainingPack {
            guid,
            code: opt_string(data, "Code"),
            name: opt_string(data, "TM_Name"),
            training_type: enum_name(data, "Type")
                .map_or(TrainingType::None, |name| TrainingType::from_name(&name)),
            difficulty: enum_name(data, "Difficulty")
                .map_or(Difficulty::Easy, |name| Difficulty::from_name(&name)),
            creator_name: opt_string(data, "CreatorName"),
            description: opt_string(data, "Description"),
            tags,
            map_name: opt_string(data, "MapName"),
            created_at: qword_or(data, "CreatedAt", 0),
            updated_at: qword_or(data, "UpdatedAt", 0),
            creator_player_id,
            rounds,
            player_team_number: int_or(&self.root, "PlayerTeamNumber", 0),
            unowned: bool_or(&self.root, "bUnowned", false),
            perfect_completed: bool_or(&self.root, "bPerfectCompleted", false),
            shots_completed: int_or(&self.root, "ShotsCompleted", 0),
        })
    }

    /// Build a fresh container from a typed pack, following the reference
    /// implementation's convention of omitting properties whose value equals
    /// the type default.
    pub fn from_pack(pack: &TrainingPack) -> Result<TrainingFile> {
        let mut data = PropertyList::default();
        if pack.guid != Guid::default() {
            data.set(
                "TM_Guid",
                PropertyValue::Struct(StructValue {
                    struct_type: UeString::new("Guid"),
                    body: StructBody::Raw(pack.guid.to_raw()),
                }),
            );
        }
        if let Some(code) = &pack.code {
            data.set("Code", str_property(Some(code)));
        }
        if let Some(name) = &pack.name {
            data.set("TM_Name", str_property(Some(name)));
        }
        if pack.training_type != TrainingType::None {
            data.set(
                "Type",
                enum_property("ETrainingType", pack.training_type.as_name()),
            );
        }
        if pack.difficulty != Difficulty::Easy {
            data.set(
                "Difficulty",
                enum_property("EDifficulty", pack.difficulty.as_name()),
            );
        }
        if let Some(creator_name) = &pack.creator_name {
            data.set("CreatorName", str_property(Some(creator_name)));
        }
        if let Some(description) = &pack.description {
            data.set("Description", str_property(Some(description)));
        }
        if !pack.tags.is_empty() {
            data.set(
                "Tags",
                PropertyValue::Array(ArrayValue::Ints(pack.tags.clone())),
            );
        }
        if let Some(map_name) = &pack.map_name {
            data.set("MapName", PropertyValue::Name(UeString::new(map_name)));
        }
        if pack.created_at != 0 {
            data.set("CreatedAt", PropertyValue::QWord(pack.created_at));
        }
        if pack.updated_at != 0 {
            data.set("UpdatedAt", PropertyValue::QWord(pack.updated_at));
        }
        if pack.creator_player_id != PlayerId::default() {
            let player = &pack.creator_player_id;
            let mut fields = PropertyList::default();
            if player.uid != 0 {
                fields.set("Uid", PropertyValue::QWord(player.uid));
            }
            if let Some(epic) = &player.epic_account_id {
                fields.set("EpicAccountId", str_property(Some(epic)));
            }
            if let Some(platform) = &player.platform {
                if platform != "OnlinePlatform_Unknown" {
                    fields.set("Platform", enum_property("OnlinePlatform", platform));
                }
            }
            if player.splitscreen_id != 0 {
                fields.set(
                    "SplitscreenID",
                    PropertyValue::Byte(ByteValue::Raw(player.splitscreen_id)),
                );
            }
            data.set(
                "CreatorPlayerID",
                PropertyValue::Struct(StructValue {
                    struct_type: UeString::new("UniqueNetId"),
                    body: StructBody::Properties(fields),
                }),
            );
        }
        if !pack.rounds.is_empty() {
            data.set(
                "Rounds",
                PropertyValue::Array(ArrayValue::Structs(
                    pack.rounds.iter().map(round_to_properties).collect(),
                )),
            );
        }

        let mut root = PropertyList::default();
        root.set("TrainingData", PropertyValue::Object(0));
        if pack.player_team_number != 0 {
            root.set(
                "PlayerTeamNumber",
                PropertyValue::Int(pack.player_team_number),
            );
        }
        if pack.unowned {
            root.set("bUnowned", PropertyValue::Bool(1));
        }
        if pack.perfect_completed {
            root.set("bPerfectCompleted", PropertyValue::Bool(1));
        }
        if pack.shots_completed != 0 {
            root.set("ShotsCompleted", PropertyValue::Int(pack.shots_completed));
        }

        Ok(TrainingFile {
            version: VersionInfo::default(),
            root,
            objects: vec![SaveObject {
                type_name: UeString::new(TRAINING_DATA_TYPE),
                object_index: 0,
                properties: data,
            }],
        })
    }

    // --- scalar setters (write into the tree, preserving unknown props) ---

    pub fn set_name(&mut self, name: Option<&str>) -> Result<()> {
        self.training_data_mut()?.set("TM_Name", str_property(name));
        Ok(())
    }

    pub fn set_code(&mut self, code: Option<&str>) -> Result<()> {
        self.training_data_mut()?.set("Code", str_property(code));
        Ok(())
    }

    pub fn set_description(&mut self, description: Option<&str>) -> Result<()> {
        self.training_data_mut()?
            .set("Description", str_property(description));
        Ok(())
    }

    pub fn set_creator_name(&mut self, creator_name: Option<&str>) -> Result<()> {
        self.training_data_mut()?
            .set("CreatorName", str_property(creator_name));
        Ok(())
    }

    pub fn set_training_type(&mut self, training_type: &TrainingType) -> Result<()> {
        self.training_data_mut()?.set(
            "Type",
            enum_property("ETrainingType", training_type.as_name()),
        );
        Ok(())
    }

    pub fn set_difficulty(&mut self, difficulty: &Difficulty) -> Result<()> {
        self.training_data_mut()?.set(
            "Difficulty",
            enum_property("EDifficulty", difficulty.as_name()),
        );
        Ok(())
    }

    pub fn set_map_name(&mut self, map_name: &str) -> Result<()> {
        self.training_data_mut()?
            .set("MapName", PropertyValue::Name(UeString::new(map_name)));
        Ok(())
    }

    pub fn set_tags(&mut self, tags: Vec<i32>) -> Result<()> {
        self.training_data_mut()?
            .set("Tags", PropertyValue::Array(ArrayValue::Ints(tags)));
        Ok(())
    }

    pub fn set_guid(&mut self, guid: Guid) -> Result<()> {
        self.training_data_mut()?.set(
            "TM_Guid",
            PropertyValue::Struct(StructValue {
                struct_type: UeString::new("Guid"),
                body: StructBody::Raw(guid.to_raw()),
            }),
        );
        Ok(())
    }

    pub fn set_created_at(&mut self, created_at: u64) -> Result<()> {
        self.training_data_mut()?
            .set("CreatedAt", PropertyValue::QWord(created_at));
        Ok(())
    }

    pub fn set_updated_at(&mut self, updated_at: u64) -> Result<()> {
        self.training_data_mut()?
            .set("UpdatedAt", PropertyValue::QWord(updated_at));
        Ok(())
    }

    pub fn set_player_team_number(&mut self, team: i32) {
        self.root.set("PlayerTeamNumber", PropertyValue::Int(team));
    }

    pub fn set_unowned(&mut self, unowned: bool) {
        self.root
            .set("bUnowned", PropertyValue::Bool(unowned.into()));
    }

    pub fn set_perfect_completed(&mut self, perfect_completed: bool) {
        self.root.set(
            "bPerfectCompleted",
            PropertyValue::Bool(perfect_completed.into()),
        );
    }

    pub fn set_shots_completed(&mut self, shots_completed: i32) {
        self.root
            .set("ShotsCompleted", PropertyValue::Int(shots_completed));
    }

    // --- round editing ---

    /// Mutable access to the rounds as raw property lists, creating the
    /// `Rounds` property if absent. Round-level unknown properties survive
    /// reorder/duplicate/remove because whole property lists are moved.
    pub fn rounds_mut(&mut self) -> Result<&mut Vec<PropertyList>> {
        let data = self.training_data_mut()?;
        if data.get("Rounds").is_none() {
            data.set("Rounds", PropertyValue::Array(ArrayValue::Structs(vec![])));
        }
        let property = data.get_mut("Rounds").unwrap();
        // An empty array parsed without a usable element interpretation is
        // represented as `Raw`; coerce it so it can be edited.
        if matches!(
            &property.value,
            PropertyValue::Array(ArrayValue::Raw { count: 0, data })
                if data.is_empty()
        ) {
            property.value = PropertyValue::Array(ArrayValue::Structs(vec![]));
            property.declared_length = None;
        }
        match &mut property.value {
            PropertyValue::Array(ArrayValue::Structs(items)) => Ok(items),
            _ => Err(Error::UnexpectedPropertyShape {
                name: "Rounds".to_string(),
                reason: "not an array of structs".to_string(),
            }),
        }
    }

    /// The parsed rounds (typed view).
    pub fn rounds(&self) -> Result<Vec<Round>> {
        Ok(self.pack()?.rounds)
    }

    /// Append a round built from the typed representation.
    pub fn add_round(&mut self, round: &Round) -> Result<()> {
        self.rounds_mut()?.push(round_to_properties(round));
        Ok(())
    }

    /// Insert a round at `index`.
    pub fn insert_round(&mut self, index: usize, round: &Round) -> Result<()> {
        let rounds = self.rounds_mut()?;
        let index = index.min(rounds.len());
        rounds.insert(index, round_to_properties(round));
        Ok(())
    }

    /// Remove and return the round at `index` (typed view of what was
    /// removed).
    pub fn remove_round(&mut self, index: usize) -> Result<Round> {
        let rounds = self.rounds_mut()?;
        if index >= rounds.len() {
            return Err(Error::UnexpectedPropertyShape {
                name: "Rounds".to_string(),
                reason: format!("index {index} out of range ({} rounds)", rounds.len()),
            });
        }
        Ok(round_from_properties(&rounds.remove(index)))
    }

    /// Move the round at `from` to position `to`.
    pub fn move_round(&mut self, from: usize, to: usize) -> Result<()> {
        let rounds = self.rounds_mut()?;
        if from >= rounds.len() || to >= rounds.len() {
            return Err(Error::UnexpectedPropertyShape {
                name: "Rounds".to_string(),
                reason: format!("move {from} -> {to} out of range ({} rounds)", rounds.len()),
            });
        }
        let round = rounds.remove(from);
        rounds.insert(to, round);
        Ok(())
    }

    /// Duplicate the round at `index`, inserting the copy right after it.
    pub fn duplicate_round(&mut self, index: usize) -> Result<()> {
        let rounds = self.rounds_mut()?;
        if index >= rounds.len() {
            return Err(Error::UnexpectedPropertyShape {
                name: "Rounds".to_string(),
                reason: format!("index {index} out of range ({} rounds)", rounds.len()),
            });
        }
        let copy = rounds[index].clone();
        rounds.insert(index + 1, copy);
        Ok(())
    }

    /// Append every round of `other` to this pack (lossless: whole property
    /// lists are copied, including unknown round-level properties).
    pub fn append_rounds_from(&mut self, other: &TrainingFile) -> Result<usize> {
        let other_rounds = match other
            .training_data()?
            .get("Rounds")
            .map(|property| &property.value)
        {
            Some(PropertyValue::Array(ArrayValue::Structs(items))) => items.clone(),
            _ => Vec::new(),
        };
        let count = other_rounds.len();
        self.rounds_mut()?.extend(other_rounds);
        Ok(count)
    }
}

/// Convenience: parse the property list of a round.
pub fn round_view(list: &PropertyList) -> Round {
    round_from_properties(list)
}

#[cfg(test)]
#[path = "pack_tests.rs"]
mod tests;
