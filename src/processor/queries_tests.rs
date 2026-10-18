use super::*;

const ACTOR: boxcars::ActorId = boxcars::ActorId(7);
const PROPERTY: boxcars::ObjectId = boxcars::ObjectId(1);

fn frame(value: Option<i32>, spawned: bool, deleted: bool) -> boxcars::Frame {
    boxcars::Frame {
        time: 0.0,
        delta: 0.0,
        new_actors: if spawned {
            vec![boxcars::NewActor {
                actor_id: ACTOR,
                object_id: boxcars::ObjectId(0),
                name_id: None,
                initial_trajectory: boxcars::Trajectory {
                    location: None,
                    rotation: None,
                },
            }]
        } else {
            Vec::new()
        },
        deleted_actors: if deleted { vec![ACTOR] } else { Vec::new() },
        updated_actors: value
            .map(|value| boxcars::UpdatedAttribute {
                actor_id: ACTOR,
                stream_id: boxcars::StreamId(0),
                object_id: PROPERTY,
                attribute: boxcars::Attribute::Int(value),
            })
            .into_iter()
            .collect(),
    }
}

fn find_update(
    frames: Vec<boxcars::Frame>,
    index: usize,
    direction: SearchDirection,
) -> SubtrActorResult<(boxcars::Attribute, usize)> {
    let mut replay = crate::ReplayClip {
        clip_version: crate::CLIP_VERSION,
        net_version: Some(10),
        major_version: 868,
        minor_version: 32,
        game_type: String::new(),
        objects: Vec::new(),
        names: Vec::new(),
        frames: Vec::new(),
        provenance: crate::ClipProvenance {
            source_first_real_frame: 0,
            source_last_real_frame: 0,
            lead_in_frames: 0,
            synthetic_frame_count: 0,
        },
    }
    .to_replay();
    let bootstrap_replay = replay.clone();
    let mut processor = ReplayProcessor::new(&bootstrap_replay)?;
    replay.network_frames.as_mut().unwrap().frames = frames;
    processor.replay = &replay;
    processor.find_update_in_direction(index, &ACTOR, &PROPERTY, direction)
}

#[test]
fn update_search_stays_within_actor_lifetime() {
    let frames = vec![
        frame(Some(10), true, false),
        frame(Some(20), false, false),
        frame(None, false, true),
        frame(Some(30), true, false),
        frame(Some(40), false, false),
    ];
    assert_eq!(
        find_update(frames.clone(), 0, SearchDirection::Forward).unwrap(),
        (boxcars::Attribute::Int(20), 1)
    );
    assert_eq!(
        find_update(frames.clone(), 4, SearchDirection::Backward).unwrap(),
        (boxcars::Attribute::Int(30), 3)
    );
    for (index, direction) in [
        (1, SearchDirection::Forward),
        (3, SearchDirection::Backward),
    ] {
        assert!(matches!(
            find_update(frames.clone(), index, direction)
                .unwrap_err()
                .variant,
            SubtrActorErrorVariant::NoUpdateAfterFrame { .. }
        ));
    }
}

#[test]
fn update_search_does_not_cross_same_frame_actor_reuse() {
    let frames = vec![
        frame(Some(10), true, false),
        frame(Some(20), true, true),
        frame(None, false, false),
    ];
    for (index, direction) in [
        (0, SearchDirection::Forward),
        (1, SearchDirection::Backward),
    ] {
        assert!(matches!(
            find_update(frames.clone(), index, direction)
                .unwrap_err()
                .variant,
            SubtrActorErrorVariant::NoUpdateAfterFrame { .. }
        ));
    }
    assert_eq!(
        find_update(frames, 2, SearchDirection::Backward).unwrap(),
        (boxcars::Attribute::Int(20), 1)
    );
}

#[test]
fn backward_update_search_stops_at_spawn_without_property_update() {
    let frames = vec![
        frame(Some(10), true, false),
        frame(None, false, true),
        frame(None, true, false),
        frame(Some(20), false, false),
    ];
    assert!(matches!(
        find_update(frames, 3, SearchDirection::Backward)
            .unwrap_err()
            .variant,
        SubtrActorErrorVariant::NoUpdateAfterFrame { .. }
    ));
}

#[test]
fn update_search_rejects_out_of_bounds_indices() {
    for direction in [SearchDirection::Forward, SearchDirection::Backward] {
        for index in [1, usize::MAX] {
            assert!(matches!(
                find_update(vec![frame(Some(10), true, false)], index, direction)
                    .unwrap_err()
                    .variant,
                SubtrActorErrorVariant::FrameIndexOutOfBounds
            ));
        }
        assert!(matches!(
            find_update(Vec::new(), 0, direction).unwrap_err().variant,
            SubtrActorErrorVariant::FrameIndexOutOfBounds
        ));
    }
}

#[test]
fn rigid_body_extrapolates_when_no_update_remains_in_its_lifetime() {
    let bytes = std::fs::read("assets/post-eac-ranked-duel-2026-04-28-a.replay").unwrap();
    let replay = boxcars::ParserBuilder::new(&bytes)
        .must_parse_network_data()
        .parse()
        .unwrap();
    for reused in [false, true] {
        let mut processor = ReplayProcessor::new(&replay).unwrap();
        processor.process_long_enough_to_get_actor_ids().unwrap();
        let car = processor.get_car_actor_id(&processor.team_zero[0]).unwrap();
        let property = *processor
            .get_object_id_for_key(RIGID_BODY_STATE_KEY)
            .unwrap();
        let mut body = *processor.get_actor_rigid_body(&car).unwrap().0;
        body.location = boxcars::Vector3f {
            x: 100.0,
            y: 0.0,
            z: 0.0,
        };
        body.linear_velocity = Some(boxcars::Vector3f {
            x: 10.0,
            y: 0.0,
            z: 0.0,
        });
        body.angular_velocity = None;
        processor
            .actor_state
            .actor_states
            .get_mut(&car)
            .unwrap()
            .attributes
            .insert(property, (boxcars::Attribute::RigidBody(body), 0));
        let mut sample = replay.clone();
        let mut frames = vec![frame(None, false, false), frame(None, false, false)];
        frames[1].time = 1.0;
        if reused {
            let mut deletion = frame(None, false, false);
            deletion.deleted_actors.push(car);
            deletion.time = 2.0;
            frames.push(deletion);
            let mut replacement = frame(None, false, false);
            replacement.time = 3.0;
            let mut new_body = body;
            new_body.location.x = 10000.0;
            replacement.updated_actors.push(boxcars::UpdatedAttribute {
                actor_id: car,
                object_id: property,
                stream_id: boxcars::StreamId(0),
                attribute: boxcars::Attribute::RigidBody(new_body),
            });
            frames.push(replacement);
        }
        sample.network_frames.as_mut().unwrap().frames = frames;
        processor.replay = &sample;
        let actual = processor
            .get_interpolated_actor_rigid_body(&car, 1.0, 0.0)
            .unwrap();
        let expected = processor.normalize_rigid_body(&body);
        assert_eq!(
            actual.location.x,
            expected.location.x + expected.linear_velocity.unwrap().x
        );
        assert_eq!(actual.location.y, 0.0);
        assert!(actual.location.x.is_finite());
    }
}
