use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(in crate::collector::stats::playback) fn timeline_event_sets_typed(
        &self,
    ) -> SubtrActorResult<ReplayStatsTimelineEvents> {
        macro_rules! events {
            ($module:literal, $field:literal, $parser:ident) => {
                self.module_player_events($module, $field, $parser)?
            };
        }

        Ok(ReplayStatsTimelineEvents {
            timeline: self.timeline_events_typed()?,
            core_player: events!("core", "player_events", parse_core_player_stats_event),
            core_team: events!("core", "team_events", parse_core_team_stats_event),
            possession: events!("possession", "events", parse_possession_event),
            pressure: events!("pressure", "events", parse_pressure_event),
            territorial_pressure: events!(
                "territorial_pressure",
                "events",
                parse_territorial_pressure_event
            ),
            movement: events!("movement", "events", parse_movement_event),
            positioning: events!("positioning", "events", parse_positioning_event),
            rotation_player: events!("rotation", "player_events", parse_rotation_player_event),
            rotation_team: events!("rotation", "team_events", parse_rotation_team_event),
            mechanics: self.mechanic_events_typed()?,
            goal_context: events!("core", "goal_context", parse_goal_context_event),
            backboard: events!("backboard", "events", parse_backboard_event),
            ceiling_shot: events!("ceiling_shot", "events", parse_ceiling_shot_event),
            wall_aerial: events!("wall_aerial", "events", parse_wall_aerial_event),
            wall_aerial_shot: events!("wall_aerial_shot", "events", parse_wall_aerial_shot_event),
            center: events!("center", "events", parse_center_event),
            flick: events!("flick", "events", parse_flick_event),
            musty_flick: events!("musty_flick", "events", parse_musty_flick_event),
            dodge_reset: events!("dodge_reset", "events", parse_dodge_reset_event),
            double_tap: events!("double_tap", "events", parse_double_tap_event),
            one_timer: events!("one_timer", "events", parse_one_timer_event),
            fifty_fifty: events!("fifty_fifty", "events", parse_fifty_fifty_event),
            pass: events!("pass", "events", parse_pass_event),
            pass_last_completed: events!(
                "pass",
                "last_completed_events",
                parse_pass_last_completed_event
            ),
            ball_carry: events!("ball_carry", "events", parse_ball_carry_event),
            goal_tags: self.goal_tag_events_typed()?,
            rush: self.module_typed_array("rush", "events")?,
            speed_flip: events!("speed_flip", "events", parse_speed_flip_event),
            half_flip: events!("half_flip", "events", parse_half_flip_event),
            half_volley: events!("half_volley", "events", parse_half_volley_event),
            wavedash: events!("wavedash", "events", parse_wavedash_event),
            whiff: events!("whiff", "events", parse_whiff_event),
            powerslide: events!("powerslide", "events", parse_powerslide_event),
            touch: events!("touch", "events", parse_touch_stats_event),
            touch_ball_movement: events!(
                "touch",
                "ball_movement_events",
                parse_touch_ball_movement_event
            ),
            touch_last_touch: events!("touch", "last_touch_events", parse_touch_last_touch_event),
            boost_pickups: events!("boost", "events", parse_boost_pickup_comparison_event),
            boost_ledger: events!("boost", "ledger_events", parse_boost_ledger_event),
            boost_state: events!("boost", "state_events", parse_boost_state_event),
            bump: events!("bump", "events", parse_bump_event),
        })
    }
}
