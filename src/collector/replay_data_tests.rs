use super::*;

#[test]
fn missing_metadata_i32_defaults_to_zero() {
    let missing_seconds = SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
        property: SECONDS_REMAINING_KEY,
    });

    assert_eq!(metadata_i32_or_default(Err(missing_seconds)), 0);
}

#[test]
fn present_metadata_i32_is_preserved() {
    assert_eq!(metadata_i32_or_default(Ok(42)), 42);
}
