//! Pins the per-event-type fan-out scope so timeline lane behavior cannot
//! silently regress. `scope` is declared next to each `define_stats_event!` and
//! read back through [`EventPayload::scope`]; these tests assert the declared
//! values rather than the routing, since the match in `EventPayload::scope` is
//! already exhaustiveness-checked by the compiler.

use super::*;
use crate::stats::timeline::EventScope;

#[test]
fn fan_out_defaults_to_match() {
    // Fan-out is opt-in: a definition that declares no scope stays on a single
    // shared lane rather than splitting per player.
    assert_eq!(
        pending_event_definition("example", "Example", EventCategory::Other).scope,
        EventScope::Match,
    );
    assert_eq!(SHOT_EVENT_DEFINITION.scope, EventScope::Match);
    assert_eq!(TIMELINE_EVENT_DEFINITION.scope, EventScope::Match);
    assert_eq!(
        CORE_PLAYER_SCOREBOARD_EVENT_DEFINITION.scope,
        EventScope::Match
    );
    assert_eq!(GOAL_CONTEXT_EVENT_DEFINITION.scope, EventScope::Match);
}

#[test]
fn control_and_contest_streams_are_team_scoped() {
    for def in [
        POSSESSION_EVENT_DEFINITION,
        PRESSURE_EVENT_DEFINITION, // ball_half
        TERRITORIAL_PRESSURE_EVENT_DEFINITION,
        CONTROLLED_PLAY_EVENT_DEFINITION,
        FIFTY_FIFTY_EVENT_DEFINITION,
        RUSH_EVENT_DEFINITION,
    ] {
        assert_eq!(
            def.scope,
            EventScope::Team,
            "stream {} should be team-scoped",
            def.id
        );
    }
}

#[test]
fn per_player_streams_fan_out_to_player_lanes() {
    // Positioning span streams are the streams that would otherwise pile
    // overlapping spans onto a single lane, plus the per-player mechanic and
    // possession streams that were per-player before scope became declarative.
    for def in [
        FIELD_HALF_EVENT_DEFINITION,
        FIELD_THIRD_EVENT_DEFINITION,
        BALL_DEPTH_EVENT_DEFINITION,
        BALL_THIRD_EVENT_DEFINITION,
        DEPTH_ROLE_EVENT_DEFINITION,
        BALL_PROXIMITY_EVENT_DEFINITION,
        ROTATION_ROLE_EVENT_DEFINITION,
        FIRST_MAN_CHANGE_EVENT_DEFINITION,
        PLAYER_ACTIVITY_EVENT_DEFINITION,
        MOVEMENT_EVENT_DEFINITION,
        PLAYER_POSSESSION_EVENT_DEFINITION,
        KICKOFF_EVENT_DEFINITION,
        TOUCH_CLASSIFICATION_EVENT_DEFINITION,
        FLICK_EVENT_DEFINITION,
        DODGE_EVENT_DEFINITION,
        DEMOLITION_EVENT_DEFINITION,
        BOOST_PICKUP_EVENT_DEFINITION,
    ] {
        assert_eq!(
            def.scope,
            EventScope::Player,
            "stream {} should be player-scoped",
            def.id
        );
    }
}
