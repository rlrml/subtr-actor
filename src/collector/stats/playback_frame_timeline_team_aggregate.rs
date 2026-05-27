use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(crate) fn insert_timeline_team_aggregate_stats(
        &self,
        team: &mut Map<String, Value>,
        frame: &StatsSnapshotFrame,
        team_key: &str,
        is_team_zero: bool,
    ) -> SubtrActorResult<()> {
        team.insert(
            "fifty_fifty".to_owned(),
            serialize_to_json_value(
                &self
                    .frame_stats_or_default_typed::<FiftyFiftyStats>(frame, "fifty_fifty")?
                    .for_team(is_team_zero),
            )?,
        );
        team.insert(
            "possession".to_owned(),
            serialize_to_json_value(
                &self
                    .frame_stats_or_default_typed::<PossessionStats>(frame, "possession")?
                    .for_team(is_team_zero),
            )?,
        );
        team.insert(
            "pressure".to_owned(),
            serialize_to_json_value(
                &self
                    .frame_stats_or_default_typed::<PressureStats>(frame, "pressure")?
                    .for_team(is_team_zero),
            )?,
        );
        team.insert(
            "territorial_pressure".to_owned(),
            serialize_to_json_value(
                &self
                    .frame_stats_or_default_typed::<TerritorialPressureStats>(
                        frame,
                        "territorial_pressure",
                    )?
                    .for_team(is_team_zero),
            )?,
        );
        team.insert(
            "rotation".to_owned(),
            self.frame_team_stat_or_default::<RotationTeamStats>(frame, "rotation", team_key),
        );
        team.insert(
            "rush".to_owned(),
            serialize_to_json_value(
                &self
                    .frame_stats_or_default_typed::<RushStats>(frame, "rush")?
                    .for_team(is_team_zero),
            )?,
        );
        Ok(())
    }
}
