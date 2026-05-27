use super::*;

impl MatchStatsCalculator {
    pub(super) fn record_processor_stat_events(
        &mut self,
        events: &FrameEventsState,
    ) -> HashMap<(PlayerId, TimelineEventKind), i32> {
        let mut counts = HashMap::new();
        for event in &events.player_stat_events {
            let kind = match event.kind {
                PlayerStatEventKind::Shot => TimelineEventKind::Shot,
                PlayerStatEventKind::Save => TimelineEventKind::Save,
                PlayerStatEventKind::Assist => TimelineEventKind::Assist,
            };
            self.timeline.push(TimelineEvent {
                time: event.time,
                frame: Some(event.frame),
                kind,
                player_id: Some(event.player.clone()),
                is_team_0: Some(event.is_team_0),
            });
            *counts.entry((event.player.clone(), kind)).or_default() += 1;
        }
        counts
    }

    pub(super) fn sort_timeline(&mut self) {
        self.timeline.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}
