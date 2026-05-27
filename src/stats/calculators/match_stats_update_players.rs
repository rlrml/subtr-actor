use super::*;

impl MatchStatsCalculator {
    pub(super) fn update_player_core_stats(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        processor_event_counts: &HashMap<(PlayerId, TimelineEventKind), i32>,
    ) {
        for player in &players.players {
            self.player_teams
                .insert(player.player_id.clone(), player.is_team_0);
            let mut current_stats = self.current_player_stats(player);
            let previous_stats = self
                .previous_player_stats
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default();

            self.emit_fallback_player_stat_events(
                frame,
                player,
                &current_stats,
                &previous_stats,
                processor_event_counts,
            );
            self.record_player_goal_deltas(frame, player, &mut current_stats, &previous_stats);

            self.previous_player_stats
                .insert(player.player_id.clone(), current_stats.clone());
            self.player_stats
                .insert(player.player_id.clone(), current_stats);
        }
    }

    fn current_player_stats(&self, player: &PlayerSample) -> CorePlayerStats {
        CorePlayerStats {
            score: player.match_score.unwrap_or(0),
            goals: player.match_goals.unwrap_or(0),
            assists: player.match_assists.unwrap_or(0),
            saves: player.match_saves.unwrap_or(0),
            shots: player.match_shots.unwrap_or(0),
            scoring_context: self
                .player_stats
                .get(&player.player_id)
                .map(|stats| stats.scoring_context.clone())
                .unwrap_or_default(),
        }
    }

    fn emit_fallback_player_stat_events(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        current_stats: &CorePlayerStats,
        previous_stats: &CorePlayerStats,
        processor_event_counts: &HashMap<(PlayerId, TimelineEventKind), i32>,
    ) {
        let deltas = [
            (
                TimelineEventKind::Shot,
                current_stats.shots - previous_stats.shots,
            ),
            (
                TimelineEventKind::Save,
                current_stats.saves - previous_stats.saves,
            ),
            (
                TimelineEventKind::Assist,
                current_stats.assists - previous_stats.assists,
            ),
        ];
        for (kind, raw_delta) in deltas {
            let emitted = processor_event_counts
                .get(&(player.player_id.clone(), kind))
                .copied()
                .unwrap_or(0);
            self.emit_timeline_events(
                frame.time,
                Some(frame.frame_number),
                kind,
                &player.player_id,
                player.is_team_0,
                raw_delta - emitted,
            );
        }
    }
}
