use super::*;
use boxcars::RemoteId;

fn player(id: u32) -> PlayerId {
    RemoteId::SplitScreen(id)
}

#[allow(clippy::too_many_arguments)]
fn movement_event(
    player_id: PlayerId,
    is_team_0: bool,
    frame: usize,
    dt: f32,
    speed: f32,
    distance: f32,
    speed_band: &str,
    height_band: &str,
) -> MovementEvent {
    MovementEvent {
        time: frame as f32,
        frame,
        end_time: frame as f32,
        end_frame: frame,
        player: player_id,
        player_position: None,
        is_team_0,
        dt,
        speed,
        distance,
        speed_band: speed_band.to_owned(),
        height_band: height_band.to_owned(),
    }
}

/// Reference projection: a fresh accumulator folded over the full committed
/// history plus the pending overlay. This is what the old per-frame rebuild did,
/// and what the incremental projection must reproduce exactly.
fn full_rebuild(
    committed: &[MovementEvent],
    pending: &[MovementEvent],
) -> MovementStatsAccumulator {
    let mut accumulator = MovementStatsAccumulator::default();
    for event in committed {
        accumulator.apply_event(event);
    }
    for event in pending {
        accumulator.apply_event(event);
    }
    accumulator
}

#[test]
fn movement_projection_matches_full_rebuild_and_folds_each_event_once() {
    let p0 = player(0);
    let p1 = player(1);
    let bands = [
        ("slow", "ground"),
        ("boost", "low_air"),
        ("supersonic", "high_air"),
    ];

    let mut projection = IncrementalMovementProjection::default();
    let mut committed: Vec<MovementEvent> = Vec::new();
    // What a per-frame full rebuild would have folded in total (the quadratic
    // amount): the sum of the committed length at every frame.
    let mut per_frame_rebuild_folds = 0usize;

    for frame in 0..60usize {
        let (speed_band, height_band) = bands[frame % bands.len()];

        // Each active player carries one in-progress pending event this frame
        // whose accumulated dt/distance grow with the frame index.
        let pending = vec![
            movement_event(
                p0.clone(),
                true,
                frame,
                1.0 + frame as f32 * 0.01,
                500.0 + frame as f32,
                12.0,
                speed_band,
                height_band,
            ),
            movement_event(
                p1.clone(),
                false,
                frame,
                0.8,
                1400.0,
                9.5,
                "boost",
                "ground",
            ),
        ];

        // Periodically a pending event finalizes into the committed stream,
        // mirroring how the calculator commits on classification changes.
        if frame % 4 == 0 {
            committed.push(movement_event(
                p0.clone(),
                true,
                frame,
                1.0,
                600.0,
                30.0,
                speed_band,
                height_band,
            ));
        }
        if frame % 6 == 0 {
            committed.push(movement_event(
                p1.clone(),
                false,
                frame,
                1.0,
                1500.0,
                25.0,
                "supersonic",
                "high_air",
            ));
        }

        let actual = projection.project(&committed, &pending);
        let expected = full_rebuild(&committed, &pending);
        assert_eq!(
            actual, expected,
            "frame {frame}: incremental projection diverged from full rebuild"
        );

        per_frame_rebuild_folds += committed.len();
    }

    // Anti-quadratic guarantee: each committed event is folded into the base
    // exactly once across the whole run, not re-folded every frame.
    assert_eq!(
        projection.committed_folds,
        committed.len(),
        "each committed event must be folded into the base exactly once"
    );
    assert!(
        per_frame_rebuild_folds > committed.len(),
        "the scenario must span many frames over a growing committed stream"
    );
    assert!(
        projection.committed_folds < per_frame_rebuild_folds,
        "incremental fold work ({}) must stay below the per-frame-rebuild work ({per_frame_rebuild_folds})",
        projection.committed_folds,
    );
}

#[test]
fn movement_projection_does_not_refold_unchanged_committed_events() {
    let p0 = player(0);
    let committed = vec![
        movement_event(p0.clone(), true, 0, 1.0, 700.0, 40.0, "boost", "low_air"),
        movement_event(
            p0.clone(),
            true,
            1,
            1.0,
            2300.0,
            55.0,
            "supersonic",
            "high_air",
        ),
    ];

    let mut projection = IncrementalMovementProjection::default();

    // First projection folds both committed events into the base.
    let first = projection.project(&committed, &[]);
    assert_eq!(projection.committed_folds, committed.len());
    assert_eq!(first, full_rebuild(&committed, &[]));

    // Re-projecting with the same committed stream (only the pending overlay
    // changes) must not re-fold or double-count committed events into the base.
    let pending = vec![movement_event(
        p0.clone(),
        true,
        2,
        0.5,
        900.0,
        10.0,
        "boost",
        "ground",
    )];
    let second = projection.project(&committed, &pending);
    assert_eq!(
        projection.committed_folds,
        committed.len(),
        "committed events must not be re-folded when the committed stream is unchanged"
    );
    assert_eq!(second, full_rebuild(&committed, &pending));
}
