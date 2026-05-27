macro_rules! sa_live_processor_player_core_methods {
    () => {
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
    };
}
