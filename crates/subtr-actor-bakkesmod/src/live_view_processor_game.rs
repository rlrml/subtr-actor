macro_rules! sa_live_processor_game_methods {
    () => {
        fn get_replay_meta(&self) -> SubtrActorResult<ReplayMeta> {
            self.replay_meta.cloned().ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta)
            })
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
    };
}
