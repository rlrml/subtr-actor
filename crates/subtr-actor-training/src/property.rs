//! Generic UE3 tagged-property tree.
//!
//! Save data objects are serialized as a sequence of tagged properties:
//! `name` (string), `type` (string such as `IntProperty`), `i32` value
//! length, `i32` value index, then the value bytes; the sequence is
//! terminated by a property named `None`. Unlike the RocketRP reference
//! implementation (which reflects into concrete C# types and errors on
//! anything unknown), this tree preserves every property it encounters —
//! including unknown ones — so that parse → serialize reproduces the
//! original bytes exactly.
//!
//! Value-length quirks faithfully mirrored from RocketRP's
//! `RecalculateValueSizeFromType`:
//!
//! * `BoolProperty` writes one value byte but declares a length of **0**.
//! * A `ByteProperty` carrying an enum writes the enum *type* name followed
//!   by the enum *value* name, but the declared length covers only the value
//!   name.
//! * `StructProperty` writes the struct type name before the body, but the
//!   declared length covers only the body.
//! * `StrProperty` lengths are not trusted (they can mismatch for UTF-16
//!   strings); any divergence between the declared and computed length is
//!   preserved via [`Property::declared_length`].

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::io::{Reader, UeString, Writer};

/// Struct types that are "special serialized" as raw bytes rather than as a
/// nested tagged-property list (see RocketRP's `ISpecialSerialized`).
const SPECIAL_STRUCT_TYPES: [&str; 3] = ["Guid", "Vector", "Rotator"];

const OBJ_HEADER: u32 = 0xFFFF_FFFF;

/// An ordered list of tagged properties. Duplicate names are allowed (fixed
/// size arrays repeat the property name with different value indexes).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PropertyList {
    pub properties: Vec<Property>,
}

impl PropertyList {
    /// First property whose name matches `name` case-insensitively.
    pub fn get(&self, name: &str) -> Option<&Property> {
        self.properties
            .iter()
            .find(|property| property.name.is_ignore_case(name))
    }

    /// Mutable variant of [`PropertyList::get`].
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Property> {
        self.properties
            .iter_mut()
            .find(|property| property.name.is_ignore_case(name))
    }

    /// Replace the value of `name` in place (preserving its position), or
    /// append a new property if absent. Clears any recorded length mismatch.
    pub fn set(&mut self, name: &str, value: PropertyValue) {
        match self.get_mut(name) {
            Some(property) => {
                property.value = value;
                property.declared_length = None;
            }
            None => self.properties.push(Property {
                name: UeString::new(name),
                index: 0,
                declared_length: None,
                value,
            }),
        }
    }

    /// Remove all properties named `name` (case-insensitive).
    pub fn remove(&mut self, name: &str) {
        self.properties
            .retain(|property| !property.name.is_ignore_case(name));
    }

    /// Parse a tagged-property list up to and including its `None`
    /// terminator.
    pub fn parse(reader: &mut Reader) -> Result<PropertyList> {
        let mut properties = Vec::new();
        loop {
            let (name, name_value) = reader.ue_string_non_null()?;
            if name_value == "None" {
                return Ok(PropertyList { properties });
            }
            properties.push(Property::parse_after_name(reader, name)?);
        }
    }

    /// Parse a byte slice that must contain exactly one property list.
    fn parse_exact(data: &[u8]) -> Result<PropertyList> {
        let mut reader = Reader::new(data);
        let list = PropertyList::parse(&mut reader)?;
        if !reader.is_empty() {
            return Err(Error::TrailingGarbage(reader.remaining()));
        }
        Ok(list)
    }

    /// Serialize this list including its `None` terminator.
    pub fn serialize(&self, writer: &mut Writer) -> Result<()> {
        for property in &self.properties {
            property.serialize(writer)?;
        }
        writer.str("None")
    }

    fn serialized_bytes(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new();
        self.serialize(&mut writer)?;
        Ok(writer.into_bytes())
    }
}

/// One tagged property.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Property {
    pub name: UeString,
    /// The value index (nonzero only for fixed-size array entries).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub index: i32,
    /// The declared value length from the file, recorded only when it
    /// differs from the length we would compute (a known quirk for
    /// `StrProperty` values). Serialization writes this value back when
    /// present so round-trips stay byte-faithful.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_length: Option<i32>,
    pub value: PropertyValue,
}

fn is_zero(value: &i32) -> bool {
    *value == 0
}

