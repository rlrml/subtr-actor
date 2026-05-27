macro_rules! sa_live_processor_player_stats_methods {
    () => {
        fn get_player_id_from_car_id(
            &self,
            actor_id: &boxcars::ActorId,
        ) -> SubtrActorResult<PlayerId> {
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
    };
}
