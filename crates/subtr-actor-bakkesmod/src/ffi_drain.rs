use super::*;

#[no_mangle]
/// Copies and removes pending events from the engine.
///
/// Returns the number of events copied into `out_events`.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_events` must point to writable
/// storage for at least `max_events` `SaMechanicEvent` values.
pub unsafe extern "C" fn subtr_actor_bakkesmod_drain_events(
    engine: *mut SaEngine,
    out_events: *mut SaMechanicEvent,
    max_events: usize,
) -> usize {
    let Some(engine) = engine.as_mut() else {
        return 0;
    };
    if out_events.is_null() || max_events == 0 {
        return 0;
    }

    let count = max_events.min(engine.pending_events.len());
    ptr::copy_nonoverlapping(engine.pending_events.as_ptr(), out_events, count);
    engine.pending_events.drain(..count);
    count
}

#[no_mangle]
/// Copies and removes pending team-owned events from the engine.
///
/// Returns the number of events copied into `out_events`.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_events` must point to writable
/// storage for at least `max_events` `SaTeamEvent` values.
pub unsafe extern "C" fn subtr_actor_bakkesmod_drain_team_events(
    engine: *mut SaEngine,
    out_events: *mut SaTeamEvent,
    max_events: usize,
) -> usize {
    let Some(engine) = engine.as_mut() else {
        return 0;
    };
    if out_events.is_null() || max_events == 0 {
        return 0;
    }

    let count = max_events.min(engine.pending_team_events.len());
    ptr::copy_nonoverlapping(engine.pending_team_events.as_ptr(), out_events, count);
    engine.pending_team_events.drain(..count);
    count
}

#[no_mangle]
/// Copies and removes pending goal-context events from the engine.
///
/// Returns the number of events copied into `out_events`.
///
/// # Safety
///
/// `engine` must be a valid engine pointer. `out_events` must point to writable
/// storage for at least `max_events` `SaGoalContextEvent` values.
pub unsafe extern "C" fn subtr_actor_bakkesmod_drain_goal_context_events(
    engine: *mut SaEngine,
    out_events: *mut SaGoalContextEvent,
    max_events: usize,
) -> usize {
    let Some(engine) = engine.as_mut() else {
        return 0;
    };
    if out_events.is_null() || max_events == 0 {
        return 0;
    }

    let count = max_events.min(engine.pending_goal_context_events.len());
    ptr::copy_nonoverlapping(
        engine.pending_goal_context_events.as_ptr(),
        out_events,
        count,
    );
    engine.pending_goal_context_events.drain(..count);
    count
}