/// The value of a tagged property, by property type tag.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    /// `BoolProperty`. The raw byte is preserved (the game writes 0/1).
    Bool(u8),
    /// `IntProperty` (also used for enums with a `u32` underlying type).
    Int(i32),
    /// `QWordProperty`.
    QWord(u64),
    /// `FloatProperty`.
    Float(f32),
    /// `StrProperty`.
    Str(UeString),
    /// `NameProperty`.
    Name(UeString),
    /// `ByteProperty`: either a raw byte or an enum value.
    Byte(ByteValue),
    /// `ObjectProperty`: an index into the container's object list.
    Object(i32),
    /// `StructProperty`.
    Struct(StructValue),
    /// `ArrayProperty` (dynamic arrays; fixed-size arrays appear as repeated
    /// scalar properties with distinct indexes).
    Array(ArrayValue),
    /// A property type this crate does not know; the value bytes are kept
    /// verbatim.
    Unknown(UnknownValue),
}

/// `ByteProperty` payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ByteValue {
    /// Serialized as the string `"None"` followed by one byte.
    Raw(u8),
    /// Serialized as the enum type name followed by the enum value name.
    Enum {
        enum_type: UeString,
        value: UeString,
    },
}

/// `StructProperty` payload: the struct type name plus its body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructValue {
    pub struct_type: UeString,
    pub body: StructBody,
}

/// A struct body: either raw bytes (for "special serialized" structs such as
/// `Guid`, or anything that fails to parse as tagged properties) or a nested
/// tagged-property list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StructBody {
    Raw(#[serde(with = "hex_bytes")] Vec<u8>),
    Properties(PropertyList),
}

/// Elements of a dynamic array. The element type is not recorded in the file
/// (readers are expected to know it), so parsing uses name-based hints for
/// the training-pack fields plus conservative sniffing, falling back to raw
/// bytes; every representation re-serializes to the original bytes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArrayValue {
    Ints(Vec<i32>),
    Strings(Vec<UeString>),
    /// Value-type struct elements: consecutive tagged-property lists with no
    /// per-element header.
    Structs(Vec<PropertyList>),
    /// Class (non-value-type) elements: each is prefixed with its full class
    /// name and a `0xFFFFFFFF` object header.
    Objects(Vec<ArrayObject>),
    Raw {
        count: i32,
        #[serde(with = "hex_bytes")]
        data: Vec<u8>,
    },
}

/// One class-typed dynamic array element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayObject {
    pub class: UeString,
    pub properties: PropertyList,
}

/// A property with an unrecognized type tag, preserved verbatim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnknownValue {
    pub type_name: UeString,
    #[serde(with = "hex_bytes")]
    pub data: Vec<u8>,
}

impl Property {
    fn parse_after_name(reader: &mut Reader, name: UeString) -> Result<Property> {
        let (type_tag, type_name) = reader.ue_string_non_null()?;
        let declared = reader.i32()?;
        let index = reader.i32()?;
        let declared_usize = usize::try_from(declared).map_err(|_| Error::BadValueLength {
            name: name.as_str().unwrap_or_default().to_string(),
            declared,
            reason: "negative".to_string(),
        })?;

        let value = match type_name.as_str() {
            "BoolProperty" => PropertyValue::Bool(reader.u8()?),
            "IntProperty" => PropertyValue::Int(reader.i32()?),
            "QWordProperty" => PropertyValue::QWord(reader.u64()?),
            "FloatProperty" => PropertyValue::Float(reader.f32()?),
            "StrProperty" => PropertyValue::Str(reader.ue_string()?),
            "NameProperty" => PropertyValue::Name(reader.ue_string()?),
            "ByteProperty" => {
                let (first, first_value) = reader.ue_string_non_null()?;
                if first_value == "None" {
                    PropertyValue::Byte(ByteValue::Raw(reader.u8()?))
                } else {
                    PropertyValue::Byte(ByteValue::Enum {
                        enum_type: first,
                        value: reader.ue_string()?,
                    })
                }
            }
            "ObjectProperty" => PropertyValue::Object(reader.i32()?),
            "StructProperty" => {
                let struct_type = reader.ue_string()?;
                let body_bytes = reader.bytes(declared_usize)?;
                let body = if struct_type
                    .as_str()
                    .is_some_and(|t| SPECIAL_STRUCT_TYPES.contains(&t))
                {
                    StructBody::Raw(body_bytes.to_vec())
                } else {
                    match PropertyList::parse_exact(body_bytes) {
                        Ok(list) => StructBody::Properties(list),
                        Err(_) => StructBody::Raw(body_bytes.to_vec()),
                    }
                };
                PropertyValue::Struct(StructValue { struct_type, body })
            }
            "ArrayProperty" => {
                let payload = reader.bytes(declared_usize)?;
                PropertyValue::Array(parse_array(&name, payload)?)
            }
            _ => {
                let data = reader.bytes(declared_usize)?.to_vec();
                PropertyValue::Unknown(UnknownValue {
                    type_name: type_tag,
                    data,
                })
            }
        };

        let computed = value.computed_length()?;
        let declared_length = (declared != computed).then_some(declared);
        Ok(Property {
            name,
            index,
            declared_length,
            value,
        })
    }

