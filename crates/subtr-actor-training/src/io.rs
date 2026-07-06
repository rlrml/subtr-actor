//! Little-endian byte cursor helpers and the UE3 length-prefixed string type.
//!
//! UE3 strings are serialized as an `i32` length followed by the character
//! data:
//!
//! * `length > 0`: `length` bytes of windows-1252 text, the last of which is
//!   a NUL terminator.
//! * `length < 0`: `-2 * length` bytes of UTF-16LE text, the last two of
//!   which are a NUL terminator.
//! * `length == 0`: a null string (distinct from the empty string, which is
//!   serialized as `length == 1` plus a NUL byte).
//!
//! The game's writer picks UTF-16 iff any UTF-16 code unit of the value is
//! `> 0xFF` — note this is *not* "not windows-1252-representable": `€`
//! (U+20AC) is representable in windows-1252, but is still written as UTF-16
//! when authored fresh. Because of that, decoding a windows-1252 string can
//! yield characters `> 0xFF` (bytes 0x80–0x9F), so we must remember the
//! encoding each parsed string actually used to keep round-trips
//! byte-faithful; [`UeString`] stores it explicitly.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{Error, Result};

/// A UE3 string together with the encoding it was (or will be) serialized in.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UeString {
    /// Serialized as length 0.
    Null,
    /// Serialized as windows-1252 with a trailing NUL.
    Ansi(String),
    /// Serialized as UTF-16LE with a trailing NUL.
    Utf16(String),
}

impl UeString {
    /// Build a string, choosing the encoding the way the game's writer does:
    /// UTF-16 iff any UTF-16 code unit is greater than `0xFF`.
    pub fn new(value: &str) -> UeString {
        if value.encode_utf16().any(|unit| unit > 0xFF) {
            UeString::Utf16(value.to_string())
        } else {
            UeString::Ansi(value.to_string())
        }
    }

    /// The string value, or `None` for the null string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            UeString::Null => None,
            UeString::Ansi(value) | UeString::Utf16(value) => Some(value),
        }
    }

    /// Whether this string equals `other` (null never matches).
    pub fn is(&self, other: &str) -> bool {
        self.as_str() == Some(other)
    }

    /// Case-insensitive comparison, mirroring the reference implementation's
    /// case-insensitive property lookup.
    pub fn is_ignore_case(&self, other: &str) -> bool {
        self.as_str().is_some_and(|s| s.eq_ignore_ascii_case(other))
    }

    /// Build from an optional value (None becomes the null string).
    pub fn from_option(value: Option<&str>) -> UeString {
        match value {
            None => UeString::Null,
            Some(value) => UeString::new(value),
        }
    }

    /// The number of bytes this string occupies when serialized (including
    /// the `i32` length prefix and NUL terminator).
    pub fn serialized_len(&self) -> usize {
        match self {
            UeString::Null => 4,
            UeString::Ansi(value) => 4 + value.chars().count() + 1,
            UeString::Utf16(value) => 4 + (value.encode_utf16().count() + 1) * 2,
        }
    }
}

// Serde representation: null for Null, a plain JSON string for Ansi, and
// {"utf16": "..."} for Utf16, keeping decoded JSON readable while staying
// lossless.
impl Serialize for UeString {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            UeString::Null => serializer.serialize_none(),
            UeString::Ansi(value) => serializer.serialize_str(value),
            UeString::Utf16(value) => {
                #[derive(Serialize)]
                struct Utf16Repr<'a> {
                    utf16: &'a str,
                }
                Utf16Repr { utf16: value }.serialize(serializer)
            }
        }
    }
}

impl<'de> Deserialize<'de> for UeString {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Null,
            Plain(String),
            Utf16 { utf16: String },
        }
        Ok(match Repr::deserialize(deserializer)? {
            Repr::Null => UeString::Null,
            Repr::Plain(value) => {
                // Reject plain strings that would not round-trip as ANSI so
                // JSON edits cannot silently change meaning.
                if value.encode_utf16().any(|unit| unit > 0xFF)
                    && encoding_rs::WINDOWS_1252.encode(&value).2
                {
                    return Err(D::Error::custom(format!(
                        "string {value:?} is not windows-1252 encodable; use {{\"utf16\": ...}}"
                    )));
                }
                UeString::Ansi(value)
            }
            Repr::Utf16 { utf16 } => UeString::Utf16(utf16),
        })
    }
}

