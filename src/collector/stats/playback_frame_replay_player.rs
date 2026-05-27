use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(crate) fn replay_player_stats(
        &self,
        frame: &StatsSnapshotFrame,
        player: &PlayerInfo,
    ) -> SubtrActorResult<PlayerStatsSnapshot> {
        let player_key = player_info_key(player)?;
        macro_rules! player_stat {
            ($module:literal) => {
                self.frame_player_stat_or_default_typed_by_key(frame, $module, &player_key)?
            };
        }

        Ok(PlayerStatsSnapshot {
            player_id: player.remote_id.clone(),
            name: player.name.clone(),
            is_team_0: self.is_team_zero_player(player),
            core: self.frame_core_player_stat_or_default_by_key(frame, &player_key)?,
            backboard: player_stat!("backboard"),
            ceiling_shot: player_stat!("ceiling_shot"),
            wall_aerial: player_stat!("wall_aerial"),
            wall_aerial_shot: player_stat!("wall_aerial_shot"),
            double_tap: player_stat!("double_tap"),
            one_timer: player_stat!("one_timer"),
            pass: player_stat!("pass"),
            fifty_fifty: player_stat!("fifty_fifty"),
            speed_flip: player_stat!("speed_flip"),
            half_flip: player_stat!("half_flip"),
            wavedash: player_stat!("wavedash"),
            touch: self.replay_player_touch_stats(frame, &player_key)?,
            whiff: player_stat!("whiff"),
            flick: player_stat!("flick"),
            musty_flick: player_stat!("musty_flick"),
            dodge_reset: player_stat!("dodge_reset"),
            ball_carry: player_stat!("ball_carry"),
            air_dribble: player_stat!("air_dribble"),
            boost: player_stat!("boost"),
            bump: player_stat!("bump"),
            half_volley: player_stat!("half_volley"),
            movement: self.replay_player_movement_stats(frame, &player_key)?,
            positioning: player_stat!("positioning"),
            rotation: player_stat!("rotation"),
            powerslide: player_stat!("powerslide"),
            demo: player_stat!("demo"),
        })
    }

    pub(crate) fn replay_player_touch_stats(
        &self,
        frame: &StatsSnapshotFrame,
        player_key: &str,
    ) -> SubtrActorResult<TouchStats> {
        if frame.modules.contains_key("touch") {
            self.frame_player_stat_or_default_with_by_key(frame, "touch", player_key, || {
                TouchStats::default().with_complete_labeled_touch_counts()
            })
        } else {
            self.frame_player_stat_or_default_typed_by_key(frame, "touch", player_key)
        }
    }

    pub(crate) fn replay_player_movement_stats(
        &self,
        frame: &StatsSnapshotFrame,
        player_key: &str,
    ) -> SubtrActorResult<MovementStats> {
        self.frame_player_stat_or_default_with_by_key(frame, "movement", player_key, || {
            MovementStats::default().with_complete_labeled_tracked_time()
        })
    }

    pub(crate) fn is_team_zero_player(&self, player: &PlayerInfo) -> bool {
        self.replay_meta
            .team_zero
            .iter()
            .any(|team_player| team_player.remote_id == player.remote_id)
    }
}
