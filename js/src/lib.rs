use js_sys::Function;
use subtr_actor::{
    collector::replay_data::{ReplayDataCollector, ReplayDataSupplementalData},
    collector::CallbackCollector,
    BoostReducer, FlipResetTracker, FrameRateDecorator, NDArrayCollector, ReducerCollector,
    ReplayProcessor, StatsTimelineCollector, SubtrActorError, SubtrActorErrorVariant,
    SubtrActorResult,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log!("subtr-actor WASM bindings loaded");
}

// Default feature adders (same as Python bindings)
const DEFAULT_GLOBAL_FEATURE_ADDERS: &[&str] = &["BallRigidBody"];
const DEFAULT_PLAYER_FEATURE_ADDERS: &[&str] = &["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"];

fn parse_replay_from_data(data: &[u8]) -> Result<boxcars::Replay, JsValue> {
    boxcars::ParserBuilder::new(data)
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()
        .map_err(|e| JsValue::from_str(&format!("Failed to parse replay: {e}")))
}

fn get_total_frames(replay: &boxcars::Replay) -> Result<usize, JsValue> {
    replay
        .network_frames
        .as_ref()
        .map(|network_frames| network_frames.frames.len())
        .ok_or_else(|| JsValue::from_str("Replay has no network frames"))
}

