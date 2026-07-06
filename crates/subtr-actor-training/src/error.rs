//! Error type for training-pack parsing and serialization.

/// Errors produced while decoding, editing, or encoding training pack files.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unexpected end of data at offset {offset}: needed {needed} more bytes")]
    UnexpectedEof { offset: usize, needed: usize },

    #[error("file too short to contain the encrypted envelope header")]
    EnvelopeTooShort,

    #[error("declared encrypted length {declared} exceeds available bytes {available}")]
    EncryptedLengthOutOfRange { declared: usize, available: usize },

    #[error("encrypted payload length {0} is not a multiple of the AES block size")]
    EncryptedLengthNotBlockAligned(usize),

    #[error("CRC mismatch: stored {stored:#010x}, computed {computed:#010x}")]
    CrcMismatch { stored: u32, computed: u32 },

    #[error(
        "bad magic values: expected f005ba11/7fffffff, got {found_foosball:#010x}/{found_magic:#010x}"
    )]
    BadMagic {
        found_foosball: u32,
        found_magic: u32,
    },

    #[error("bad object header at offset {offset}: expected ffffffff, got {found:#010x}")]
    BadObjectHeader { offset: usize, found: u32 },

    #[error("string is not NUL terminated at offset {0}")]
    StringNotNulTerminated(usize),

    #[error("invalid UTF-16 string data at offset {0}")]
    InvalidUtf16(usize),

    #[error("string cannot be encoded as windows-1252: {0:?}")]
    UnencodableAnsiString(String),

    #[error("unexpected null string at offset {0}")]
    NullString(usize),

    #[error("property {name:?}: declared value length {declared} is invalid ({reason})")]
    BadValueLength {
        name: String,
        declared: i32,
        reason: String,
    },

    #[error(
        "non-contiguous object layout: object {index} expected at blob offset {expected}, table says {found}"
    )]
    NonContiguousObjects {
        index: usize,
        expected: usize,
        found: usize,
    },

    #[error("trailing non-padding bytes after save data structure ({0} bytes)")]
    TrailingGarbage(usize),

    #[error("save data blob length field {0} is invalid")]
    BadSaveDataLength(i32),

    #[error("missing expected property {0:?}")]
    MissingProperty(String),

    #[error("property {name:?} has unexpected shape: {reason}")]
    UnexpectedPropertyShape { name: String, reason: String },

    #[error("root TrainingData object reference {index} is out of range ({count} objects)")]
    TrainingDataIndexOutOfRange { index: i32, count: usize },

    #[error("no TrainingEditorData_TA object found in this file")]
    NoTrainingData,

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Convenience result alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;
