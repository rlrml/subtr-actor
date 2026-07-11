use super::EventAssembler;
use crate::{EventLifecycle, EventPayload, EventTiming, TimelineEvent, TimelineEventKind};

const FPS: f32 = 30.0;

fn assembler_event(assembler: &mut EventAssembler, frame: usize) -> String {
    let time = frame as f32 / FPS;
    assembler.push(
        "timeline",
        frame,
        EventLifecycle::Finalized,
        EventTiming::Moment { frame, time },
        EventPayload::Timeline(TimelineEvent {
            time,
            frame: Some(frame),
            kind: TimelineEventKind::Shot,
            player_id: None,
            player_position: None,
            is_team_0: None,
        }),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assembler
        .events
        .last()
        .expect("push appends an event")
        .meta
        .id
        .clone()
}

/// A later projection that visits an *earlier-anchored* event the previous
/// projection had not seen (a retroactively committed entry) must not shift
/// the ids of the events that were already visible: the disambiguator only
/// counts within one `(stream, anchor)` group.
#[test]
fn retroactive_insertion_does_not_shift_existing_ids() {
    let mut first = EventAssembler::new();
    let first_ids = [
        assembler_event(&mut first, 10),
        assembler_event(&mut first, 10),
        assembler_event(&mut first, 20),
    ];

    // The retro event (frame 5) is committed before the others in list order,
    // as a calculator whose list is time-sorted would present it.
    let mut second = EventAssembler::new();
    let retro_id = assembler_event(&mut second, 5);
    let second_ids = [
        assembler_event(&mut second, 10),
        assembler_event(&mut second, 10),
        assembler_event(&mut second, 20),
    ];

    assert_eq!(first_ids, second_ids);
    assert_eq!(
        first_ids,
        ["timeline:10:0", "timeline:10:1", "timeline:20:0"]
    );
    assert_eq!(retro_id, "timeline:5:0");
}