fn emit_progress(
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

/// Parse a replay file and return the raw replay data as JavaScript object
#[wasm_bindgen]
pub fn parse_replay(data: &[u8]) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;
    let replay_value = serde_json::to_value(&replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize replay: {e}")))?;

    serde_wasm_bindgen::to_value(&replay_value)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

/// Get NDArray data with metadata from replay data
#[wasm_bindgen]
pub fn get_ndarray_with_info(
    data: &[u8],
    global_feature_adders: Option<Vec<String>>,
    player_feature_adders: Option<Vec<String>>,
    fps: Option<f32>,
) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;
    let mut collector = build_ndarray_collector(global_feature_adders, player_feature_adders)?;

    // Use FrameRateDecorator with specified FPS (default 10.0)
    let mut decorated_collector =
        FrameRateDecorator::new_from_fps(fps.unwrap_or(10.0), &mut collector);

    let mut processor = ReplayProcessor::new(&replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to create processor: {e:?}")))?;

    processor
        .process(&mut decorated_collector)
        .map_err(|e| JsValue::from_str(&format!("Failed to process replay: {e:?}")))?;

    let (replay_meta_with_headers, ndarray) = collector
        .get_meta_and_ndarray()
        .map_err(|e| JsValue::from_str(&format!("Failed to get data: {e:?}")))?;

    // Convert ndarray to nested Vec for JavaScript
    let shape = ndarray.shape();
    let array_data: Vec<Vec<f32>> = ndarray.outer_iter().map(|row| row.to_vec()).collect();

    let result = serde_json::json!({
        "metadata": replay_meta_with_headers,
        "array_data": array_data,
        "shape": shape
    });

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

/// Get only the replay metadata (without processing frames)
#[wasm_bindgen]
pub fn get_replay_meta(
    data: &[u8],
    global_feature_adders: Option<Vec<String>>,
    player_feature_adders: Option<Vec<String>>,
) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;

    let mut collector = build_ndarray_collector(global_feature_adders, player_feature_adders)?;

    let replay_meta = collector
        .process_and_get_meta_and_headers(&replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to get metadata: {e:?}")))?;

    serde_wasm_bindgen::to_value(&replay_meta)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

/// Get column headers for the NDArray (useful for understanding the data structure)
#[wasm_bindgen]
pub fn get_column_headers(
    global_feature_adders: Option<Vec<String>>,
    player_feature_adders: Option<Vec<String>>,
) -> Result<JsValue, JsValue> {
    let collector = build_ndarray_collector(global_feature_adders, player_feature_adders)?;
    let headers = collector.get_column_headers();

    serde_wasm_bindgen::to_value(&headers)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

/// Get structured frame data using ReplayDataCollector
/// This matches Python behavior - no FPS resampling, so goal frame numbers align
#[wasm_bindgen]
pub fn get_replay_frames_data(data: &[u8]) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;

    // Keep replay-data assembly explicit so additional collectors can be
    // composed in the same processor pass without coupling them to
    // ReplayDataCollector itself.
    let mut processor = ReplayProcessor::new(&replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to initialize replay processor: {e:?}")))?;
    let mut replay_data_collector = ReplayDataCollector::new();
    let mut flip_reset_tracker = FlipResetTracker::new();
    let mut boost_pad_collector = ReducerCollector::new(BoostReducer::new());

    processor
        .process_all(&mut [
            &mut replay_data_collector,
            &mut flip_reset_tracker,
            &mut boost_pad_collector,
        ])
        .map_err(|e| JsValue::from_str(&format!("Failed to process replay: {e:?}")))?;

    let supplemental_data = ReplayDataSupplementalData::from_flip_reset_tracker(flip_reset_tracker)
        .with_boost_pads(boost_pad_collector.into_inner().resolved_boost_pads());
    let replay_data = replay_data_collector
        .into_replay_data_with_supplemental_data(processor, supplemental_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to assemble replay data: {e:?}")))?;

    serde_wasm_bindgen::to_value(&replay_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

#[wasm_bindgen]
pub fn get_replay_frames_data_with_progress(
    data: &[u8],
    callback: Function,
    report_every_n_frames: Option<usize>,
) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;
    let total_frames = get_total_frames(&replay)?;
    let frame_interval = report_every_n_frames.unwrap_or(1000).max(1);

    emit_progress(&callback, "processing", 0, total_frames)
        .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;

    let mut processor = ReplayProcessor::new(&replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to initialize replay processor: {e:?}")))?;
    let mut replay_data_collector = ReplayDataCollector::new();
    let mut flip_reset_tracker = FlipResetTracker::new();
    let mut boost_pad_collector = ReducerCollector::new(BoostReducer::new());
    let mut last_reported_frames = 0usize;
    let mut progress_collector = CallbackCollector::with_frame_interval(
        |_frame, frame_number, _current_time| {
            last_reported_frames = frame_number + 1;
            emit_progress(&callback, "processing", last_reported_frames, total_frames)
        },
        frame_interval,
    );

    processor
        .process_all(&mut [
            &mut replay_data_collector,
            &mut flip_reset_tracker,
            &mut boost_pad_collector,
            &mut progress_collector,
        ])
        .map_err(|e| JsValue::from_str(&format!("Failed to process replay: {e:?}")))?;

    if last_reported_frames < total_frames {
        emit_progress(&callback, "processing", total_frames, total_frames)
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }

    let supplemental_data = ReplayDataSupplementalData::from_flip_reset_tracker(flip_reset_tracker)
        .with_boost_pads(boost_pad_collector.into_inner().resolved_boost_pads());
    let replay_data = replay_data_collector
        .into_replay_data_with_supplemental_data(processor, supplemental_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to assemble replay data: {e:?}")))?;

    serde_wasm_bindgen::to_value(&replay_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

/// Get cumulative stats snapshots for each replay sample.
#[wasm_bindgen]
pub fn get_stats_timeline(data: &[u8]) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;

    let stats_timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to process replay stats: {e:?}")))?;

    serde_wasm_bindgen::to_value(&stats_timeline)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

/// Get dynamically-described cumulative stats snapshots for each replay sample.
#[wasm_bindgen]
pub fn get_dynamic_stats_timeline(data: &[u8]) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;

    let stats_timeline = StatsTimelineCollector::new()
        .get_dynamic_replay_data(&replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to process replay stats: {e:?}")))?;

    serde_wasm_bindgen::to_value(&stats_timeline)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

/// Validate that a replay file can be parsed
#[wasm_bindgen]
pub fn validate_replay(data: &[u8]) -> Result<JsValue, JsValue> {
    match parse_replay_from_data(data) {
        Ok(_) => serde_wasm_bindgen::to_value(&serde_json::json!({
            "valid": true,
            "message": "Replay is valid"
        })),
        Err(e) => serde_wasm_bindgen::to_value(&serde_json::json!({
            "valid": false,
            "error": e.as_string().unwrap_or_else(|| "Unknown error".to_string())
        })),
    }
    .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

/// Get basic replay information (version, etc.)
#[wasm_bindgen]
pub fn get_replay_info(data: &[u8]) -> Result<JsValue, JsValue> {
    let replay = parse_replay_from_data(data)?;

    let info = serde_json::json!({
        "header_size": replay.header_size,
        "major_version": replay.major_version,
        "minor_version": replay.minor_version,
        "net_version": replay.net_version,
        "properties_count": replay.properties.len()
    });

    serde_wasm_bindgen::to_value(&info)
        .map_err(|e| JsValue::from_str(&format!("Failed to convert to JS: {e}")))
}

fn build_ndarray_collector(
    global_feature_adders: Option<Vec<String>>,
    player_feature_adders: Option<Vec<String>>,
) -> Result<NDArrayCollector<f32>, JsValue> {
    let global_feature_adders = global_feature_adders.unwrap_or_else(|| {
        DEFAULT_GLOBAL_FEATURE_ADDERS
            .iter()
            .map(|s| s.to_string())
            .collect()
    });
    let player_feature_adders = player_feature_adders.unwrap_or_else(|| {
        DEFAULT_PLAYER_FEATURE_ADDERS
            .iter()
            .map(|s| s.to_string())
            .collect()
    });

    let global_strs: Vec<&str> = global_feature_adders.iter().map(|s| s.as_str()).collect();
    let player_strs: Vec<&str> = player_feature_adders.iter().map(|s| s.as_str()).collect();

    NDArrayCollector::<f32>::from_strings(&global_strs, &player_strs)
        .map_err(|e| JsValue::from_str(&format!("Failed to build collector: {e:?}")))
}
