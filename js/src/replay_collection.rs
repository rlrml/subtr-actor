use js_sys::Function;
use subtr_actor::{
    collector::replay_data::{ReplayData, ReplayDataCollector},
    collector::CallbackCollector,
    Collector, ReplayProcessor, ResolvedBoostPadCollector, StatsTimelineEventCollector,
    SubtrActorError,
};
use wasm_bindgen::prelude::*;

use crate::progress::{emit_progress, emit_stage_progress, emit_stats_timeline_progress};

fn get_total_frames(replay: &boxcars::Replay) -> Result<usize, JsValue> {
    replay
        .network_frames
        .as_ref()
        .map(|network_frames| network_frames.frames.len())
        .ok_or_else(|| JsValue::from_str("Replay has no network frames"))
}

pub(crate) fn collect_replay_data_with_optional_progress(
    replay: &boxcars::Replay,
    progress: Option<(&Function, usize)>,
) -> Result<ReplayData, JsValue> {
    let total_frames = get_total_frames(replay)?;
    let mut processor = ReplayProcessor::new(replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to initialize replay processor: {e:?}")))?;
    let mut replay_data_collector = ReplayDataCollector::new();
    let mut boost_pad_collector = ResolvedBoostPadCollector::new();
    let mut last_reported_frames = 0usize;
    let mut progress_collector = progress
        .map(|(callback, frame_interval)| {
            emit_progress(callback, "processing", 0, total_frames)?;
            Ok::<_, SubtrActorError>(CallbackCollector::with_frame_interval(
                |_frame, frame_number, _current_time| {
                    last_reported_frames = frame_number + 1;
                    emit_progress(callback, "processing", last_reported_frames, total_frames)
                },
                frame_interval.max(1),
            ))
        })
        .transpose()
        .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;

    let mut collectors: Vec<&mut dyn Collector> =
        vec![&mut replay_data_collector, &mut boost_pad_collector];
    if let Some(progress_collector) = progress_collector.as_mut() {
        collectors.push(progress_collector);
    }

    processor
        .process_all(&mut collectors)
        .map_err(|e| JsValue::from_str(&format!("Failed to process replay: {e:?}")))?;

    if let Some((callback, _)) = progress {
        if last_reported_frames < total_frames {
            emit_progress(callback, "processing", total_frames, total_frames).map_err(|error| {
                JsValue::from_str(&format!("Failed to emit progress: {error:?}"))
            })?;
        }
    }

    replay_data_collector
        .into_replay_data_with_boost_pads(processor, boost_pad_collector.into_resolved_boost_pads())
        .map_err(|e| JsValue::from_str(&format!("Failed to assemble replay data: {e:?}")))
}

pub(crate) fn collect_replay_bundle_with_optional_progress(
    replay: &boxcars::Replay,
    progress: Option<(&Function, usize)>,
) -> Result<(ReplayData, subtr_actor::ReplayStatsTimelineScaffold), JsValue> {
    let total_frames = get_total_frames(replay)?;
    let mut processor = ReplayProcessor::new(replay)
        .map_err(|e| JsValue::from_str(&format!("Failed to initialize replay processor: {e:?}")))?;
    let mut replay_data_collector = ReplayDataCollector::new();
    let mut stats_collector = StatsTimelineEventCollector::new();
    let mut boost_pad_collector = ResolvedBoostPadCollector::new();
    let mut last_reported_frames = 0usize;
    let mut progress_collector = progress
        .map(|(callback, frame_interval)| {
            emit_progress(callback, "processing", 0, total_frames)?;
            Ok::<_, SubtrActorError>(CallbackCollector::with_frame_interval(
                |_frame, frame_number, _current_time| {
                    last_reported_frames = frame_number + 1;
                    emit_progress(callback, "processing", last_reported_frames, total_frames)
                },
                frame_interval.max(1),
            ))
        })
        .transpose()
        .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;

    let mut collectors: Vec<&mut dyn Collector> = vec![
        &mut replay_data_collector,
        &mut stats_collector,
        &mut boost_pad_collector,
    ];
    if let Some(progress_collector) = progress_collector.as_mut() {
        collectors.push(progress_collector);
    }

    processor
        .process_all(&mut collectors)
        .map_err(|e| JsValue::from_str(&format!("Failed to process replay: {e:?}")))?;

    if let Some((callback, _)) = progress {
        if last_reported_frames < total_frames {
            emit_progress(callback, "processing", total_frames, total_frames).map_err(|error| {
                JsValue::from_str(&format!("Failed to emit progress: {error:?}"))
            })?;
        }
        emit_stage_progress(callback, "building-stats", 0.0)
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }

    let stats_timeline = stats_collector
        .into_replay_stats_timeline_scaffold()
        .map_err(|e| JsValue::from_str(&format!("Failed to assemble stats timeline: {e:?}")))?;
    if let Some((callback, _)) = progress {
        emit_stage_progress(callback, "building-stats", 1.0)
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }

    let replay_data = replay_data_collector
        .into_replay_data_with_boost_pads(processor, boost_pad_collector.into_resolved_boost_pads())
        .map_err(|e| JsValue::from_str(&format!("Failed to assemble replay data: {e:?}")))?;
    if let Some((callback, _)) = progress {
        emit_stats_timeline_progress(callback, 0.35)
            .map_err(|error| JsValue::from_str(&format!("Failed to emit progress: {error:?}")))?;
    }
    Ok((replay_data, stats_timeline))
}
