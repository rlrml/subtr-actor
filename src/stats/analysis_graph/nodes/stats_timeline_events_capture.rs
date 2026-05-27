use super::*;

impl StatsTimelineEventsNode {
    pub(super) fn capture_events(
        &mut self,
        ctx: &AnalysisStateContext<'_>,
    ) -> SubtrActorResult<()> {
        let sources = StatsTimelineEventSources::from_context(ctx)?;
        self.state.events = build_replay_stats_timeline_events(&sources);
        Ok(())
    }
}

fn build_replay_stats_timeline_events(
    sources: &StatsTimelineEventSources<'_>,
) -> ReplayStatsTimelineEvents {
    let mut timeline = sources.match_stats.timeline().to_vec();
    timeline.extend(sources.demo.timeline().to_vec());
    timeline.sort_by(|left, right| left.time.total_cmp(&right.time));

    ReplayStatsTimelineEvents {
        timeline,
        core_player: sources.match_stats.core_player_events().to_vec(),
        core_team: sources.match_stats.core_team_events().to_vec(),
        possession: sources.possession.events().to_vec(),
        pressure: sources.pressure.events().to_vec(),
        territorial_pressure: sources.territorial_pressure.events().to_vec(),
        movement: sources.movement.events().to_vec(),
        positioning: sources.positioning.events().to_vec(),
        rotation_player: sources.rotation.player_events().to_vec(),
        rotation_team: sources.rotation.team_events().to_vec(),
        mechanics: build_mechanic_events(&sources.mechanics),
        goal_context: sources.match_stats.goal_context_events().to_vec(),
        backboard: sources.backboard.events().to_vec(),
        ceiling_shot: sources.mechanics.ceiling_shot.events().to_vec(),
        wall_aerial: sources.mechanics.wall_aerial.events().to_vec(),
        wall_aerial_shot: sources.mechanics.wall_aerial_shot.events().to_vec(),
        center: sources.mechanics.center.events().to_vec(),
        flick: sources.mechanics.flick.events().to_vec(),
        musty_flick: sources.mechanics.musty_flick.events().to_vec(),
        dodge_reset: sources.mechanics.dodge_reset.events().to_vec(),
        double_tap: sources.mechanics.double_tap.events().to_vec(),
        one_timer: sources.mechanics.one_timer.events().to_vec(),
        pass: sources.mechanics.pass.events().to_vec(),
        pass_last_completed: sources.mechanics.pass.last_completed_events().to_vec(),
        ball_carry: sources.mechanics.ball_carry.carry_events().to_vec(),
        fifty_fifty: sources.fifty_fifty.events().to_vec(),
        goal_tags: sources.goal_tags.combined_events(),
        rush: sources.rush.events().to_vec(),
        speed_flip: sources.mechanics.speed_flip.events().to_vec(),
        half_flip: sources.mechanics.half_flip.events().to_vec(),
        half_volley: sources.mechanics.half_volley.events().to_vec(),
        wavedash: sources.mechanics.wavedash.events().to_vec(),
        whiff: sources.whiff.events().to_vec(),
        powerslide: sources.powerslide.events().to_vec(),
        touch: sources.touch.events().to_vec(),
        touch_ball_movement: sources.touch.ball_movement_events().to_vec(),
        touch_last_touch: sources.touch.last_touch_events().to_vec(),
        boost_pickups: sources.boost.pickup_comparison_events().to_vec(),
        boost_ledger: sources.boost.ledger_events().to_vec(),
        boost_state: sources.boost.state_events().to_vec(),
        bump: sources.bump.events().to_vec(),
    }
}
