use super::*;

pub(crate) struct SaFrameEventSlices<'a> {
    pub(crate) touches: &'a [SaTouchEvent],
    pub(crate) dodge_refreshes: &'a [SaDodgeRefreshedEvent],
    pub(crate) boost_pad_events: &'a [SaBoostPadEvent],
    pub(crate) goals: &'a [SaGoalEvent],
    pub(crate) player_stat_events: &'a [SaPlayerStatEvent],
    pub(crate) demolishes: &'a [SaDemolishEvent],
}

pub(crate) struct SaLiveProcessorView<'a> {
    replay_meta: Option<&'a ReplayMeta>,
    frame: &'a SaLiveFrame,
    players: &'a [SaPlayerFrame],
    player_ids: Vec<PlayerId>,
    events: FrameEventsState,
    event_history: &'a SaLiveEventHistory,
}

impl<'a> SaLiveProcessorView<'a> {
    pub(crate) fn new(
        replay_meta: Option<&'a ReplayMeta>,
        frame: &'a SaLiveFrame,
        players: &'a [SaPlayerFrame],
        events: FrameEventsState,
        event_history: &'a SaLiveEventHistory,
    ) -> Self {
        Self {
            replay_meta,
            frame,
            players,
            player_ids: players
                .iter()
                .map(|player| player_id(player.player_index))
                .collect(),
            events,
            event_history,
        }
    }

    fn missing<T>(property: &'static str) -> SubtrActorResult<T> {
        SubtrActorError::new_result(SubtrActorErrorVariant::PropertyNotFoundInState { property })
    }

    pub(crate) fn player_index(player_id: &PlayerId) -> Option<u32> {
        match player_id {
            RemoteId::SplitScreen(index) => Some(*index),
            _ => None,
        }
    }

    fn player(&self, player_id: &PlayerId) -> SubtrActorResult<&SaPlayerFrame> {
        let Some(index) = Self::player_index(player_id) else {
            return Self::missing("live player");
        };
        self.players
            .iter()
            .find(|player| player.player_index == index)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "live player",
                })
            })
    }
}

pub(crate) unsafe fn checked_slice<'a, T>(items: *const T, count: usize) -> Result<&'a [T], ()> {
    if items.is_null() && count != 0 {
        return Err(());
    }
    if count == 0 {
        Ok(&[])
    } else {
        Ok(slice::from_raw_parts(items, count))
    }
}

pub(crate) unsafe fn frame_event_slices(frame: &SaLiveFrame) -> Result<SaFrameEventSlices<'_>, ()> {
    Ok(SaFrameEventSlices {
        touches: checked_slice(frame.touches, frame.touch_count)?,
        dodge_refreshes: checked_slice(frame.dodge_refreshes, frame.dodge_refresh_count)?,
        boost_pad_events: checked_slice(frame.boost_pad_events, frame.boost_pad_event_count)?,
        goals: checked_slice(frame.goals, frame.goal_count)?,
        player_stat_events: checked_slice(frame.player_stat_events, frame.player_stat_event_count)?,
        demolishes: checked_slice(frame.demolishes, frame.demolish_count)?,
    })
}

