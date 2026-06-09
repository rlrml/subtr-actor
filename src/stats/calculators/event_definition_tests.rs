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
    assert_stats_event::<DodgeRefreshedEvent>();
    assert_stats_event::<ConfirmedFlipResetEvent>();
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
    assert_stats_event::<BoostPickupComparisonEvent>();
    assert_stats_event::<BoostLedgerEvent>();
    assert_stats_event::<BoostBucketEvent>();
    assert_stats_event::<BoostStateEvent>();
    assert_stats_event::<BumpEvent>();
    assert_stats_event::<PossessionEvent>();
    assert_stats_event::<PressureEvent>();
    assert_stats_event::<TerritorialPressureEvent>();
    assert_stats_event::<MovementEvent>();
    assert_stats_event::<PositioningActivityEvent>();
    assert_stats_event::<PositioningFieldZoneEvent>();
    assert_stats_event::<PositioningBallDepthEvent>();
    assert_stats_event::<PositioningTeammateRoleEvent>();
    assert_stats_event::<PositioningBallProximityEvent>();
    assert_stats_event::<PositioningGoalContextEvent>();
    assert_stats_event::<RotationPlayerEvent>();
    assert_stats_event::<RotationTeamEvent>();
    assert_stats_event::<FlipResetEvent>();
    assert_stats_event::<PostWallDodgeEvent>();
    assert_stats_event::<FlipResetFollowupDodgeEvent>();
    assert_stats_event::<Event>();
}

#[test]
fn event_definitions_start_unknown_and_no_evidence() {
    for definition in ALL_EVENT_DEFINITIONS {
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
    for definition in ALL_EVENT_DEFINITIONS {
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
fn low_level_ball_interaction_events_are_other() {
    for definition in [
        TOUCH_CLASSIFICATION_EVENT_DEFINITION,
        WHIFF_EVENT_DEFINITION,
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
    let mut ids = std::collections::BTreeSet::new();
    for definition in ALL_EVENT_DEFINITIONS {
        assert!(
            ids.insert(definition.id),
            "duplicate event definition id {}",
            definition.id
        );
    }
}
