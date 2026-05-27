use super::match_stats_delta::*;
use super::*;

impl MatchStatsCalculator {
    pub(super) fn emit_timeline_events(
        &mut self,
        time: f32,
        frame: Option<usize>,
        kind: TimelineEventKind,
        player_id: &PlayerId,
        is_team_0: bool,
        delta: i32,
    ) {
        for _ in 0..delta.max(0) {
            self.timeline.push(TimelineEvent {
                time,
                frame,
                kind,
                player_id: Some(player_id.clone()),
                is_team_0: Some(is_team_0),
            });
        }
    }

    pub(super) fn emit_core_stats_events(&mut self, frame: &FrameInfo) {
        self.emit_core_player_stats_events(frame);
        self.emit_core_team_stats_events(frame);
    }

    fn emit_core_player_stats_events(&mut self, frame: &FrameInfo) {
        let mut player_ids: Vec<_> = self.player_stats.keys().cloned().collect();
        player_ids.sort_by(|left, right| format!("{left:?}").cmp(&format!("{right:?}")));
        for player_id in player_ids {
            self.emit_core_player_stats_event(frame, player_id);
        }
    }

    fn emit_core_player_stats_event(&mut self, frame: &FrameInfo, player_id: PlayerId) {
        let Some(stats) = self.player_stats.get(&player_id) else {
            return;
        };
        let previous_stats = self
            .last_emitted_player_stats
            .get(&player_id)
            .cloned()
            .unwrap_or_default();
        if previous_stats == *stats {
            return;
        }
        let Some(is_team_0) = self.player_teams.get(&player_id).copied() else {
            return;
        };
        self.core_player_events.push(CorePlayerStatsEvent {
            time: frame.time,
            frame: frame.frame_number,
            player: player_id.clone(),
            is_team_0,
            delta: core_player_stats_delta(stats, &previous_stats),
        });
        self.last_emitted_player_stats
            .insert(player_id, stats.clone());
    }

    fn emit_core_team_stats_events(&mut self, frame: &FrameInfo) {
        let team_zero_stats = self.team_zero_stats();
        if team_zero_stats != self.last_emitted_team_zero_stats {
            self.core_team_events.push(CoreTeamStatsEvent {
                time: frame.time,
                frame: frame.frame_number,
                is_team_0: true,
                delta: core_team_stats_delta(&team_zero_stats, &self.last_emitted_team_zero_stats),
            });
            self.last_emitted_team_zero_stats = team_zero_stats;
        }

        let team_one_stats = self.team_one_stats();
        if team_one_stats != self.last_emitted_team_one_stats {
            self.core_team_events.push(CoreTeamStatsEvent {
                time: frame.time,
                frame: frame.frame_number,
                is_team_0: false,
                delta: core_team_stats_delta(&team_one_stats, &self.last_emitted_team_one_stats),
            });
            self.last_emitted_team_one_stats = team_one_stats;
        }
    }
}
