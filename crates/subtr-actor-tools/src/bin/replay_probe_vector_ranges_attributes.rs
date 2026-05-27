use std::collections::BTreeMap;

use super::stats::VectorRangeStats;

pub(super) fn record_attribute_vectors(
    ranges: &mut BTreeMap<&'static str, VectorRangeStats>,
    attribute: &boxcars::Attribute,
) {
    match attribute {
        boxcars::Attribute::AppliedDamage(damage) => {
            add_vector(ranges, "AppliedDamage.position", damage.position);
        }
        boxcars::Attribute::DamageState(state) => {
            add_vector(ranges, "DamageState.ball_position", state.ball_position);
        }
        boxcars::Attribute::Demolish(demo) => {
            add_vector(ranges, "Demolish.attack_velocity", demo.attack_velocity);
            add_vector(ranges, "Demolish.victim_velocity", demo.victim_velocity);
        }
        boxcars::Attribute::DemolishExtended(demo) => {
            add_vector(
                ranges,
                "DemolishExtended.attacker_velocity",
                demo.attacker_velocity,
            );
            add_vector(
                ranges,
                "DemolishExtended.victim_velocity",
                demo.victim_velocity,
            );
        }
        boxcars::Attribute::DemolishFx(demo) => {
            add_vector(ranges, "DemolishFx.attack_velocity", demo.attack_velocity);
            add_vector(ranges, "DemolishFx.victim_velocity", demo.victim_velocity);
        }
        boxcars::Attribute::Explosion(explosion) => {
            add_vector(ranges, "Explosion.location", explosion.location);
        }
        boxcars::Attribute::ExtendedExplosion(explosion) => {
            add_vector(
                ranges,
                "ExtendedExplosion.explosion.location",
                explosion.explosion.location,
            );
        }
        boxcars::Attribute::Location(location) => {
            add_vector(ranges, "Attribute::Location", *location);
        }
        boxcars::Attribute::Welded(welded) => {
            add_vector(ranges, "Welded.offset", welded.offset);
        }
        boxcars::Attribute::RigidBody(rigid_body) => {
            add_vector(ranges, "RigidBody.location", rigid_body.location);
            if let Some(linear_velocity) = rigid_body.linear_velocity {
                add_vector(ranges, "RigidBody.linear_velocity", linear_velocity);
            }
            if let Some(angular_velocity) = rigid_body.angular_velocity {
                add_vector(ranges, "RigidBody.angular_velocity", angular_velocity);
            }
        }
        _ => {}
    }
}

pub(super) fn add_vector3i(
    ranges: &mut BTreeMap<&'static str, VectorRangeStats>,
    field: &'static str,
    vector: boxcars::Vector3i,
) {
    add_vector(
        ranges,
        field,
        boxcars::Vector3f {
            x: vector.x as f32,
            y: vector.y as f32,
            z: vector.z as f32,
        },
    );
}

fn add_vector(
    ranges: &mut BTreeMap<&'static str, VectorRangeStats>,
    field: &'static str,
    vector: boxcars::Vector3f,
) {
    ranges.entry(field).or_default().add(vector);
}
