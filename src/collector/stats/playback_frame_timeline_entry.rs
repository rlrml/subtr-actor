use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(crate) fn timeline_frame_value(
        &self,
        frame: &StatsSnapshotFrame,
    ) -> SubtrActorResult<Value> {
        let mut timeline = Map::new();
        timeline.insert(
            "frame_number".to_owned(),
            serialize_to_json_value(&frame.frame_number)?,
        );
        timeline.insert("time".to_owned(), serialize_to_json_value(&frame.time)?);
        timeline.insert("dt".to_owned(), serialize_to_json_value(&frame.dt)?);
        timeline.insert(
            "seconds_remaining".to_owned(),
            serialize_to_json_value(&frame.seconds_remaining)?,
        );
        timeline.insert(
            "game_state".to_owned(),
            serialize_to_json_value(&frame.game_state)?,
        );
        timeline.insert(
            "ball_has_been_hit".to_owned(),
            serialize_to_json_value(&frame.ball_has_been_hit)?,
        );
        timeline.insert(
            "kickoff_countdown_time".to_owned(),
            serialize_to_json_value(&frame.kickoff_countdown_time)?,
        );
        timeline.insert(
            "gameplay_phase".to_owned(),
            serialize_to_json_value(&frame.gameplay_phase)?,
        );
        timeline.insert(
            "is_live_play".to_owned(),
            serialize_to_json_value(&frame.is_live_play)?,
        );
        timeline.insert(
            "fifty_fifty".to_owned(),
            self.frame_stats_or_default::<FiftyFiftyStats>(frame, "fifty_fifty"),
        );
        timeline.insert(
            "possession".to_owned(),
            self.frame_stats_or_default::<PossessionStats>(frame, "possession"),
        );
        timeline.insert(
            "pressure".to_owned(),
            self.frame_stats_or_default::<PressureStats>(frame, "pressure"),
        );
        timeline.insert(
            "territorial_pressure".to_owned(),
            self.frame_stats_or_default::<TerritorialPressureStats>(frame, "territorial_pressure"),
        );
        timeline.insert(
            "rush".to_owned(),
            self.frame_stats_or_default::<RushStats>(frame, "rush"),
        );
        timeline.insert(
            "team_zero".to_owned(),
            self.timeline_team_value(frame, "team_zero")?,
        );
        timeline.insert(
            "team_one".to_owned(),
            self.timeline_team_value(frame, "team_one")?,
        );
        timeline.insert(
            "players".to_owned(),
            Value::Array(
                self.replay_meta
                    .player_order()
                    .map(|player| self.timeline_player_value(frame, player))
                    .collect::<SubtrActorResult<Vec<_>>>()?,
            ),
        );
        Ok(Value::Object(timeline))
    }
}
