use super::*;

fn player(name: &str) -> PlayerId {
    PlayerId::Epic(name.to_owned())
}

fn record_frame(tracker: &mut PlayerSpanTracker<u8>, frame: usize, time: f32, state: u8) {
    tracker.record(
        frame,
        time - 1.0,
        time,
        1.0,
        &player("a"),
        None,
        true,
        state,
    );
}

#[test]
fn same_state_contributions_coalesce_into_one_span() {
    let mut tracker = PlayerSpanTracker::<u8>::default();
    record_frame(&mut tracker, 1, 1.0, 7);
    record_frame(&mut tracker, 2, 2.0, 7);
    record_frame(&mut tracker, 3, 3.0, 7);
    assert!(tracker.events().is_empty());
    let projected = tracker.projected_events();
    assert_eq!(projected.len(), 1);
    assert_eq!(projected[0].time, 0.0);
    assert_eq!(projected[0].end_time, 3.0);
    assert_eq!(projected[0].duration, 3.0);
    assert_eq!(projected[0].frame, 1);
    assert_eq!(projected[0].end_frame, 3);
}

#[test]
fn state_change_closes_the_previous_span() {
    let mut tracker = PlayerSpanTracker::<u8>::default();
    record_frame(&mut tracker, 1, 1.0, 7);
    record_frame(&mut tracker, 2, 2.0, 9);
    assert_eq!(tracker.events().len(), 1);
    assert_eq!(tracker.events()[0].state, 7);
    let projected = tracker.projected_events();
    assert_eq!(projected.len(), 2);
    assert_eq!(projected[1].state, 9);
}

#[test]
fn close_prevents_bridging_across_gaps() {
    let mut tracker = PlayerSpanTracker::<u8>::default();
    record_frame(&mut tracker, 1, 1.0, 7);
    tracker.close(&player("a"));
    record_frame(&mut tracker, 5, 5.0, 7);
    let projected = tracker.projected_events();
    assert_eq!(projected.len(), 2);
    assert_eq!(projected[0].duration, 1.0);
    assert_eq!(projected[1].time, 4.0);
}

#[test]
fn scalar_segments_pure_frame_is_single_segment() {
    let segments = scalar_state_segments(5.0, 5.0, &[-10.0, 10.0], &['a', 'b', 'c']);
    assert_eq!(segments, vec![('b', 1.0)]);
}

#[test]
fn scalar_segments_split_at_crossing() {
    let segments = scalar_state_segments(-20.0, 20.0, &[-10.0, 10.0], &['a', 'b', 'c']);
    assert_eq!(segments.len(), 3);
    assert_eq!(segments[0].0, 'a');
    assert_eq!(segments[1].0, 'b');
    assert_eq!(segments[2].0, 'c');
    assert!((segments[0].1 - 0.25).abs() < 1e-6);
    assert!((segments[1].1 - 0.5).abs() < 1e-6);
    assert!((segments[2].1 - 0.25).abs() < 1e-6);
    let total: f32 = segments.iter().map(|(_, fraction)| fraction).sum();
    assert!((total - 1.0).abs() < 1e-6);
}

#[test]
fn scalar_segments_handle_downward_motion() {
    let segments = scalar_state_segments(15.0, -15.0, &[-10.0, 10.0], &['a', 'b', 'c']);
    assert_eq!(
        segments.iter().map(|(state, _)| *state).collect::<Vec<_>>(),
        vec!['c', 'b', 'a']
    );
}
