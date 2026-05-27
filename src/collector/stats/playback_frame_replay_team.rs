use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(crate) fn replay_team_stats(
        &self,
        frame: &StatsSnapshotFrame,
        team_key: &str,
    ) -> SubtrActorResult<TeamStatsSnapshot> {
        let is_team_zero = team_key == "team_zero";
        Ok(TeamStatsSnapshot {
            fifty_fifty: self
                .frame_stats_or_default_typed::<FiftyFiftyStats>(frame, "fifty_fifty")?
                .for_team(is_team_zero),
            possession: self
                .frame_stats_or_default_typed::<PossessionStats>(frame, "possession")?
                .for_team(is_team_zero),
            pressure: self
                .frame_stats_or_default_typed::<PressureStats>(frame, "pressure")?
                .for_team(is_team_zero),
            territorial_pressure: self
                .frame_stats_or_default_typed::<TerritorialPressureStats>(
                    frame,
                    "territorial_pressure",
                )?
                .for_team(is_team_zero),
            rotation: self.frame_team_stat_or_default_typed(frame, "rotation", team_key)?,
            rush: self
                .frame_stats_or_default_typed::<RushStats>(frame, "rush")?
                .for_team(is_team_zero),
            core: self.frame_team_stat_or_default_typed(frame, "core", team_key)?,
            backboard: self.frame_team_stat_or_default_typed(frame, "backboard", team_key)?,
            double_tap: self.frame_team_stat_or_default_typed(frame, "double_tap", team_key)?,
            one_timer: self.frame_team_stat_or_default_typed(frame, "one_timer", team_key)?,
            pass: self.frame_team_stat_or_default_typed(frame, "pass", team_key)?,
            ball_carry: self.frame_team_stat_or_default_typed(frame, "ball_carry", team_key)?,
            air_dribble: self.frame_team_stat_or_default_typed(frame, "air_dribble", team_key)?,
            boost: self.frame_team_stat_or_default_typed(frame, "boost", team_key)?,
            bump: self.frame_team_stat_or_default_typed(frame, "bump", team_key)?,
            half_volley: self.frame_team_stat_or_default_typed(frame, "half_volley", team_key)?,
            movement: self.frame_team_stat_or_default_typed(frame, "movement", team_key)?,
            powerslide: self.frame_team_stat_or_default_typed(frame, "powerslide", team_key)?,
            demo: self.frame_team_stat_or_default_typed(frame, "demo", team_key)?,
        })
    }
}
