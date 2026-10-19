use super::*;

fn replay_with_frame_count(frame_count: usize) -> boxcars::Replay {
    ReplayClip {
        clip_version: CLIP_VERSION,
        net_version: Some(10),
        major_version: 868,
        minor_version: 32,
        game_type: String::new(),
        objects: Vec::new(),
        names: Vec::new(),
        frames: (0..frame_count)
            .map(|index| boxcars::Frame {
                time: index as f32,
                delta: 1.0,
                new_actors: Vec::new(),
                updated_actors: Vec::new(),
                deleted_actors: Vec::new(),
            })
            .collect(),
        provenance: ClipProvenance {
            source_first_real_frame: 0,
            source_last_real_frame: frame_count.saturating_sub(1),
            lead_in_frames: 0,
            synthetic_frame_count: 0,
        },
    }
    .to_replay()
}

#[test]
fn padding_does_not_hide_invalid_clip_regions() {
    let replay = replay_with_frame_count(5);
    for (start, end) in [(3, 1), (5, 5), (usize::MAX, usize::MAX)] {
        assert!(matches!(
            clip_replay_around(&replay, start, end, usize::MAX, usize::MAX)
                .unwrap_err()
                .variant,
            SubtrActorErrorVariant::FrameIndexOutOfBounds
        ));
    }
}

#[test]
fn valid_clip_padding_is_clamped_to_replay_bounds() {
    let replay = replay_with_frame_count(5);
    let clip = clip_replay_around(&replay, 1, 3, usize::MAX, usize::MAX).unwrap();
    assert_eq!(clip.frames.len(), 5);
    assert_eq!(clip.provenance.source_first_real_frame, 0);
    assert_eq!(clip.provenance.source_last_real_frame, 4);
    assert_eq!(clip.provenance.lead_in_frames, 1);
    assert_eq!(clip.provenance.clip_index_of(1), Some(1));
}

#[test]
fn empty_replay_has_no_frame_at_time() {
    let replay = replay_with_frame_count(0);
    assert!(matches!(
        frame_index_at_time(&replay, 0.0).unwrap_err().variant,
        SubtrActorErrorVariant::FrameIndexOutOfBounds
    ));
    assert!(matches!(
        clip_replay_around(&replay, 0, 0, 0, 0).unwrap_err().variant,
        SubtrActorErrorVariant::FrameIndexOutOfBounds
    ));
}

#[test]
fn frame_time_lookup_preserves_boundary_and_clamping_behavior() {
    let replay = replay_with_frame_count(5);
    for (time, expected) in [(-1.0, 0), (0.0, 0), (0.1, 1), (1.0, 1), (10.0, 4)] {
        assert_eq!(frame_index_at_time(&replay, time).unwrap(), expected);
    }
}

#[test]
fn clip_time_lookup_rejects_nonfinite_times() {
    let replay = replay_with_frame_count(5);
    for time in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(matches!(
            frame_index_at_time(&replay, time).unwrap_err().variant,
            SubtrActorErrorVariant::InvalidReplayTime { .. }
        ));
        for (start, end) in [(time, 1.0), (0.0, time)] {
            assert!(matches!(
                clip_replay_around_times(&replay, start, end, 10, 10)
                    .unwrap_err()
                    .variant,
                SubtrActorErrorVariant::InvalidReplayTime { .. }
            ));
        }
    }
}

#[test]
fn clip_rejects_reversed_times_that_resolve_to_the_same_frame() {
    let replay = replay_with_frame_count(5);
    for (start, end) in [(0.2, 0.1), (10.0, 9.0)] {
        assert!(matches!(
            clip_replay_around_times(&replay, start, end, 10, 10)
                .unwrap_err()
                .variant,
            SubtrActorErrorVariant::InvalidClipTimeRange { .. }
        ));
    }
}

#[test]
fn provenance_maps_source_frames_onto_clip_frames() {
    let provenance = ClipProvenance {
        source_first_real_frame: 100,
        source_last_real_frame: 150,
        lead_in_frames: 10,
        synthetic_frame_count: 1,
    };

    // The first real frame sits right after the synthetic keyframe.
    assert_eq!(provenance.clip_index_of(100), Some(1));
    assert_eq!(provenance.clip_index_of(110), Some(11));
    assert_eq!(provenance.clip_index_of(150), Some(51));
    // Outside the window.
    assert_eq!(provenance.clip_index_of(99), None);
    assert_eq!(provenance.clip_index_of(151), None);
}

#[test]
fn provenance_without_keyframe_is_identity() {
    let provenance = ClipProvenance {
        source_first_real_frame: 0,
        source_last_real_frame: 10,
        lead_in_frames: 0,
        synthetic_frame_count: 0,
    };
    assert_eq!(provenance.clip_index_of(0), Some(0));
    assert_eq!(provenance.clip_index_of(10), Some(10));
}

#[test]
fn clip_region_end_is_clamped_to_the_last_frame() {
    let replay = replay_with_frame_count(5);
    for end in [5, usize::MAX] {
        let clip = clip_replay_around(&replay, 2, end, 0, 0).unwrap();
        assert_eq!(clip.provenance.source_first_real_frame, 2);
        assert_eq!(clip.provenance.source_last_real_frame, 4);
    }
}
