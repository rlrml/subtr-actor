use crate::*;

use super::*;

impl StatFieldProvider for MovementStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::float(
            "movement",
            "tracked_time",
            StatUnit::Seconds,
            self.tracked_time,
        ));
        for entry in self.complete_labeled_tracked_time().entries {
            visitor(ExportedStat::float_labeled(
                "movement",
                "tracked_time",
                StatUnit::Seconds,
                entry.labels,
                entry.value,
            ));
        }
        visitor(ExportedStat::float(
            "movement",
            "total_distance",
            StatUnit::UnrealUnits,
            self.total_distance,
        ));
        visitor(ExportedStat::float(
            "movement",
            "avg_speed",
            StatUnit::UnrealUnitsPerSecond,
            self.average_speed(),
        ));
        visitor(ExportedStat::float(
            "movement",
            "time_supersonic_speed",
            StatUnit::Seconds,
            self.time_supersonic_speed,
        ));
        visitor(ExportedStat::float(
            "movement",
            "time_boost_speed",
            StatUnit::Seconds,
            self.time_boost_speed,
        ));
        visitor(ExportedStat::float(
            "movement",
            "time_slow_speed",
            StatUnit::Seconds,
            self.time_slow_speed,
        ));
        visitor(ExportedStat::float(
            "movement",
            "time_ground",
            StatUnit::Seconds,
            self.time_on_ground,
        ));
        visitor(ExportedStat::float(
            "movement",
            "time_low_air",
            StatUnit::Seconds,
            self.time_low_air,
        ));
        visitor(ExportedStat::float(
            "movement",
            "time_high_air",
            StatUnit::Seconds,
            self.time_high_air,
        ));
        visitor(ExportedStat::float(
            "movement",
            "avg_speed_percentage",
            StatUnit::Percent,
            self.average_speed_pct(),
        ));
        visitor(ExportedStat::float(
            "movement",
            "percent_slow_speed",
            StatUnit::Percent,
            self.slow_speed_pct(),
        ));
        visitor(ExportedStat::float(
            "movement",
            "percent_boost_speed",
            StatUnit::Percent,
            self.boost_speed_pct(),
        ));
        visitor(ExportedStat::float(
            "movement",
            "percent_supersonic_speed",
            StatUnit::Percent,
            self.supersonic_speed_pct(),
        ));
        visitor(ExportedStat::float(
            "movement",
            "percent_ground",
            StatUnit::Percent,
            self.on_ground_pct(),
        ));
        visitor(ExportedStat::float(
            "movement",
            "percent_low_air",
            StatUnit::Percent,
            self.low_air_pct(),
        ));
        visitor(ExportedStat::float(
            "movement",
            "percent_high_air",
            StatUnit::Percent,
            self.high_air_pct(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_export_includes_labeled_tracked_time_stats() {
        let mut stats = MovementStats::default();
        stats.tracked_time = 3.0;
        stats.labeled_tracked_time.add(
            [
                StatLabel::new("speed_band", "boost"),
                StatLabel::new("height_band", "low_air"),
            ],
            1.25,
        );

        let labeled_stats: Vec<_> = stats
            .stat_fields()
            .into_iter()
            .filter(|stat| {
                stat.descriptor.domain == "movement"
                    && stat.descriptor.name == "tracked_time"
                    && stat.descriptor.variant == LABELED_STAT_VARIANT
            })
            .collect();

        assert_eq!(labeled_stats.len(), 9);
        assert_eq!(
            labeled_stats
                .iter()
                .find(|stat| {
                    stat.descriptor.labels
                        == vec![
                            StatLabel::new("height_band", "low_air"),
                            StatLabel::new("speed_band", "boost"),
                        ]
                })
                .unwrap()
                .descriptor
                .labels,
            vec![
                StatLabel::new("height_band", "low_air"),
                StatLabel::new("speed_band", "boost"),
            ]
        );
        assert_eq!(
            labeled_stats
                .iter()
                .find(|stat| {
                    stat.descriptor.labels
                        == vec![
                            StatLabel::new("height_band", "low_air"),
                            StatLabel::new("speed_band", "boost"),
                        ]
                })
                .unwrap()
                .value,
            StatValue::Float(1.25)
        );
        assert_eq!(
            labeled_stats
                .iter()
                .find(|stat| {
                    stat.descriptor.labels
                        == vec![
                            StatLabel::new("height_band", "ground"),
                            StatLabel::new("speed_band", "slow"),
                        ]
                })
                .unwrap()
                .value,
            StatValue::Float(0.0)
        );
    }
}