/// A little-endian read cursor over a byte slice.
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Reader<'a> {
        Reader { data, pos: 0 }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub fn bytes(&mut self, count: usize) -> Result<&'a [u8]> {
        if self.remaining() < count {
            return Err(Error::UnexpectedEof {
                offset: self.pos,
                needed: count - self.remaining(),
            });
        }
        let slice = &self.data[self.pos..self.pos + count];
        self.pos += count;
        Ok(slice)
    }

    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.bytes(1)?[0])
    }

    pub fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }

    pub fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }

    pub fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.bytes(8)?.try_into().unwrap()))
    }

    pub fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }

    /// Read a UE3 length-prefixed string, preserving its encoding.
    pub fn ue_string(&mut self) -> Result<UeString> {
        let start = self.pos;
        let length = self.i32()?;
        if length == 0 {
            return Ok(UeString::Null);
        }
        if length > 0 {
            let bytes = self.bytes(length as usize)?;
            let (terminator, content) = bytes.split_last().unwrap();
            if *terminator != 0 {
                return Err(Error::StringNotNulTerminated(start));
            }
            let (decoded, _) = encoding_rs::WINDOWS_1252.decode_without_bom_handling(content);
            Ok(UeString::Ansi(decoded.into_owned()))
        } else {
            let unit_count = length
                .checked_mul(-2)
                .and_then(|n| usize::try_from(n).ok())
                .ok_or(Error::InvalidUtf16(start))?
                / 2;
            let bytes = self.bytes(unit_count * 2)?;
            let units: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .collect();
            let (terminator, content) = units.split_last().unwrap();
            if *terminator != 0 {
                return Err(Error::StringNotNulTerminated(start));
            }
            let decoded = String::from_utf16(content).map_err(|_| Error::InvalidUtf16(start))?;
            Ok(UeString::Utf16(decoded))
        }
    }

    /// Read a UE3 string that must not be null (e.g. a property name).
    pub fn ue_string_non_null(&mut self) -> Result<(UeString, String)> {
        let start = self.pos;
        let string = self.ue_string()?;
        match string.as_str() {
            Some(value) => {
                let value = value.to_string();
                Ok((string, value))
            }
            None => Err(Error::NullString(start)),
        }
    }
}

/// A little-endian write cursor.
#[derive(Default)]
pub struct Writer {
    data: Vec<u8>,
}

impl Writer {
    pub fn new() -> Writer {
        Writer::default()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn u8(&mut self, value: u8) {
        self.data.push(value);
    }

    pub fn u32(&mut self, value: u32) {
        self.bytes(&value.to_le_bytes());
    }

    pub fn i32(&mut self, value: i32) {
        self.bytes(&value.to_le_bytes());
    }

    pub fn u64(&mut self, value: u64) {
        self.bytes(&value.to_le_bytes());
    }

    pub fn f32(&mut self, value: f32) {
        self.bytes(&value.to_le_bytes());
    }

    /// Overwrite 4 bytes at `offset` with `value` (for length backpatching).
    pub fn patch_u32(&mut self, offset: usize, value: u32) {
        self.data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    /// Write a UE3 length-prefixed string in its recorded encoding.
    pub fn ue_string(&mut self, string: &UeString) -> Result<()> {
        match string {
            UeString::Null => self.i32(0),
            UeString::Ansi(value) => {
                let (encoded, _, had_errors) = encoding_rs::WINDOWS_1252.encode(value);
                if had_errors {
                    return Err(Error::UnencodableAnsiString(value.clone()));
                }
                // encoding_rs produces one byte per char for windows-1252, so
                // the char count is the byte count.
                self.i32(i32::try_from(encoded.len() + 1).unwrap());
                self.bytes(&encoded);
                self.u8(0);
            }
            UeString::Utf16(value) => {
                let units: Vec<u16> = value.encode_utf16().collect();
                self.i32(-i32::try_from(units.len() + 1).unwrap());
                for unit in &units {
                    self.bytes(&unit.to_le_bytes());
                }
                self.bytes(&0u16.to_le_bytes());
            }
        }
        Ok(())
    }

    /// Write a plain `&str`, deriving the encoding like the game does.
    pub fn str(&mut self, value: &str) -> Result<()> {
        self.ue_string(&UeString::new(value))
    }
}

#[cfg(test)]
#[path = "io_tests.rs"]
mod tests;
