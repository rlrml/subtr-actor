use super::*;

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
