use js_sys::Function;
use subtr_actor::{SubtrActorError, SubtrActorErrorVariant, SubtrActorResult};
use wasm_bindgen::prelude::*;

pub(crate) fn emit_progress(
    callback: &Function,
    stage: &str,
    processed_frames: usize,
    total_frames: usize,
) -> SubtrActorResult<()> {
    let progress = if total_frames == 0 {
        1.0
    } else {
        processed_frames as f64 / total_frames as f64
    };
    let payload = serde_wasm_bindgen::to_value(&serde_json::json!({
        "stage": stage,
        "processedFrames": processed_frames,
        "totalFrames": total_frames,
        "progress": progress,
    }))
    .map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
            "Failed to serialize progress payload: {error}"
        )))
    })?;

    callback.call1(&JsValue::NULL, &payload).map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
            error
                .as_string()
                .unwrap_or_else(|| "Progress callback threw a non-string error".to_string()),
        ))
    })?;
    Ok(())
}

pub(crate) fn emit_stage_progress(
    callback: &Function,
    stage: &str,
    progress: f64,
) -> SubtrActorResult<()> {
    let payload = serde_wasm_bindgen::to_value(&serde_json::json!({
        "stage": stage,
        "progress": progress.clamp(0.0, 1.0),
    }))
    .map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
            "Failed to serialize progress payload: {error}"
        )))
    })?;

    callback.call1(&JsValue::NULL, &payload).map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
            error
                .as_string()
                .unwrap_or_else(|| "Progress callback threw a non-string error".to_string()),
        ))
    })?;
    Ok(())
}

pub(crate) fn emit_stats_timeline_progress(
    callback: &Function,
    progress: f64,
) -> SubtrActorResult<()> {
    emit_stage_progress(callback, "stats-timeline", progress)
}