impl ProcessorView for SaLiveProcessorView<'_> {
    fn get_replay_meta(&self) -> SubtrActorResult<ReplayMeta> {
        self.replay_meta
            .cloned()
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))
    }

    fn player_count(&self) -> usize {
        self.players.len()
    }

    fn iter_player_ids_in_order(&self) -> Box<dyn Iterator<Item = &PlayerId> + '_> {
        Box::new(self.player_ids.iter())
    }

    fn current_in_game_team_player_counts(&self) -> [usize; 2] {
        let mut counts = [0, 0];
        for player in self.players {
            counts[usize::from(player.is_team_0 == 0)] += 1;
        }
        counts
    }

    fn get_seconds_remaining(&self) -> SubtrActorResult<i32> {
        (self.frame.has_seconds_remaining != 0)
            .then_some(self.frame.seconds_remaining)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "seconds_remaining",
                })
            })
    }

    fn get_replicated_state_name(&self) -> SubtrActorResult<i32> {
        (self.frame.has_game_state != 0)
            .then_some(self.frame.game_state)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "game_state",
                })
            })
    }

    fn get_replicated_game_state_time_remaining(&self) -> SubtrActorResult<i32> {
        (self.frame.has_kickoff_countdown_time != 0)
            .then_some(self.frame.kickoff_countdown_time)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "kickoff_countdown_time",
                })
            })
    }

    fn get_ball_has_been_hit(&self) -> SubtrActorResult<bool> {
        (self.frame.has_ball_has_been_hit != 0)
            .then_some(self.frame.ball_has_been_hit != 0)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "ball_has_been_hit",
                })
            })
    }

    fn get_ignore_ball_syncing(&self) -> SubtrActorResult<bool> {
        Ok(false)
    }

    fn get_team_scores(&self) -> SubtrActorResult<(i32, i32)> {
        if self.frame.has_team_zero_score != 0 && self.frame.has_team_one_score != 0 {
            Ok((self.frame.team_zero_score, self.frame.team_one_score))
        } else {
            Self::missing("team_scores")
        }
    }

    fn get_ball_hit_team_num(&self) -> SubtrActorResult<u8> {
        (self.frame.has_possession_team != 0)
            .then_some(if self.frame.possession_team_is_team_0 != 0 {
                0
            } else {
                1
            })
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "possession_team",
                })
            })
    }

    fn get_scored_on_team_num(&self) -> SubtrActorResult<u8> {
        (self.frame.has_scored_on_team != 0)
            .then_some(if self.frame.scored_on_team_is_team_0 != 0 {
                0
            } else {
                1
            })
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "scored_on_team",
                })
            })
    }

    fn get_normalized_ball_rigid_body(&self) -> SubtrActorResult<RigidBody> {
        (self.frame.has_ball != 0)
            .then(|| rigid_body(self.frame.ball))
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "ball",
                })
            })
    }

    fn get_velocity_applied_ball_rigid_body(
        &self,
        target_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        let rigid_body = self.get_normalized_ball_rigid_body()?;
        Ok(apply_velocities_to_rigid_body(
            &rigid_body,
            target_time - self.frame.time,
        ))
    }

    fn get_interpolated_ball_rigid_body(
        &self,
        target_time: f32,
        close_enough_to_frame_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        let rigid_body = self.get_normalized_ball_rigid_body()?;
        if (target_time - self.frame.time).abs() <= close_enough_to_frame_time.abs() {
            return Ok(rigid_body);
        }
        Ok(apply_velocities_to_rigid_body(
            &rigid_body,
            target_time - self.frame.time,
        ))
    }

    fn get_normalized_player_rigid_body(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<RigidBody> {
        let player = self.player(player_id)?;
        (player.has_rigid_body != 0)
            .then(|| rigid_body(player.rigid_body))
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "player rigid body",
                })
            })
    }

    fn get_velocity_applied_player_rigid_body(
        &self,
        player_id: &PlayerId,
        target_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        let rigid_body = self.get_normalized_player_rigid_body(player_id)?;
        Ok(apply_velocities_to_rigid_body(
            &rigid_body,
            target_time - self.frame.time,
        ))
    }

    fn get_interpolated_player_rigid_body(
        &self,
        player_id: &PlayerId,
        target_time: f32,
        close_enough_to_frame_time: f32,
    ) -> SubtrActorResult<RigidBody> {
        let rigid_body = self.get_normalized_player_rigid_body(player_id)?;
        if (target_time - self.frame.time).abs() <= close_enough_to_frame_time.abs() {
            return Ok(rigid_body);
        }
        Ok(apply_velocities_to_rigid_body(
            &rigid_body,
            target_time - self.frame.time,
        ))
    }

    fn get_player_name(&self, player_id: &PlayerId) -> SubtrActorResult<String> {
        let player = self.player(player_id)?;
        player_name(player).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                property: "player name",
            })
        })
    }

    fn get_player_team_key(&self, player_id: &PlayerId) -> SubtrActorResult<String> {
        Ok(if self.get_player_is_team_0(player_id)? {
            "0".to_owned()
        } else {
            "1".to_owned()
        })
    }

    fn get_player_is_team_0(&self, player_id: &PlayerId) -> SubtrActorResult<bool> {
        Ok(self.player(player_id)?.is_team_0 != 0)
    }

    fn get_player_id_from_car_id(&self, actor_id: &boxcars::ActorId) -> SubtrActorResult<PlayerId> {
        let Some(index) = u32::try_from(actor_id.0).ok() else {
            return Err(SubtrActorError::new(
                SubtrActorErrorVariant::NoMatchingPlayerId {
                    actor_id: *actor_id,
                },
            ));
        };
        self.players
            .iter()
            .find(|player| player.player_index == index)
            .map(|player| player_id(player.player_index))
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::NoMatchingPlayerId {
                    actor_id: *actor_id,
                })
            })
    }

    fn get_player_boost_level(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        Ok(self.player(player_id)?.boost_amount)
    }

    fn get_player_last_boost_level(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        Ok(self.player(player_id)?.last_boost_amount)
    }

    fn get_player_boost_percentage(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        self.get_player_boost_level(player_id)
            .map(boost_amount_to_percent)
    }

    fn get_boost_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        Ok(self.player(player_id)?.boost_active)
    }

    fn get_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        Ok(self.player(player_id)?.jump_active)
    }

    fn get_double_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        Ok(self.player(player_id)?.double_jump_active)
    }

    fn get_dodge_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        Ok(self.player(player_id)?.dodge_active)
    }

    fn get_powerslide_active(&self, player_id: &PlayerId) -> SubtrActorResult<bool> {
        Ok(self.player(player_id)?.powerslide_active != 0)
    }

    fn get_player_match_assists(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        let player = self.player(player_id)?;
        (player.has_match_stats != 0)
            .then_some(player.match_assists)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "match assists",
                })
            })
    }

    fn get_player_match_goals(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        let player = self.player(player_id)?;
        (player.has_match_stats != 0)
            .then_some(player.match_goals)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "match goals",
                })
            })
    }

    fn get_player_match_saves(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        let player = self.player(player_id)?;
        (player.has_match_stats != 0)
            .then_some(player.match_saves)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "match saves",
                })
            })
    }

    fn get_player_match_score(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        let player = self.player(player_id)?;
        (player.has_match_stats != 0)
            .then_some(player.match_score)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "match score",
                })
            })
    }

    fn get_player_match_shots(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        let player = self.player(player_id)?;
        (player.has_match_stats != 0)
            .then_some(player.match_shots)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "match shots",
                })
            })
    }

    fn get_active_demos(&self) -> SubtrActorResult<Vec<DemolishAttribute>> {
        let mut seen = HashSet::new();
        let mut demos = Vec::new();
        for sample in &self.events.active_demos {
            if !seen.insert((sample.attacker.clone(), sample.victim.clone())) {
                continue;
            }
            let demolish = self.events.demo_events.iter().find(|demolish| {
                demolish.attacker == sample.attacker && demolish.victim == sample.victim
            });
            demos.push(live_demolish_attribute(
                &sample.attacker,
                &sample.victim,
                demolish,
            )?);
        }
        Ok(demos)
    }

    fn demolishes(&self) -> &[DemolishInfo] {
        &self.event_history.demo_events
    }

    fn boost_pad_events(&self) -> &[BoostPadEvent] {
        &self.event_history.boost_pad_events
    }

    fn touch_events(&self) -> &[TouchEvent] {
        &self.event_history.touch_events
    }

    fn dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent] {
        &self.event_history.dodge_refreshed_events
    }

    fn player_stat_events(&self) -> &[PlayerStatEvent] {
        &self.event_history.player_stat_events
    }

    fn goal_events(&self) -> &[GoalEvent] {
        &self.event_history.goal_events
    }

    fn current_frame_active_demo_events(&self) -> &[DemoEventSample] {
        &self.events.active_demos
    }

    fn current_frame_demolish_events(&self) -> &[DemolishInfo] {
        &self.events.demo_events
    }

    fn current_frame_boost_pad_events(&self) -> &[BoostPadEvent] {
        &self.events.boost_pad_events
    }

    fn current_frame_touch_events(&self) -> &[TouchEvent] {
        &self.events.touch_events
    }

    fn current_frame_dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent] {
        &self.events.dodge_refreshed_events
    }

    fn current_frame_player_stat_events(&self) -> &[PlayerStatEvent] {
        &self.events.player_stat_events
    }

    fn current_frame_goal_events(&self) -> &[GoalEvent] {
        &self.events.goal_events
    }
}

pub(crate) fn find_counter(counters: &[(RemoteId, i32)], player_id: &RemoteId) -> Option<i32> {
    counters
        .iter()
        .find_map(|(id, value)| (id == player_id).then_some(*value))
}

pub(crate) fn set_counter(counters: &mut Vec<(RemoteId, i32)>, player_id: RemoteId, value: i32) {
    if let Some((_, counter)) = counters.iter_mut().find(|(id, _)| id == &player_id) {
        *counter = value;
    } else {
        counters.push((player_id, value));
    }
}
