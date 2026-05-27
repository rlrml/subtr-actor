use super::legacy_types::LegacyRotationProbe;
use super::rotation_interpret::{derive_world_angular_velocity, reinterpret_euler_rotation};

const MAX_PAIR_DT_SECONDS: f32 = 0.2;
const MIN_DISPLACEMENT_SPEED: f32 = 100.0;
const MIN_REPORTED_SPEED: f32 = 100.0;
const MIN_ANGULAR_VELOCITY_SPEED: f32 = 30.0;
const MIN_DERIVED_ORIENTATION_SPEED: f32 = 0.5;

impl LegacyRotationProbe {
    pub(super) fn sample_player(
        &mut self,
        player_id: &subtr_actor::PlayerId,
        time: f32,
        rigid_body: boxcars::RigidBody,
    ) {
        if let Some(linear_velocity) = rigid_body.linear_velocity {
            self.sample_grounded_alignment(rigid_body, linear_velocity);
        }

        if let Some((previous_time, previous_body)) = self.previous_bodies.get(player_id) {
            let dt = time - previous_time;
            if (0.0..=MAX_PAIR_DT_SECONDS).contains(&dt) {
                let displacement = glam::Vec3::new(
                    rigid_body.location.x - previous_body.location.x,
                    rigid_body.location.y - previous_body.location.y,
                    rigid_body.location.z - previous_body.location.z,
                );
                let displacement_speed = displacement.length() / dt;
                if displacement_speed >= MIN_DISPLACEMENT_SPEED {
                    let reported_velocity = previous_body
                        .linear_velocity
                        .or(rigid_body.linear_velocity)
                        .map(|velocity| {
                            glam::Vec3::new(velocity.x, velocity.y, velocity.z).length()
                        });
                    if let Some(reported_speed) = reported_velocity {
                        if reported_speed >= MIN_REPORTED_SPEED {
                            for (scale, accumulator) in &mut self.velocity_accumulators {
                                let ratio = (reported_speed * *scale) / displacement_speed;
                                if ratio.is_finite() && ratio > 0.0 {
                                    accumulator.ratios.push(ratio);
                                }
                            }
                        }
                    }
                }

                if let Some(reported_angular_velocity) = previous_body
                    .angular_velocity
                    .or(rigid_body.angular_velocity)
                {
                    let reported_angular_velocity = glam::Vec3::new(
                        reported_angular_velocity.x,
                        reported_angular_velocity.y,
                        reported_angular_velocity.z,
                    );
                    let reported_angular_speed = reported_angular_velocity.length();
                    if reported_angular_speed >= MIN_ANGULAR_VELOCITY_SPEED {
                        for mode in &self.euler_modes {
                            let previous_rotation =
                                reinterpret_euler_rotation(previous_body.rotation, *mode);
                            let current_rotation =
                                reinterpret_euler_rotation(rigid_body.rotation, *mode);
                            if let Some(derived_angular_velocity) = derive_world_angular_velocity(
                                previous_rotation,
                                current_rotation,
                                dt,
                            ) {
                                if derived_angular_velocity.length()
                                    >= MIN_DERIVED_ORIENTATION_SPEED
                                {
                                    let direction_dot = derived_angular_velocity
                                        .normalize()
                                        .dot(reported_angular_velocity.normalize());
                                    if direction_dot.is_finite() {
                                        self.euler_angular_accumulators
                                            .get_mut(mode)
                                            .unwrap()
                                            .direction_dots
                                            .push(direction_dot);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.previous_bodies
            .insert(player_id.clone(), (time, rigid_body));
    }
}
