use super::*;

pub(crate) type FirstManCounts = HashMap<PlayerId, u32>;

impl RotationCalculator {
    pub(crate) fn record_first_man_change(
        &mut self,
        is_team_0: bool,
        frame: &FrameInfo,
        raw_first_man: Option<&PlayerId>,
    ) -> (FirstManCounts, FirstManCounts) {
        let mut became_first_man_counts = HashMap::<PlayerId, u32>::new();
        let mut lost_first_man_counts = HashMap::<PlayerId, u32>::new();
        let debounce_seconds = self.config.first_man_debounce_seconds;
        let change =
            self.team_tracker_mut(is_team_0)
                .update(raw_first_man, frame.dt, debounce_seconds);

        if let Some((previous, next)) = change {
            self.record_team_rotation(is_team_0, frame);
            self.player_stats
                .entry(previous.clone())
                .or_default()
                .lost_first_man_count += 1;
            *lost_first_man_counts.entry(previous).or_default() += 1;
            self.player_stats
                .entry(next.clone())
                .or_default()
                .became_first_man_count += 1;
            *became_first_man_counts.entry(next).or_default() += 1;
        }

        (became_first_man_counts, lost_first_man_counts)
    }

    fn record_team_rotation(&mut self, is_team_0: bool, frame: &FrameInfo) {
        let team_stats = self.team_stats_mut(is_team_0);
        team_stats.first_man_changes_for_team += 1;
        team_stats.rotation_count += 1;
        self.team_events.push(RotationTeamEvent {
            time: frame.time,
            frame: frame.frame_number,
            is_team_0,
            first_man_changes_for_team: 1,
            rotation_count: 1,
        });
    }
}
