macro_rules! sa_live_processor_event_methods {
    () => {
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
    };
}