    fn serialize(&self, writer: &mut Writer) -> Result<()> {
        writer.ue_string(&self.name)?;
        writer.str(self.value.type_tag())?;
        writer.i32(
            self.declared_length
                .map_or_else(|| self.value.computed_length(), Ok)?,
        );
        writer.i32(self.index);
        self.value.serialize_body(writer)
    }
}

impl PropertyValue {
    /// The property type tag string this value serializes under.
    pub fn type_tag(&self) -> &str {
        match self {
            PropertyValue::Bool(_) => "BoolProperty",
            PropertyValue::Int(_) => "IntProperty",
            PropertyValue::QWord(_) => "QWordProperty",
            PropertyValue::Float(_) => "FloatProperty",
            PropertyValue::Str(_) => "StrProperty",
            PropertyValue::Name(_) => "NameProperty",
            PropertyValue::Byte(_) => "ByteProperty",
            PropertyValue::Object(_) => "ObjectProperty",
            PropertyValue::Struct(_) => "StructProperty",
            PropertyValue::Array(_) => "ArrayProperty",
            PropertyValue::Unknown(unknown) => unknown.type_name.as_str().unwrap_or_default(),
        }
    }

    /// The declared value length this value would be written with, honoring
    /// the reference implementation's quirks.
    fn computed_length(&self) -> Result<i32> {
        let length = match self {
            PropertyValue::Bool(_) => 0,
            PropertyValue::Int(_) | PropertyValue::Object(_) => 4,
            PropertyValue::QWord(_) => 8,
            PropertyValue::Float(_) => 4,
            PropertyValue::Str(value) | PropertyValue::Name(value) => value.serialized_len(),
            PropertyValue::Byte(ByteValue::Raw(_)) => 1,
            PropertyValue::Byte(ByteValue::Enum { value, .. }) => value.serialized_len(),
            PropertyValue::Struct(value) => match &value.body {
                StructBody::Raw(data) => data.len(),
                StructBody::Properties(list) => list.serialized_bytes()?.len(),
            },
            PropertyValue::Array(value) => value.serialized_bytes()?.len(),
            PropertyValue::Unknown(unknown) => unknown.data.len(),
        };
        Ok(i32::try_from(length).expect("property value length fits in i32"))
    }

    fn serialize_body(&self, writer: &mut Writer) -> Result<()> {
        match self {
            PropertyValue::Bool(value) => writer.u8(*value),
            PropertyValue::Int(value) => writer.i32(*value),
            PropertyValue::QWord(value) => writer.u64(*value),
            PropertyValue::Float(value) => writer.f32(*value),
            PropertyValue::Str(value) | PropertyValue::Name(value) => {
                writer.ue_string(value)?;
            }
            PropertyValue::Byte(ByteValue::Raw(value)) => {
                writer.str("None")?;
                writer.u8(*value);
            }
            PropertyValue::Byte(ByteValue::Enum { enum_type, value }) => {
                writer.ue_string(enum_type)?;
                writer.ue_string(value)?;
            }
            PropertyValue::Object(value) => writer.i32(*value),
            PropertyValue::Struct(value) => {
                writer.ue_string(&value.struct_type)?;
                match &value.body {
                    StructBody::Raw(data) => writer.bytes(data),
                    StructBody::Properties(list) => list.serialize(writer)?,
                }
            }
            PropertyValue::Array(value) => {
                writer.bytes(&value.serialized_bytes()?);
            }
            PropertyValue::Unknown(unknown) => writer.bytes(&unknown.data),
        }
        Ok(())
    }
}

