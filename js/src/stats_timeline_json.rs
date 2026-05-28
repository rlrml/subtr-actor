use js_sys::{Array, Function, Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;

use crate::progress::emit_stats_timeline_progress;

const DEFAULT_STATS_TIMELINE_FRAME_CHUNK_BYTES: usize = 32 * 1024 * 1024;

fn set_json_bytes<T: serde::Serialize>(
    object: &Object,
    key: &str,
    value: &T,
) -> Result<(), JsValue> {
    let bytes = serde_json::to_vec(value)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize {key}: {e}")))?;
    Reflect::set(
        object,
        &JsValue::from_str(key),
        &Uint8Array::from(bytes.as_slice()),
    )?;
    Ok(())
}

pub(crate) fn stats_timeline_json_parts(
    timeline: subtr_actor::ReplayStatsTimelineScaffold,
    max_frame_chunk_bytes: Option<usize>,
    progress: Option<(&Function, usize, f64, f64)>,
) -> Result<JsValue, JsValue> {
    let max_frame_chunk_bytes = max_frame_chunk_bytes
        .unwrap_or(DEFAULT_STATS_TIMELINE_FRAME_CHUNK_BYTES)
        .max(1024);
    let result = Object::new();
    set_json_bytes(&result, "config", &timeline.config)?;
    if let Some((callback, _, start, end)) = progress {
        emit_stats_timeline_progress(callback, start + ((end - start) * 0.05))
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }
    set_json_bytes(&result, "replayMeta", &timeline.replay_meta)?;
    if let Some((callback, _, start, end)) = progress {
        emit_stats_timeline_progress(callback, start + ((end - start) * 0.1))
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }
    set_json_bytes(&result, "events", &timeline.events)?;
    if let Some((callback, _, start, end)) = progress {
        emit_stats_timeline_progress(callback, start + ((end - start) * 0.15))
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }

    let frame_chunks = Array::new();
    let mut current_chunk = Vec::new();
    current_chunk.push(b'[');
    let mut current_chunk_frames = 0usize;
    let total_frames = timeline.frames.len();

    for (frame_index, frame) in timeline.frames.iter().enumerate() {
        let frame_bytes = serde_json::to_vec(frame)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize stats frame: {e}")))?;
        let separator_bytes = usize::from(current_chunk_frames > 0);
        if current_chunk_frames > 0
            && current_chunk.len() + separator_bytes + frame_bytes.len() + 1 > max_frame_chunk_bytes
        {
            current_chunk.push(b']');
            frame_chunks.push(&Uint8Array::from(current_chunk.as_slice()));
            current_chunk = Vec::new();
            current_chunk.push(b'[');
            current_chunk_frames = 0;
        }
        if current_chunk_frames > 0 {
            current_chunk.push(b',');
        }
        current_chunk.extend_from_slice(&frame_bytes);
        current_chunk_frames += 1;

        if let Some((callback, report_every_n_frames, start, end)) = progress {
            let processed_frames = frame_index + 1;
            if processed_frames == total_frames
                || processed_frames.is_multiple_of(report_every_n_frames.max(1))
            {
                let frame_progress = if total_frames == 0 {
                    1.0
                } else {
                    processed_frames as f64 / total_frames as f64
                };
                let weighted_progress = start + ((end - start) * (0.15 + (frame_progress * 0.85)));
                emit_stats_timeline_progress(callback, weighted_progress).map_err(|error| {
                    JsValue::from_str(&format!("Failed to emit progress: {error:?}"))
                })?;
            }
        }
    }

    current_chunk.push(b']');
    frame_chunks.push(&Uint8Array::from(current_chunk.as_slice()));
    Reflect::set(
        &result,
        &JsValue::from_str("frameChunks"),
        &frame_chunks.into(),
    )?;
    if let Some((callback, _, _, end)) = progress {
        emit_stats_timeline_progress(callback, end)
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }
    Ok(result.into())
}
