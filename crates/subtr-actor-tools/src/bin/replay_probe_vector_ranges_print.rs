use std::collections::BTreeMap;

use super::super::replay::parse_replay;
use super::attributes::{add_vector3i, record_attribute_vectors};
use super::stats::VectorRangeStats;

pub(crate) fn print_vector_ranges(path: &str) {
    let replay = parse_replay(path);
    println!(
        "replay={path} major_version={} minor_version={} net_version={:?}",
        replay.major_version, replay.minor_version, replay.net_version
    );

    let mut ranges = BTreeMap::<&'static str, VectorRangeStats>::new();
    if let Some(network_frames) = &replay.network_frames {
        for frame in &network_frames.frames {
            for actor in &frame.new_actors {
                if let Some(location) = actor.initial_trajectory.location {
                    add_vector3i(
                        &mut ranges,
                        "NewActor.initial_trajectory.location",
                        location,
                    );
                }
            }

            for update in &frame.updated_actors {
                record_attribute_vectors(&mut ranges, &update.attribute);
            }
        }
    }

    println!(
        "{:<44} {:>8} {:>10} {:>10} {:>10} {:>10} {:>19} {:>19} {:>19}",
        "field",
        "count",
        "axis_max",
        "mag_p50",
        "mag_p95",
        "mag_max",
        "x_range",
        "y_range",
        "z_range"
    );
    for (field, stats) in &mut ranges {
        if let Some(summary) = stats.summary() {
            println!(
                "{:<44} {:>8} {:>10.2} {:>10.2} {:>10.2} {:>10.2} {:>9.2}..{:<9.2} {:>9.2}..{:<9.2} {:>9.2}..{:<9.2}",
                field,
                summary.count,
                summary.max_abs_axis,
                summary.median_magnitude,
                summary.p95_magnitude,
                summary.max_magnitude,
                summary.min_x,
                summary.max_x,
                summary.min_y,
                summary.max_y,
                summary.min_z,
                summary.max_z
            );
        }
    }
}