impl ArrayValue {
    pub fn len(&self) -> usize {
        match self {
            ArrayValue::Ints(items) => items.len(),
            ArrayValue::Strings(items) => items.len(),
            ArrayValue::Structs(items) => items.len(),
            ArrayValue::Objects(items) => items.len(),
            ArrayValue::Raw { count, .. } => usize::try_from(*count).unwrap_or(0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn serialized_bytes(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new();
        match self {
            ArrayValue::Ints(items) => {
                writer.i32(i32::try_from(items.len()).unwrap());
                for item in items {
                    writer.i32(*item);
                }
            }
            ArrayValue::Strings(items) => {
                writer.i32(i32::try_from(items.len()).unwrap());
                for item in items {
                    writer.ue_string(item)?;
                }
            }
            ArrayValue::Structs(items) => {
                writer.i32(i32::try_from(items.len()).unwrap());
                for item in items {
                    item.serialize(&mut writer)?;
                }
            }
            ArrayValue::Objects(items) => {
                writer.i32(i32::try_from(items.len()).unwrap());
                for item in items {
                    writer.ue_string(&item.class)?;
                    writer.u32(OBJ_HEADER);
                    item.properties.serialize(&mut writer)?;
                }
            }
            ArrayValue::Raw { count, data } => {
                writer.i32(*count);
                writer.bytes(data);
            }
        }
        Ok(writer.into_bytes())
    }
}

/// Element-type hints for the dynamic array fields the training-pack schema
/// is known to use.
fn array_hint(name: &UeString) -> Option<ArrayKind> {
    let name = name.as_str()?;
    if name.eq_ignore_ascii_case("Rounds") {
        Some(ArrayKind::Structs)
    } else if name.eq_ignore_ascii_case("SerializedArchetypes") {
        Some(ArrayKind::Strings)
    } else if name.eq_ignore_ascii_case("Tags") {
        Some(ArrayKind::Ints)
    } else {
        None
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ArrayKind {
    Ints,
    Strings,
    Structs,
    Objects,
}

fn parse_array(name: &UeString, payload: &[u8]) -> Result<ArrayValue> {
    let mut reader = Reader::new(payload);
    let count = reader.i32()?;
    let elements = &payload[4..];
    let count_usize = usize::try_from(count).unwrap_or(0);

    if count <= 0 {
        return Ok(ArrayValue::Raw {
            count,
            data: elements.to_vec(),
        });
    }

    let candidates: Vec<ArrayKind> = match array_hint(name) {
        Some(kind) => vec![kind],
        None => vec![
            ArrayKind::Ints,
            ArrayKind::Strings,
            ArrayKind::Structs,
            ArrayKind::Objects,
        ],
    };

    for kind in candidates {
        if let Some(value) = try_parse_elements(kind, count_usize, elements) {
            return Ok(value);
        }
    }

    Ok(ArrayValue::Raw {
        count,
        data: elements.to_vec(),
    })
}

/// Attempt to parse `data` as exactly `count` elements of `kind`; `None` if
/// the bytes do not fit that interpretation exactly.
fn try_parse_elements(kind: ArrayKind, count: usize, data: &[u8]) -> Option<ArrayValue> {
    let mut reader = Reader::new(data);
    let value = match kind {
        ArrayKind::Ints => {
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                items.push(reader.i32().ok()?);
            }
            ArrayValue::Ints(items)
        }
        ArrayKind::Strings => {
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                items.push(reader.ue_string().ok()?);
            }
            ArrayValue::Strings(items)
        }
        ArrayKind::Structs => {
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                items.push(PropertyList::parse(&mut reader).ok()?);
            }
            ArrayValue::Structs(items)
        }
        ArrayKind::Objects => {
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                let class = reader.ue_string().ok()?;
                class.as_str()?;
                if reader.u32().ok()? != OBJ_HEADER {
                    return None;
                }
                items.push(ArrayObject {
                    class,
                    properties: PropertyList::parse(&mut reader).ok()?,
                });
            }
            ArrayValue::Objects(items)
        }
    };
    reader.is_empty().then_some(value)
}

/// Serde helper: `Vec<u8>` as a lowercase hex string.
pub(crate) mod hex_bytes {
    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        let mut out = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            out.push_str(&format!("{byte:02x}"));
        }
        serializer.serialize_str(&out)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let text = String::deserialize(deserializer)?;
        if text.len() % 2 != 0 {
            return Err(D::Error::custom("odd-length hex string"));
        }
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).map_err(D::Error::custom))
            .collect()
    }
}

#[cfg(test)]
#[path = "property_tests.rs"]
mod tests;
