use super::*;

#[path = "timeline_replay_base.rs"]
mod timeline_replay_base;
pub(crate) use timeline_replay_base::*;
#[path = "timeline_replay_mechanics.rs"]
mod timeline_replay_mechanics;
pub(crate) use timeline_replay_mechanics::*;
#[path = "timeline_replay_contact.rs"]
mod timeline_replay_contact;
pub(crate) use timeline_replay_contact::*;
#[path = "timeline_replay_timeline_events.rs"]
mod timeline_replay_timeline_events;
pub(crate) use timeline_replay_timeline_events::*;
#[path = "timeline_replay_core_player.rs"]
mod timeline_replay_core_player;
pub(crate) use timeline_replay_core_player::*;
#[path = "timeline_replay_fifty_fifty.rs"]
mod timeline_replay_fifty_fifty;
pub(crate) use timeline_replay_fifty_fifty::*;
#[path = "timeline_replay_goal_tags.rs"]
mod timeline_replay_goal_tags;
pub(crate) use timeline_replay_goal_tags::*;

pub(crate) fn replay_annotations_from_timeline(
    replay_meta: &ReplayMeta,
    timeline: &ReplayStatsTimelineEvents,
) -> Vec<SaMechanicEvent> {
    let index_map = replay_player_index_map(replay_meta);
    let mut events = Vec::new();
    let mut emitted_ids = HashSet::new();

    push_replay_mechanic_annotations(
        &mut events,
        &mut emitted_ids,
        &index_map,
        &timeline.mechanics,
    );
    push_replay_backboard_annotations(
        &mut events,
        &mut emitted_ids,
        &index_map,
        &timeline.backboard,
    );
    push_replay_whiff_annotations(&mut events, &mut emitted_ids, &index_map, &timeline.whiff);
    push_replay_boost_pickup_annotations(
        &mut events,
        &mut emitted_ids,
        &index_map,
        &timeline.boost_pickups,
    );
    push_replay_bump_annotations(&mut events, &mut emitted_ids, &index_map, &timeline.bump);
    push_replay_timeline_event_annotations(
        &mut events,
        &mut emitted_ids,
        &index_map,
        &timeline.timeline,
    );
    push_replay_core_player_annotations(
        &mut events,
        &mut emitted_ids,
        &index_map,
        &timeline.core_player,
    );
    push_replay_fifty_fifty_annotations(
        &mut events,
        &mut emitted_ids,
        &index_map,
        &timeline.fifty_fifty,
    );
    push_replay_goal_tag_annotations(
        &mut events,
        &mut emitted_ids,
        &index_map,
        &timeline.goal_tags,
    );

    sort_replay_annotations(&mut events);
    events
}
