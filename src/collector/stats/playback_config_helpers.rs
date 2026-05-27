use super::*;

pub(super) type JsonObject<'a> = Option<&'a Map<String, Value>>;

pub(super) fn module_config<'a>(config: &'a Map<String, Value>, module: &str) -> JsonObject<'a> {
    config.get(module).and_then(Value::as_object)
}

pub(super) fn f32_config(config: JsonObject<'_>, key: &str, default: f32) -> f32 {
    config
        .and_then(|config| config.get(key))
        .and_then(json_f32)
        .unwrap_or(default)
}

pub(super) fn f64_config(config: JsonObject<'_>, key: &str, default: f32) -> f64 {
    config
        .and_then(|config| config.get(key))
        .and_then(Value::as_f64)
        .unwrap_or(default as f64)
}

pub(super) fn f64_config_with_source_key(
    config: JsonObject<'_>,
    output_key: &'static str,
    source_key: &str,
    default: f32,
) -> (&'static str, f64) {
    (output_key, f64_config(config, source_key, default))
}

pub(super) fn insert_f64_config(
    output: &mut Map<String, Value>,
    key: &str,
    value: f64,
) -> SubtrActorResult<()> {
    output.insert(key.to_owned(), serialize_to_json_value(&value)?);
    Ok(())
}

pub(super) fn insert_config_pairs(
    output: &mut Map<String, Value>,
    pairs: &[(&str, f64)],
) -> SubtrActorResult<()> {
    for (key, value) in pairs {
        insert_f64_config(output, key, *value)?;
    }
    Ok(())
}
