use std::backtrace::Backtrace;

#[path = "error_variant.rs"]
mod error_variant;

pub use error_variant::SubtrActorErrorVariant;

/// [`SubtrActorError`] struct provides an error variant
/// [`SubtrActorErrorVariant`] along with its backtrace.
#[derive(Debug)]
pub struct SubtrActorError {
    pub backtrace: Backtrace,
    pub variant: SubtrActorErrorVariant,
}

impl SubtrActorError {
    pub fn new(variant: SubtrActorErrorVariant) -> Self {
        Self {
            backtrace: Backtrace::capture(),
            variant,
        }
    }

    pub fn new_result<T>(variant: SubtrActorErrorVariant) -> Result<T, Self> {
        Err(Self::new(variant))
    }
}

#[allow(clippy::result_large_err)]
pub type SubtrActorResult<T> = Result<T, SubtrActorError>;
