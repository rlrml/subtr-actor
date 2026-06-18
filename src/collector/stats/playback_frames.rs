use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(in crate::collector::stats::playback) fn timeline_frame_value(
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
            "kickoff".to_owned(),
            self.frame_stats_or_default::<KickoffStats>(frame, "kickoff"),
        );
        timeline.insert(
            "possession".to_owned(),
            self.frame_stats_or_default::<PossessionStats>(frame, "possession"),
        );
        timeline.insert(
            "ball_half".to_owned(),
            self.frame_stats_or_default::<BallHalfStats>(frame, "ball_half"),
        );
        timeline.insert(
            "ball_third".to_owned(),
            self.frame_stats_or_default::<BallThirdStats>(frame, "ball_third"),
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

    pub(crate) fn replay_stats_frame(
        &self,
        frame: &StatsSnapshotFrame,
    ) -> SubtrActorResult<ReplayStatsFrame> {
        Ok(ReplayStatsFrame {
            frame_number: frame.frame_number,
            time: frame.time,
            dt: frame.dt,
            seconds_remaining: frame.seconds_remaining,
            game_state: frame.game_state,
            ball_has_been_hit: frame.ball_has_been_hit,
            kickoff_countdown_time: frame.kickoff_countdown_time,
            gameplay_phase: frame.gameplay_phase,
            is_live_play: frame.is_live_play,
            team_zero: self.replay_team_stats(frame, "team_zero")?,
            team_one: self.replay_team_stats(frame, "team_one")?,
            players: self
                .replay_meta
                .player_order()
                .map(|player| self.replay_player_stats(frame, player))
                .collect::<SubtrActorResult<Vec<_>>>()?,
        })
    }

    pub(in crate::collector::stats::playback) fn replay_team_stats(
        &self,
        frame: &StatsSnapshotFrame,
        team_key: &str,
    ) -> SubtrActorResult<TeamStatsSnapshot> {
        let is_team_zero = team_key == "team_zero";
        Ok(TeamStatsSnapshot {
            fifty_fifty: self
                .frame_stats_or_default_typed::<FiftyFiftyStats>(frame, "fifty_fifty")?
                .for_team(is_team_zero),
            kickoff: self
                .frame_stats_or_default_typed::<KickoffStats>(frame, "kickoff")?
                .for_team(is_team_zero),
            possession: self
                .frame_stats_or_default_typed::<PossessionStats>(frame, "possession")?
                .for_team(is_team_zero),
            ball_half: self
                .frame_stats_or_default_typed::<BallHalfStats>(frame, "ball_half")?
                .for_team(is_team_zero),
            ball_third: self
                .frame_stats_or_default_typed::<BallThirdStats>(frame, "ball_third")?
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
            controlled_play: self.frame_team_stat_or_default_typed(
                frame,
                "controlled_play",
                team_key,
            )?,
            air_dribble: self.frame_team_stat_or_default_typed(frame, "air_dribble", team_key)?,
            boost: self.frame_team_stat_or_default_typed(frame, "boost", team_key)?,
            bump: self.frame_team_stat_or_default_typed(frame, "bump", team_key)?,
            half_volley: self.frame_team_stat_or_default_typed(frame, "half_volley", team_key)?,
            movement: self.frame_team_stat_or_default_typed(frame, "movement", team_key)?,
            positioning: self.frame_team_stat_or_default_typed(frame, "positioning", team_key)?,
            powerslide: self.frame_team_stat_or_default_typed(frame, "powerslide", team_key)?,
            demo: self.frame_team_stat_or_default_typed(frame, "demo", team_key)?,
        })
    }

    pub(in crate::collector::stats::playback) fn replay_player_stats(
        &self,
        frame: &StatsSnapshotFrame,
        player: &PlayerInfo,
    ) -> SubtrActorResult<PlayerStatsSnapshot> {
        let player_key = player_info_key(player)?;
        Ok(PlayerStatsSnapshot {
            player_id: player.remote_id.clone(),
            name: player.name.clone(),
            is_team_0: self.is_team_zero_player(player),
            core: self.frame_core_player_stat_or_default_by_key(frame, &player_key)?,
            backboard: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "backboard",
                &player_key,
            )?,
            ceiling_shot: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "ceiling_shot",
                &player_key,
            )?,
            wall_aerial: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "wall_aerial",
                &player_key,
            )?,
            wall_aerial_shot: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "wall_aerial_shot",
                &player_key,
            )?,
            double_tap: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "double_tap",
                &player_key,
            )?,
            one_timer: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "one_timer",
                &player_key,
            )?,
            pass: self.frame_player_stat_or_default_typed_by_key(frame, "pass", &player_key)?,
            fifty_fifty: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "fifty_fifty",
                &player_key,
            )?,
            kickoff: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "kickoff",
                &player_key,
            )?,
            speed_flip: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "speed_flip",
                &player_key,
            )?,
            half_flip: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "half_flip",
                &player_key,
            )?,
            wavedash: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "wavedash",
                &player_key,
            )?,
            touch: if frame.modules.contains_key("touch") {
                self.frame_player_stat_or_default_with_by_key(frame, "touch", &player_key, || {
                    TouchStats::default().with_complete_labeled_touch_counts()
                })?
            } else {
                self.frame_player_stat_or_default_typed_by_key(frame, "touch", &player_key)?
            },
            whiff: self.frame_player_stat_or_default_typed_by_key(frame, "whiff", &player_key)?,
            flick: self.frame_player_stat_or_default_typed_by_key(frame, "flick", &player_key)?,
            musty_flick: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "musty_flick",
                &player_key,
            )?,
            dodge_reset: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "dodge_reset",
                &player_key,
            )?,
            ball_carry: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "ball_carry",
                &player_key,
            )?,
            controlled_play: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "controlled_play",
                &player_key,
            )?,
            air_dribble: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "air_dribble",
                &player_key,
            )?,
            boost: self.frame_player_stat_or_default_typed_by_key(frame, "boost", &player_key)?,
            bump: self.frame_player_stat_or_default_typed_by_key(frame, "bump", &player_key)?,
            half_volley: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "half_volley",
                &player_key,
            )?,
            movement: self.frame_player_stat_or_default_with_by_key(
                frame,
                "movement",
                &player_key,
                || MovementStats::default().with_complete_labeled_tracked_time(),
            )?,
            positioning: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "positioning",
                &player_key,
            )?,
            rotation: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "rotation",
                &player_key,
            )?,
            powerslide: self.frame_player_stat_or_default_typed_by_key(
                frame,
                "powerslide",
                &player_key,
            )?,
            demo: self.frame_player_stat_or_default_typed_by_key(frame, "demo", &player_key)?,
        })
    }

    pub(in crate::collector::stats::playback) fn is_team_zero_player(
        &self,
        player: &PlayerInfo,
    ) -> bool {
        self.replay_meta
            .team_zero
            .iter()
            .any(|team_player| team_player.remote_id == player.remote_id)
    }

    pub(in crate::collector::stats::playback) fn timeline_team_value(
        &self,
        frame: &StatsSnapshotFrame,
        team_key: &str,
    ) -> SubtrActorResult<Value> {
        let is_team_zero = team_key == "team_zero";
        let mut team = Map::new();
        team.insert(
            "fifty_fifty".to_owned(),
            serialize_to_json_value(
                &self
                    .frame_stats_or_default_typed::<FiftyFiftyStats>(frame, "fifty_fifty")?
                    .for_team(is_team_zero),
            )?,
        );
        team.insert(
            "kickoff".to_owned(),
            serialize_to_json_value(
                &self
                    .frame_stats_or_default_typed::<KickoffStats>(frame, "kickoff")?
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
            "ball_half".to_owned(),
            serialize_to_json_value(
                &self
                    .frame_stats_or_default_typed::<BallHalfStats>(frame, "ball_half")?
                    .for_team(is_team_zero),
            )?,
        );
        team.insert(
            "ball_third".to_owned(),
            serialize_to_json_value(
                &self
                    .frame_stats_or_default_typed::<BallThirdStats>(frame, "ball_third")?
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
            "controlled_play".to_owned(),
            self.frame_team_stat_or_default::<ControlledPlayStats>(
                frame,
                "controlled_play",
                team_key,
            ),
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
            "positioning".to_owned(),
            self.frame_team_stat_or_default::<PositioningTeamStats>(frame, "positioning", team_key),
        );
        team.insert(
            "powerslide".to_owned(),
            self.frame_team_stat_or_default::<PowerslideStats>(frame, "powerslide", team_key),
        );
        team.insert(
            "demo".to_owned(),
            self.frame_team_stat_or_default::<DemoTeamStats>(frame, "demo", team_key),
        );
        Ok(Value::Object(team))
    }

    pub(in crate::collector::stats::playback) fn timeline_player_value(
        &self,
        frame: &StatsSnapshotFrame,
        player: &PlayerInfo,
    ) -> SubtrActorResult<Value> {
        let player_key = player_info_key(player)?;
        let mut player_value = Map::new();
        player_value.insert(
            "player_id".to_owned(),
            serialize_to_json_value(&player.remote_id)?,
        );
        player_value.insert("name".to_owned(), serialize_to_json_value(&player.name)?);
        player_value.insert(
            "is_team_0".to_owned(),
            serialize_to_json_value(
                &self
                    .replay_meta
                    .team_zero
                    .iter()
                    .any(|team_player| team_player.remote_id == player.remote_id),
            )?,
        );
        player_value.insert(
            "core".to_owned(),
            self.frame_player_stat_or_default_by_key::<CorePlayerStats>(
                frame,
                "core",
                &player_key,
            )?,
        );
        player_value.insert(
            "backboard".to_owned(),
            self.frame_player_stat_or_default_by_key::<BackboardPlayerStats>(
                frame,
                "backboard",
                &player_key,
            )?,
        );
        player_value.insert(
            "ceiling_shot".to_owned(),
            self.frame_player_stat_or_default_by_key::<CeilingShotStats>(
                frame,
                "ceiling_shot",
                &player_key,
            )?,
        );
        player_value.insert(
            "wall_aerial".to_owned(),
            self.frame_player_stat_or_default_by_key::<WallAerialStats>(
                frame,
                "wall_aerial",
                &player_key,
            )?,
        );
        player_value.insert(
            "wall_aerial_shot".to_owned(),
            self.frame_player_stat_or_default_by_key::<WallAerialShotStats>(
                frame,
                "wall_aerial_shot",
                &player_key,
            )?,
        );
        player_value.insert(
            "double_tap".to_owned(),
            self.frame_player_stat_or_default_by_key::<DoubleTapPlayerStats>(
                frame,
                "double_tap",
                &player_key,
            )?,
        );
        player_value.insert(
            "one_timer".to_owned(),
            self.frame_player_stat_or_default_by_key::<OneTimerPlayerStats>(
                frame,
                "one_timer",
                &player_key,
            )?,
        );
        player_value.insert(
            "pass".to_owned(),
            self.frame_player_stat_or_default_by_key::<PassPlayerStats>(
                frame,
                "pass",
                &player_key,
            )?,
        );
        player_value.insert(
            "fifty_fifty".to_owned(),
            self.frame_player_stat_or_default_by_key::<FiftyFiftyPlayerStats>(
                frame,
                "fifty_fifty",
                &player_key,
            )?,
        );
        player_value.insert(
            "kickoff".to_owned(),
            self.frame_player_stat_or_default_by_key::<KickoffPlayerStats>(
                frame,
                "kickoff",
                &player_key,
            )?,
        );
        player_value.insert(
            "speed_flip".to_owned(),
            self.frame_player_stat_or_default_by_key::<SpeedFlipStats>(
                frame,
                "speed_flip",
                &player_key,
            )?,
        );
        player_value.insert(
            "half_flip".to_owned(),
            self.frame_player_stat_or_default_by_key::<HalfFlipStats>(
                frame,
                "half_flip",
                &player_key,
            )?,
        );
        player_value.insert(
            "half_volley".to_owned(),
            self.frame_player_stat_or_default_by_key::<HalfVolleyPlayerStats>(
                frame,
                "half_volley",
                &player_key,
            )?,
        );
        player_value.insert(
            "wavedash".to_owned(),
            self.frame_player_stat_or_default_by_key::<WavedashStats>(
                frame,
                "wavedash",
                &player_key,
            )?,
        );
        player_value.insert(
            "touch".to_owned(),
            self.frame_player_stat_or_value_by_key(
                frame,
                "touch",
                &player_key,
                if frame.modules.contains_key("touch") {
                    serialize_to_json_value(
                        &TouchStats::default().with_complete_labeled_touch_counts(),
                    )?
                } else {
                    default_json_value::<TouchStats>()
                },
            )?,
        );
        player_value.insert(
            "whiff".to_owned(),
            self.frame_player_stat_or_default_by_key::<WhiffStats>(frame, "whiff", &player_key)?,
        );
        player_value.insert(
            "flick".to_owned(),
            self.frame_player_stat_or_default_by_key::<FlickStats>(frame, "flick", &player_key)?,
        );
        player_value.insert(
            "musty_flick".to_owned(),
            self.frame_player_stat_or_default_by_key::<MustyFlickStats>(
                frame,
                "musty_flick",
                &player_key,
            )?,
        );
        player_value.insert(
            "dodge_reset".to_owned(),
            self.frame_player_stat_or_default_by_key::<DodgeResetStats>(
                frame,
                "dodge_reset",
                &player_key,
            )?,
        );
        player_value.insert(
            "ball_carry".to_owned(),
            self.frame_player_stat_or_default_by_key::<BallCarryStats>(
                frame,
                "ball_carry",
                &player_key,
            )?,
        );
        player_value.insert(
            "controlled_play".to_owned(),
            self.frame_player_stat_or_default_by_key::<ControlledPlayStats>(
                frame,
                "controlled_play",
                &player_key,
            )?,
        );
        player_value.insert(
            "air_dribble".to_owned(),
            self.frame_player_stat_or_default_by_key::<AirDribbleStats>(
                frame,
                "air_dribble",
                &player_key,
            )?,
        );
        player_value.insert(
            "boost".to_owned(),
            self.frame_player_stat_or_default_by_key::<BoostStats>(frame, "boost", &player_key)?,
        );
        player_value.insert(
            "bump".to_owned(),
            self.frame_player_stat_or_default_by_key::<BumpPlayerStats>(
                frame,
                "bump",
                &player_key,
            )?,
        );
        player_value.insert(
            "movement".to_owned(),
            self.frame_player_stat_or_value_by_key(
                frame,
                "movement",
                &player_key,
                if frame.modules.contains_key("movement") {
                    serialize_to_json_value(
                        &MovementStats::default().with_complete_labeled_tracked_time(),
                    )?
                } else {
                    default_json_value::<MovementStats>()
                },
            )?,
        );
        player_value.insert(
            "positioning".to_owned(),
            self.frame_player_stat_or_default_by_key::<PositioningStats>(
                frame,
                "positioning",
                &player_key,
            )?,
        );
        player_value.insert(
            "rotation".to_owned(),
            self.frame_player_stat_or_default_by_key::<RotationPlayerStats>(
                frame,
                "rotation",
                &player_key,
            )?,
        );
        player_value.insert(
            "powerslide".to_owned(),
            self.frame_player_stat_or_default_by_key::<PowerslideStats>(
                frame,
                "powerslide",
                &player_key,
            )?,
        );
        player_value.insert(
            "demo".to_owned(),
            self.frame_player_stat_or_default_by_key::<DemoPlayerStats>(
                frame,
                "demo",
                &player_key,
            )?,
        );
        Ok(Value::Object(player_value))
    }

    pub(in crate::collector::stats::playback) fn frame_stats_or_default<T>(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
    ) -> Value
    where
        T: Default + Serialize,
    {
        frame
            .modules
            .get(module_name)
            .and_then(Value::as_object)
            .and_then(|module| module.get("stats"))
            .cloned()
            .unwrap_or_else(|| default_json_value::<T>())
    }

    pub(in crate::collector::stats::playback) fn frame_team_stat_or_default<T>(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
        team_key: &str,
    ) -> Value
    where
        T: Default + Serialize,
    {
        frame
            .modules
            .get(module_name)
            .and_then(Value::as_object)
            .and_then(|module| module.get(team_key))
            .cloned()
            .unwrap_or_else(|| default_json_value::<T>())
    }

    pub(in crate::collector::stats::playback) fn frame_player_stat_or_default_by_key<T>(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
        player_key: &str,
    ) -> SubtrActorResult<Value>
    where
        T: Default + Serialize,
    {
        self.frame_player_stat_or_value_by_key(
            frame,
            module_name,
            player_key,
            default_json_value::<T>(),
        )
    }

    pub(in crate::collector::stats::playback) fn frame_player_stat_or_value_by_key(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
        player_key: &str,
        default_value: Value,
    ) -> SubtrActorResult<Value> {
        Ok(
            player_stats_value_for_key(frame.modules.get(module_name), player_key)?
                .cloned()
                .unwrap_or(default_value),
        )
    }

    pub(in crate::collector::stats::playback) fn frame_stats_or_default_typed<T>(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
    ) -> SubtrActorResult<T>
    where
        T: Default + DeserializeOwned + Serialize,
    {
        decode_json_value(self.frame_stats_or_default::<T>(frame, module_name))
    }

    pub(in crate::collector::stats::playback) fn frame_team_stat_or_default_typed<T>(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
        team_key: &str,
    ) -> SubtrActorResult<T>
    where
        T: Default + DeserializeOwned + Serialize,
    {
        decode_json_value(self.frame_team_stat_or_default::<T>(frame, module_name, team_key))
    }

    pub(in crate::collector::stats::playback) fn frame_player_stat_or_default_typed_by_key<T>(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
        player_key: &str,
    ) -> SubtrActorResult<T>
    where
        T: Default + DeserializeOwned + Serialize,
    {
        self.frame_player_stat_or_default_with_by_key(frame, module_name, player_key, T::default)
    }

    pub(in crate::collector::stats::playback) fn frame_core_player_stat_or_default_by_key(
        &self,
        frame: &StatsSnapshotFrame,
        player_key: &str,
    ) -> SubtrActorResult<CorePlayerStats> {
        decode_core_player_stats_value(self.frame_player_stat_or_value_by_key(
            frame,
            "core",
            player_key,
            default_json_value::<CorePlayerStats>(),
        )?)
    }

    pub(in crate::collector::stats::playback) fn frame_player_stat_or_default_with_by_key<T, F>(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
        player_key: &str,
        default: F,
    ) -> SubtrActorResult<T>
    where
        T: DeserializeOwned + Serialize,
        F: FnOnce() -> T,
    {
        decode_json_value(self.frame_player_stat_or_value_by_key(
            frame,
            module_name,
            player_key,
            serialize_to_json_value(&default())?,
        )?)
    }

    pub(in crate::collector::stats::playback) fn module_typed_array<T>(
        &self,
        module_name: &str,
        field: &str,
    ) -> SubtrActorResult<Vec<T>>
    where
        T: DeserializeOwned,
    {
        decode_json_value(Value::Array(self.module_array(module_name, field)))
    }

    pub(in crate::collector::stats::playback) fn module_player_events<T, F>(
        &self,
        module_name: &str,
        field: &str,
        parse: F,
    ) -> SubtrActorResult<Vec<T>>
    where
        F: Fn(&Value) -> SubtrActorResult<T>,
    {
        self.module_array(module_name, field)
            .iter()
            .map(parse)
            .collect()
    }

    pub(in crate::collector::stats::playback) fn module_array(
        &self,
        module_name: &str,
        field: &str,
    ) -> Vec<Value> {
        self.modules
            .get(module_name)
            .and_then(Value::as_object)
            .and_then(|module| module.get(field))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    }
}
