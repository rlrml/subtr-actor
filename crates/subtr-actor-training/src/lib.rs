//! Parse, edit, and serialize Rocket League custom training pack files
//! (`.tem`), and more generally the encrypted UE3 save data container the
//! game uses for editor save data (`.save`).
//!
//! The format was reverse engineered by the
//! [RocketRP](https://github.com/Drogebot/RocketRP) project; this crate is a
//! from-scratch implementation with one deliberate departure: instead of
//! reflecting into a fixed schema (and failing on anything unknown), files
//! are parsed into a generic ordered property tree that preserves unknown
//! properties and object types, with byte-faithful round-trips as the core
//! invariant.
//!
//! # Layers
//!
//! * [`TrainingFile`] — the container: decrypt/parse ([`TrainingFile::from_bytes`]),
//!   serialize/encrypt ([`TrainingFile::to_bytes`]), and a lossless JSON
//!   representation via serde.
//! * [`PropertyList`] / [`PropertyValue`] — the generic tagged-property tree.
//! * [`TrainingPack`] — a typed view mirroring `TrainingEditorData_TA`
//!   (name, code, difficulty, rounds, ...), with edit methods on
//!   [`TrainingFile`] that write back into the tree so unknown data
//!   survives edits.
//! * [`Archetype`] — structured parsing/editing of the per-round
//!   `SerializedArchetypes` strings (ball, car spawn point, player car).
//!   Parsing is on demand and editing regenerates only the string being
//!   modified; untouched strings are preserved exactly.
//!
//! # Example
//!
//! ```no_run
//! use subtr_actor_training::TrainingFile;
//!
//! let bytes = std::fs::read("pack.tem").unwrap();
//! let mut file = TrainingFile::from_bytes(&bytes).unwrap();
//! let pack = file.pack().unwrap();
//! println!("{:?} by {:?}: {} rounds", pack.name, pack.creator_name, pack.rounds.len());
//! file.set_name(Some("My edited pack")).unwrap();
//! std::fs::write("edited.tem", file.to_bytes().unwrap()).unwrap();
//! ```

pub mod archetype;
pub mod container;
pub mod crc32;
pub mod crypto;
pub mod error;
pub mod io;
pub mod pack;
pub mod property;

pub use archetype::{Archetype, BallSpawn, CarSpawn, PlayerCarSpawn};
pub use container::{SaveObject, TrainingFile, VersionInfo};
pub use error::{Error, Result};
pub use io::UeString;
pub use pack::{Difficulty, Guid, PlayerId, Round, TrainingPack, TrainingType};
pub use property::{
    ArrayObject, ArrayValue, ByteValue, Property, PropertyList, PropertyValue, StructBody,
    StructValue, UnknownValue,
};
