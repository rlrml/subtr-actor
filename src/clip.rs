//! Replay clipping: trim a [`boxcars::Replay`] down to a small, self-contained
//! window of frames that still processes through the full [`ReplayProcessor`]
//! pipeline unchanged.
//!
//! # Why this works
//!
//! [`ReplayProcessor`] only consumes already-*decoded* [`boxcars::Frame`]s plus
//! a few static tables (`net_version`, `objects`, `names`). It never touches the
//! replay bitstream. A [`ReplayClip`] therefore needs only those tables and a
//! list of frames.
//!
//! The catch is that the processor is fully incremental: the meaning of frame
//! `N` depends on every actor spawned and every attribute set in frames `0..N`.
//! Naively slicing out `frames[N..M]` would reference actors that were never
//! created. To fix this, a clip begins with a **synthetic keyframe**: a single
//! frame that re-spawns every actor that is alive at the start of the window and
//! re-emits each of its current attributes (reconstructed from
//! [`ActorStateModeler`]). When the clip is processed, that keyframe seeds the
//! processor's world to exactly the state it had at the window boundary, after
//! which the real frames replay normally.
//!
//! # Boundary artifacts and lead-in
//!
//! The synthetic keyframe reproduces *persistent* actor state perfectly, but the
//! processor's per-frame updaters (touch detection, dodge detection, etc.) are
//! delta-based and have no history before the keyframe. To keep the region you
//! actually want to assert on clean, request a few frames of **lead-in** before
//! it (see [`clip_replay_around`]). The differential tests quantify how much
//! lead-in is needed for a faithful reproduction.
//!
//! [`ReplayProcessor`]: crate::processor::ReplayProcessor
//! [`ActorStateModeler`]: crate::actor_state::ActorStateModeler

use crate::actor_state::ActorStateModeler;
use crate::{SubtrActorError, SubtrActorErrorVariant, SubtrActorResult};
use serde::{Deserialize, Serialize};

/// Current [`ReplayClip`] schema version. Bump on breaking layout changes.
pub const CLIP_VERSION: u32 = 1;

/// Where a [`ReplayClip`] came from in its source replay, for provenance and for
/// mapping source frame indices back onto clip frame indices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClipProvenance {
    /// Index, in the source replay, of the first *real* frame included in the
    /// clip (i.e. the frame immediately after the synthetic keyframe).
    pub source_first_real_frame: usize,
    /// Index, in the source replay, of the last real frame included (inclusive).
    pub source_last_real_frame: usize,
    /// Number of leading real frames included purely as warm-up before the
    /// region of interest. `0` when the clip was taken by raw frame range.
    pub lead_in_frames: usize,
    /// Number of synthetic frames prepended (currently always `1` keyframe, or
    /// `0` when the window starts at frame `0` and needs no seeding).
    pub synthetic_frame_count: usize,
}

impl ClipProvenance {
    /// Map an index in the *source* replay to the corresponding index in the
    /// clip's `frames`, accounting for the prepended synthetic keyframe. Returns
    /// `None` if the source frame is outside the clipped window.
    pub fn clip_index_of(&self, source_frame: usize) -> Option<usize> {
        if source_frame < self.source_first_real_frame || source_frame > self.source_last_real_frame
        {
            return None;
        }
        Some(self.synthetic_frame_count + (source_frame - self.source_first_real_frame))
    }
}

/// A self-contained, serializable slice of a replay that can be processed by the
/// full subtr-actor pipeline.
///
/// Reconstruct a [`boxcars::Replay`] with [`ReplayClip::to_replay`], then feed it
/// to [`ReplayProcessor`](crate::processor::ReplayProcessor) like any other
/// replay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayClip {
    /// Schema version; see [`CLIP_VERSION`].
    pub clip_version: u32,
    /// `net_version` from the source replay (drives rigid-body normalization).
    pub net_version: Option<i32>,
    pub major_version: i32,
    pub minor_version: i32,
    /// `game_type` string from the source replay (metadata only).
    pub game_type: String,
    /// Object name table; an index into this is a `boxcars::ObjectId`.
    pub objects: Vec<String>,
    /// Name table, referenced by `name_id` on actors.
    pub names: Vec<String>,
    /// The synthetic keyframe (if any) followed by the real source frames.
    pub frames: Vec<boxcars::Frame>,
    /// Provenance / index mapping back to the source replay.
    pub provenance: ClipProvenance,
}

