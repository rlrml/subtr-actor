use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(crate) fn insert_timeline_team_module_stats(
        &self,
        team: &mut Map<String, Value>,
        frame: &StatsSnapshotFrame,
        team_key: &str,
    ) -> SubtrActorResult<()> {
        team.insert(
            "core".to_owned(),
            self.frame_team_stat_or_default::<CoreTeamStats>(frame, "core", team_key),
        );
        team.insert(
            "backboard".to_owned(),
            self.frame_team_stat_or_default::<BackboardTeamStats>(frame, "backboard", team_key),
        );
        team.insert(
            "double_tap".to_owned(),
            self.frame_team_stat_or_default::<DoubleTapTeamStats>(frame, "double_tap", team_key),
        );
        team.insert(
            "one_timer".to_owned(),
            self.frame_team_stat_or_default::<OneTimerTeamStats>(frame, "one_timer", team_key),
        );
        team.insert(
            "pass".to_owned(),
            self.frame_team_stat_or_default::<PassTeamStats>(frame, "pass", team_key),
        );
        team.insert(
            "ball_carry".to_owned(),
            self.frame_team_stat_or_default::<BallCarryStats>(frame, "ball_carry", team_key),
        );
        team.insert(
            "air_dribble".to_owned(),
            self.frame_team_stat_or_default::<AirDribbleStats>(frame, "air_dribble", team_key),
        );
        team.insert(
            "boost".to_owned(),
            self.frame_team_stat_or_default::<BoostStats>(frame, "boost", team_key),
        );
        team.insert(
            "bump".to_owned(),
            self.frame_team_stat_or_default::<BumpTeamStats>(frame, "bump", team_key),
        );
        team.insert(
            "half_volley".to_owned(),
            self.frame_team_stat_or_default::<HalfVolleyTeamStats>(frame, "half_volley", team_key),
        );
        team.insert(
            "movement".to_owned(),
            self.frame_team_stat_or_default::<MovementStats>(frame, "movement", team_key),
        );
        team.insert(
            "powerslide".to_owned(),
            self.frame_team_stat_or_default::<PowerslideStats>(frame, "powerslide", team_key),
        );
        team.insert(
            "demo".to_owned(),
            self.frame_team_stat_or_default::<DemoTeamStats>(frame, "demo", team_key),
        );
        Ok(())
    }
}
