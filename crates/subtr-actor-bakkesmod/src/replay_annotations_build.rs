use super::*;

pub(super) fn build_replay_annotations(path: &CStr) -> SubtrActorResult<SaReplayAnnotations> {
    let path = path.to_str().map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
            "invalid replay path utf-8: {error}"
        )))
    })?;
    let bytes = std::fs::read(path).map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
            "could not read replay file {path}: {error}"
        )))
    })?;
    let replay = ParserBuilder::new(&bytes)
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()
        .map_err(|error| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
                "could not parse replay file {path}: {error}"
            )))
        })?;
    let timeline =
        StatsTimelineEventCollector::new().get_replay_stats_timeline_scaffold(&replay)?;
    let events = replay_annotations_from_timeline(&timeline.replay_meta, &timeline.events);
    Ok(SaReplayAnnotations {
        events,
        cursor: 0,
        last_poll_time: 0.0,
        initialized: false,
    })
}