impl ReplayClip {
    /// Reconstruct a [`boxcars::Replay`] suitable for
    /// [`ReplayProcessor`](crate::processor::ReplayProcessor). Header properties,
    /// keyframes, net-cache and other bitstream-only tables are intentionally
    /// left empty: the processor does not need them to walk decoded frames.
    pub fn to_replay(&self) -> boxcars::Replay {
        boxcars::Replay {
            header_size: 0,
            header_crc: 0,
            major_version: self.major_version,
            minor_version: self.minor_version,
            net_version: self.net_version,
            game_type: self.game_type.clone(),
            properties: Vec::new(),
            content_size: 0,
            content_crc: 0,
            network_frames: Some(boxcars::NetworkFrames {
                frames: self.frames.clone(),
            }),
            levels: Vec::new(),
            keyframes: Vec::new(),
            debug_info: Vec::new(),
            tick_marks: Vec::new(),
            packages: Vec::new(),
            objects: self.objects.clone(),
            names: self.names.clone(),
            class_indices: Vec::new(),
            net_cache: Vec::new(),
        }
    }

    /// Serialize to pretty JSON, the canonical fixture form.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON produced by [`ReplayClip::to_json`].
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

/// Whether an attribute records a transient *event* rather than persistent
/// world state. The processor's event detectors fire on `UpdatedAttribute`s
/// carrying these (boost pad pickups, demolishes, goal explosions), so
/// re-emitting a stale one from the synthetic keyframe would manufacture a
/// phantom event that never happened inside the clip window. They carry no
/// state any detector reads back, so the keyframe simply omits them.
fn attribute_is_transient_event(attribute: &boxcars::Attribute) -> bool {
    matches!(
        attribute,
        boxcars::Attribute::Pickup(_)
            | boxcars::Attribute::PickupNew(_)
            | boxcars::Attribute::Demolish(_)
            | boxcars::Attribute::DemolishFx(_)
            | boxcars::Attribute::DemolishExtended(_)
            | boxcars::Attribute::Explosion(_)
            | boxcars::Attribute::ExtendedExplosion(_)
            | boxcars::Attribute::StatEvent(_)
    )
}

/// Build the synthetic keyframe that recreates the world modeled by `modeler`.
///
/// Emits one `NewActor` per live actor plus one `UpdatedAttribute` per current
/// attribute (transient event-like attributes excepted; see
/// [`attribute_is_transient_event`]). Output is sorted (actors by id,
/// attributes by object id) so the resulting clip is deterministic and its JSON
/// fixture is stable/diffable.
fn synthesize_keyframe(modeler: &ActorStateModeler, time: f32) -> boxcars::Frame {
    let mut new_actors: Vec<boxcars::NewActor> = Vec::with_capacity(modeler.actor_states.len());
    let mut updated_actors: Vec<boxcars::UpdatedAttribute> = Vec::new();

    let mut actor_ids: Vec<&boxcars::ActorId> = modeler.actor_states.keys().collect();
    actor_ids.sort_by_key(|id| id.0);

    for actor_id in actor_ids {
        let state = &modeler.actor_states[actor_id];
        new_actors.push(boxcars::NewActor {
            actor_id: *actor_id,
            name_id: state.name_id,
            object_id: state.object_id,
            // `initial_trajectory` is never read by the processor; position is
            // carried by the re-emitted RigidBody/Location attributes below.
            initial_trajectory: boxcars::Trajectory {
                location: None,
                rotation: None,
            },
        });

        let mut object_ids: Vec<&boxcars::ObjectId> = state.attributes.keys().collect();
        object_ids.sort_by_key(|id| id.0);
        for object_id in object_ids {
            let (attribute, _source_frame) = &state.attributes[object_id];
            if attribute_is_transient_event(attribute) {
                continue;
            }
            updated_actors.push(boxcars::UpdatedAttribute {
                actor_id: *actor_id,
                // `stream_id` is unused post-decode; mirror the object id.
                stream_id: boxcars::StreamId(object_id.0),
                object_id: *object_id,
                attribute: attribute.clone(),
            });
        }
    }

    boxcars::Frame {
        time,
        delta: 0.0,
        new_actors,
        deleted_actors: Vec::new(),
        updated_actors,
    }
}

/// Extract a clip spanning the inclusive source frame range `[real_start, real_end]`.
///
/// A synthetic keyframe reproducing the world state as of the end of frame
/// `real_start - 1` is prepended (unless `real_start == 0`). `lead_in_frames` is
/// recorded in provenance for callers that built the range with warm-up padding;
/// it does not affect which frames are included.
pub fn clip_replay_range(
    replay: &boxcars::Replay,
    real_start: usize,
    real_end: usize,
    lead_in_frames: usize,
) -> SubtrActorResult<ReplayClip> {
    let source_frames = &replay
        .network_frames
        .as_ref()
        .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::NoNetworkFrames))?
        .frames;

    if real_start > real_end || real_end >= source_frames.len() {
        return SubtrActorError::new_result(SubtrActorErrorVariant::FrameIndexOutOfBounds);
    }

    // Seed an actor-state model up to (but not including) the window start.
    let mut modeler = ActorStateModeler::new();
    for (index, frame) in source_frames.iter().enumerate().take(real_start) {
        modeler.process_frame(frame, index)?;
    }

    let mut frames = Vec::with_capacity(real_end - real_start + 2);
    let synthetic_frame_count = if real_start > 0 {
        // Time the keyframe just before the first real frame for continuity.
        let keyframe_time = source_frames[real_start - 1].time;
        frames.push(synthesize_keyframe(&modeler, keyframe_time));
        1
    } else {
        0
    };
    frames.extend(source_frames[real_start..=real_end].iter().cloned());

    Ok(ReplayClip {
        clip_version: CLIP_VERSION,
        net_version: replay.net_version,
        major_version: replay.major_version,
        minor_version: replay.minor_version,
        game_type: replay.game_type.clone(),
        objects: replay.objects.clone(),
        names: replay.names.clone(),
        frames,
        provenance: ClipProvenance {
            source_first_real_frame: real_start,
            source_last_real_frame: real_end,
            lead_in_frames,
            synthetic_frame_count,
        },
    })
}

