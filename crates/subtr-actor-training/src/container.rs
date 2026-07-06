//! The save data container: encrypted envelope, version header, savedata
//! blob, and object table.
//!
//! On-disk layout of a `.tem` (or `.save`) file:
//!
//! ```text
//! u32 encrypted_length            // padded to a 16-byte boundary
//! u32 crc32(ciphertext, 0xEFCBF201)
//! [encrypted_length bytes]        // AES-256-ECB, hardcoded key
//! ```
//!
//! Decrypted payload:
//!
//! ```text
//! u32 0xF005BA11
//! u32 0x7FFFFFFF
//! i32 engine_version  (868)
//! i32 licensee_version (32)
//! i32 type_version     (0)
//! i32 savedata_length              // includes this field itself
//! [savedata_length - 4 bytes]      // the savedata blob
//! i32 num_object_types
//! per object: string type name, u32 file_position, u32 object_index
//! ```
//!
//! The savedata blob is `u32 0xFFFFFFFF` followed by the root object's
//! tagged properties, then each listed object as `0xFFFFFFFF` + tagged
//! properties, back to back. Each table entry's `file_position` equals the
//! blob offset of that object's `0xFFFFFFFF` header plus 4 (positions are
//! measured from the `savedata_length` field). `ObjectProperty` values are
//! indexes into the object table.

use serde::{Deserialize, Serialize};

use crate::crc32::{SAVE_DATA_CRC_SEED, crc32};
use crate::crypto;
use crate::error::{Error, Result};
use crate::io::{Reader, UeString, Writer};
use crate::property::PropertyList;

const FOOSBALL: u32 = 0xF005_BA11;
const MAGIC: u32 = 0x7FFF_FFFF;
const OBJ_HEADER: u32 = 0xFFFF_FFFF;
const AES_BLOCK: usize = 16;

/// Save data version header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionInfo {
    pub engine: i32,
    pub licensee: i32,
    pub type_version: i32,
}

impl Default for VersionInfo {
    fn default() -> Self {
        VersionInfo {
            engine: 868,
            licensee: 32,
            type_version: 0,
        }
    }
}

/// One entry of the container's object list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveObject {
    /// Fully qualified type name, e.g. `TAGame.TrainingEditorData_TA`.
    pub type_name: UeString,
    /// The index recorded in the object table (normally its position).
    pub object_index: u32,
    pub properties: PropertyList,
}

/// A parsed save data container.
///
/// This is a fully generic representation — the root object and every listed
/// object are kept as ordered property trees, so files with unknown root
/// types or unknown properties round-trip byte-faithfully. The lossless JSON
/// representation is this type's serde form.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrainingFile {
    pub version: VersionInfo,
    /// The root object's properties (for `.tem` files this is
    /// `SaveData_GameEditor_Training_TA`; its type name is not stored in the
    /// file).
    pub root: PropertyList,
    pub objects: Vec<SaveObject>,
}

impl TrainingFile {
    /// Decode an encrypted save data file (the on-disk `.tem` format).
    pub fn from_bytes(data: &[u8]) -> Result<TrainingFile> {
        let mut reader = Reader::new(data);
        if data.len() < 8 {
            return Err(Error::EnvelopeTooShort);
        }
        let length = reader.u32()? as usize;
        let stored_crc = reader.u32()?;
        if reader.remaining() < length {
            return Err(Error::EncryptedLengthOutOfRange {
                declared: length,
                available: reader.remaining(),
            });
        }
        let ciphertext = reader.bytes(length)?;
        let computed_crc = crc32(ciphertext, SAVE_DATA_CRC_SEED);
        if computed_crc != stored_crc {
            return Err(Error::CrcMismatch {
                stored: stored_crc,
                computed: computed_crc,
            });
        }
        let payload = crypto::decrypt(ciphertext)?;
        Self::from_decrypted_payload(&payload)
    }

