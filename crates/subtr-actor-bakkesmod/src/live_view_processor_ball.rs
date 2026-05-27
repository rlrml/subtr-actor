macro_rules! sa_live_processor_ball_methods {
    () => {
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
    };
}