/// Extract a clip centered on a region of interest `[region_start, region_end]`,
/// padded with `lead_in` warm-up frames before it and `tail` frames after.
///
/// This is the ergonomic entry point for tests: pick the frames around an event
/// and get back a clip whose region of interest is preceded by real frames, so
/// delta-based detectors are warmed up before the assertion window.
pub fn clip_replay_around(
    replay: &boxcars::Replay,
    region_start: usize,
    region_end: usize,
    lead_in: usize,
    tail: usize,
) -> SubtrActorResult<ReplayClip> {
    let frame_count = replay
        .network_frames
        .as_ref()
        .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::NoNetworkFrames))?
        .frames
        .len();

    let real_start = region_start.saturating_sub(lead_in);
    let real_end = region_end
        .saturating_add(tail)
        .min(frame_count.saturating_sub(1));
    let actual_lead_in = region_start - real_start;
    clip_replay_range(replay, real_start, real_end, actual_lead_in)
}

/// Index of the first frame whose `time` is at or after `time` (the last frame
/// if every frame is earlier).
pub fn frame_index_at_time(replay: &boxcars::Replay, time: f32) -> SubtrActorResult<usize> {
    let frames = &replay
        .network_frames
        .as_ref()
        .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::NoNetworkFrames))?
        .frames;
    Ok(frames
        .iter()
        .position(|frame| frame.time >= time)
        .unwrap_or(frames.len().saturating_sub(1)))
}

/// [`clip_replay_around`], but with the region of interest given in replay
/// seconds instead of frame indices. Most event assertions are written against
/// event times (which clips preserve from the source replay), so this is the
/// usual entry point when migrating a full-replay test onto a clip.
pub fn clip_replay_around_times(
    replay: &boxcars::Replay,
    region_start_time: f32,
    region_end_time: f32,
    lead_in: usize,
    tail: usize,
) -> SubtrActorResult<ReplayClip> {
    let region_start = frame_index_at_time(replay, region_start_time)?;
    let region_end = frame_index_at_time(replay, region_end_time)?;
    clip_replay_around(replay, region_start, region_end, lead_in, tail)
}

#[cfg(test)]
#[path = "clip_tests.rs"]
mod tests;