    /// Decode an already-decrypted payload (no envelope, no CRC). Trailing
    /// zero bytes (AES padding remnants) are tolerated.
    pub fn from_decrypted_payload(payload: &[u8]) -> Result<TrainingFile> {
        let mut reader = Reader::new(payload);
        let foosball = reader.u32()?;
        let magic = reader.u32()?;
        if foosball != FOOSBALL || magic != MAGIC {
            return Err(Error::BadMagic {
                found_foosball: foosball,
                found_magic: magic,
            });
        }
        let version = VersionInfo {
            engine: reader.i32()?,
            licensee: reader.i32()?,
            type_version: reader.i32()?,
        };

        let savedata_length = reader.i32()?;
        let blob_length = usize::try_from(savedata_length)
            .ok()
            .and_then(|length| length.checked_sub(4))
            .ok_or(Error::BadSaveDataLength(savedata_length))?;
        let blob = reader.bytes(blob_length)?;

        let num_objects = reader.i32()?;
        let num_objects =
            usize::try_from(num_objects).map_err(|_| Error::BadSaveDataLength(num_objects))?;
        let mut table = Vec::with_capacity(num_objects);
        for _ in 0..num_objects {
            let type_name = reader.ue_string()?;
            let file_position = reader.u32()?;
            let object_index = reader.u32()?;
            table.push((type_name, file_position, object_index));
        }

        // Anything after the object table must be AES zero padding.
        if reader.remaining() >= AES_BLOCK || !payload[reader.pos()..].iter().all(|&byte| byte == 0)
        {
            return Err(Error::TrailingGarbage(reader.remaining()));
        }

        // Parse the blob: root object, then each listed object, contiguously.
        let mut blob_reader = Reader::new(blob);
        expect_obj_header(&mut blob_reader)?;
        let root = PropertyList::parse(&mut blob_reader)?;

        let mut objects = Vec::with_capacity(table.len());
        for (index, (type_name, file_position, object_index)) in table.into_iter().enumerate() {
            // file_position - 4 is the blob offset of the object's header.
            let expected = blob_reader.pos();
            let found = usize::try_from(file_position)
                .ok()
                .and_then(|position| position.checked_sub(4))
                .ok_or(Error::NonContiguousObjects {
                    index,
                    expected,
                    found: file_position as usize,
                })?;
            if found != expected {
                return Err(Error::NonContiguousObjects {
                    index,
                    expected,
                    found,
                });
            }
            expect_obj_header(&mut blob_reader)?;
            objects.push(SaveObject {
                type_name,
                object_index,
                properties: PropertyList::parse(&mut blob_reader)?,
            });
        }
        if !blob_reader.is_empty() {
            return Err(Error::TrailingGarbage(blob_reader.remaining()));
        }

        Ok(TrainingFile {
            version,
            root,
            objects,
        })
    }

    /// Serialize the decrypted payload (without AES padding).
    pub fn to_decrypted_payload(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new();
        writer.u32(FOOSBALL);
        writer.u32(MAGIC);
        writer.i32(self.version.engine);
        writer.i32(self.version.licensee);
        writer.i32(self.version.type_version);

        let length_field_offset = writer.len();
        writer.u32(0); // savedata length, backpatched below
        writer.u32(OBJ_HEADER);
        self.root.serialize(&mut writer)?;

        let mut positions = Vec::with_capacity(self.objects.len());
        for object in &self.objects {
            positions.push(
                u32::try_from(writer.len() - length_field_offset).expect("blob offset fits in u32"),
            );
            writer.u32(OBJ_HEADER);
            object.properties.serialize(&mut writer)?;
        }

        let savedata_length = writer.len() - length_field_offset;
        writer.patch_u32(
            length_field_offset,
            u32::try_from(savedata_length).expect("savedata length fits in u32"),
        );

        writer.i32(i32::try_from(self.objects.len()).unwrap());
        for (object, position) in self.objects.iter().zip(positions) {
            writer.ue_string(&object.type_name)?;
            writer.u32(position);
            writer.u32(object.object_index);
        }
        Ok(writer.into_bytes())
    }

    /// Serialize the full encrypted `.tem` file (envelope + ciphertext).
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let payload = self.to_decrypted_payload()?;
        let ciphertext = crypto::encrypt(&payload);
        let mut writer = Writer::new();
        writer.u32(u32::try_from(ciphertext.len()).expect("length fits in u32"));
        writer.u32(crc32(&ciphertext, SAVE_DATA_CRC_SEED));
        writer.bytes(&ciphertext);
        Ok(writer.into_bytes())
    }

    /// Lossless JSON representation (includes unknown properties).
    pub fn to_json(&self, pretty: bool) -> Result<String> {
        Ok(if pretty {
            serde_json::to_string_pretty(self)?
        } else {
            serde_json::to_string(self)?
        })
    }

    /// Parse the lossless JSON representation back into a container.
    pub fn from_json(json: &str) -> Result<TrainingFile> {
        Ok(serde_json::from_str(json)?)
    }
}

fn expect_obj_header(reader: &mut Reader) -> Result<()> {
    let offset = reader.pos();
    let found = reader.u32()?;
    if found != OBJ_HEADER {
        return Err(Error::BadObjectHeader { offset, found });
    }
    Ok(())
}

#[cfg(test)]
#[path = "container_tests.rs"]
mod tests;
