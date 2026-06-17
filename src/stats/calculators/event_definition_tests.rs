use super::*;

fn assert_stats_event<T: StatsEvent>() {}

#[test]
fn current_event_payloads_implement_stats_event() {
    assert_stats_event::<TimelineEvent>();
    assert_stats_event::<CorePlayerScoreboardEvent>();
    assert_stats_event::<BackboardBounceEvent>();
    assert_stats_event::<CeilingShotEvent>();
    assert_stats_event::<WallAerialEvent>();
    assert_stats_event::<WallAerialShotEvent>();
    assert_stats_event::<CenterEvent>();
    assert_stats_event::<FlickEvent>();
    assert_stats_event::<MustyFlickEvent>();
    assert_stats_event::<DodgeResetEvent>();
    assert_stats_event::<DoubleTapEvent>();
    assert_stats_event::<OneTimerEvent>();
    assert_stats_event::<PassEvent>();
    assert_stats_event::<BallCarryEvent>();
    assert_stats_event::<FiftyFiftyEvent>();
    assert_stats_event::<RushEvent>();
    assert_stats_event::<SpeedFlipEvent>();
    assert_stats_event::<HalfFlipEvent>();
    assert_stats_event::<HalfVolleyEvent>();
    assert_stats_event::<WavedashEvent>();
    assert_stats_event::<WhiffEvent>();
    assert_stats_event::<PowerslideEvent>();
    assert_stats_event::<TouchClassificationEvent>();
    assert_stats_event::<BoostPickupEvent>();
    assert_stats_event::<RespawnEvent>();
    assert_stats_event::<BumpEvent>();
    assert_stats_event::<PossessionEvent>();
    assert_stats_event::<BallHalfEvent>();
    assert_stats_event::<BallThirdEvent>();
    assert_stats_event::<TerritorialPressureEvent>();
    assert_stats_event::<MovementEvent>();
    assert_stats_event::<PlayerActivityEvent>();
    assert_stats_event::<FieldThirdEvent>();
    assert_stats_event::<FieldHalfEvent>();
    assert_stats_event::<BallDepthEvent>();
    assert_stats_event::<DepthRoleEvent>();
    assert_stats_event::<BallProximityEvent>();
    assert_stats_event::<RotationRoleEvent>();
    assert_stats_event::<FirstManChangeEvent>();
    assert_stats_event::<FlipResetEvent>();
    assert_stats_event::<Event>();
}

#[test]
fn event_definitions_start_unknown_and_no_evidence() {
    for definition in all_event_definitions() {
        assert_eq!(
            definition.confidence, UNKNOWN_DETECTION_CONFIDENCE,
            "{} should start with unknown confidence metadata",
            definition.id
        );
        assert!(definition.limitations.is_empty());
    }
}

#[test]
fn mechanic_event_definitions_have_documented_approaches() {
    for definition in all_event_definitions() {
        if definition.category != EventCategory::Mechanic {
            continue;
        }

        assert_ne!(
            definition.summary, "Definition pending.",
            "{} should describe what the mechanic event means",
            definition.id
        );
        assert!(
            !definition.approach.is_empty(),
            "{} should describe how the mechanic event is detected",
            definition.id
        );
    }
}

#[test]
fn low_level_interaction_events_are_other() {
    for definition in [
        TOUCH_CLASSIFICATION_EVENT_DEFINITION,
        WHIFF_EVENT_DEFINITION,
        MOVEMENT_EVENT_DEFINITION,
    ] {
        assert_eq!(
            definition.category,
            EventCategory::Other,
            "{}",
            definition.id
        );
    }
}

#[test]
fn event_definition_ids_are_unique() {
    // Iterate the raw distributed slice (not the de-duplicated accessor) so an
    // accidental double registration of the same id fails loudly here.
    let mut ids = std::collections::BTreeSet::new();
    for definition in EVENT_DEFINITIONS {
        assert!(
            ids.insert(definition.id),
            "duplicate event definition id {}",
            definition.id
        );
    }
}

#[test]
fn every_produced_event_has_a_registered_definition() {
    let registered: std::collections::BTreeSet<&str> =
        all_event_definitions().iter().map(|def| def.id).collect();
    for producer in event_producers() {
        for emitted in producer.emitted_events {
            assert!(
                registered.contains(emitted.event.id),
                "event {:?} is produced by {:?} but is missing from the definition registry",
                emitted.event.id,
                producer.node_name
            );
        }
    }
}

#[test]
fn expansion_parents_are_hidden_from_review() {
    // A parent that declares variants surfaces those instead of itself, so it
    // must be hidden or the raw parent key leaks into the review picker.
    for definition in all_event_definitions() {
        if !definition.variants.is_empty() {
            assert!(
                definition.hidden_from_review,
                "{} declares variants but is not hidden_from_review",
                definition.id
            );
        }
    }
}
