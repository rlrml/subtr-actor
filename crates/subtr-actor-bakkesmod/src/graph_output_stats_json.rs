use super::*;

pub(crate) unsafe fn serialize_named_stats_module(
    engine: *const SaEngine,
    module_name: *const c_char,
) -> Vec<u8> {
    let Some(engine) = engine.as_ref() else {
        return Vec::new();
    };
    let Some(module_name) = c_string_arg(module_name) else {
        return Vec::new();
    };
    match builtin_stats_module_json(&module_name, &engine.graph) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub(crate) unsafe fn serialize_named_stats_module_frame(
    engine: *const SaEngine,
    module_name: *const c_char,
) -> Vec<u8> {
    let Some(engine) = engine.as_ref() else {
        return Vec::new();
    };
    let Some(module_name) = c_string_arg(module_name) else {
        return Vec::new();
    };
    let Some(replay_meta) = engine.live_replay_meta.as_ref() else {
        return Vec::new();
    };
    match builtin_stats_module_frame_json(&module_name, &engine.graph, replay_meta) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub(crate) unsafe fn serialize_named_stats_module_config(
    engine: *const SaEngine,
    module_name: *const c_char,
) -> Vec<u8> {
    let Some(engine) = engine.as_ref() else {
        return Vec::new();
    };
    let Some(module_name) = c_string_arg(module_name) else {
        return Vec::new();
    };
    match builtin_stats_module_config_json(&module_name, &engine.graph) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}
