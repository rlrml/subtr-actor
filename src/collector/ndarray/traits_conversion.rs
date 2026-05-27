use crate::*;

/// Maps arbitrary conversion failures into a generic float-conversion error.
pub fn convert_float_conversion_error<T>(_: T) -> SubtrActorError {
    SubtrActorError::new(SubtrActorErrorVariant::FloatConversionError)
}

/// Converts a fixed list of values with a caller-supplied error mapper.
#[macro_export]
macro_rules! convert_all {
    ($err:expr, $( $item:expr ),* $(,)?) => {{
        Ok([
            $( $item.try_into().map_err($err)? ),*
        ])
    }};
}

/// Converts a fixed list of float-like values using [`convert_float_conversion_error`].
#[macro_export]
macro_rules! convert_all_floats {
    ($( $item:expr ),* $(,)?) => {{
        convert_all!(convert_float_conversion_error, $( $item ),*)
    }};
}
