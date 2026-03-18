use crate::*;

use super::*;

impl StatFieldProvider for TouchStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "touch",
            "touch_count",
            StatUnit::Count,
            self.touch_count,
        ));
        visitor(ExportedStat::unsigned(
            "touch",
            "dribble_touch_count",
            StatUnit::Count,
            self.dribble_touch_count,
        ));
        visitor(ExportedStat::unsigned(
            "touch",
            "control_touch_count",
            StatUnit::Count,
            self.control_touch_count,
        ));
        visitor(ExportedStat::unsigned(
            "touch",
            "medium_hit_count",
            StatUnit::Count,
            self.medium_hit_count,
        ));
        visitor(ExportedStat::unsigned(
            "touch",
            "hard_hit_count",
            StatUnit::Count,
            self.hard_hit_count,
        ));
        visitor(ExportedStat::unsigned(
            "touch",
            "aerial_touch_count",
            StatUnit::Count,
            self.aerial_touch_count,
        ));
        visitor(ExportedStat::unsigned(
            "touch",
            "high_aerial_touch_count",
            StatUnit::Count,
            self.high_aerial_touch_count,
        ));
        for entry in self.complete_labeled_touch_counts().entries {
            visitor(ExportedStat::unsigned_labeled(
                "touch",
                "touch_count",
                StatUnit::Count,
                entry.labels,
                entry.count,
            ));
        }
        visitor(ExportedStat::unsigned(
            "touch",
            "is_last_touch",
            StatUnit::Count,
            u32::from(self.is_last_touch),
        ));
        if let Some(value) = self.last_touch_time {
            visitor(ExportedStat::float(
                "touch",
                "last_touch_time",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.last_touch_frame {
            visitor(ExportedStat::unsigned(
                "touch",
                "last_touch_frame",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.time_since_last_touch {
            visitor(ExportedStat::float(
                "touch",
                "time_since_last_touch",
                StatUnit::Seconds,
                value,
            ));
        }
        if let Some(value) = self.frames_since_last_touch {
            visitor(ExportedStat::unsigned(
                "touch",
                "frames_since_last_touch",
                StatUnit::Count,
                u32::try_from(value).unwrap_or(u32::MAX),
            ));
        }
        if let Some(value) = self.last_ball_speed_change {
            visitor(ExportedStat::float(
                "touch",
                "last_ball_speed_change",
                StatUnit::UnrealUnitsPerSecond,
                value,
            ));
        }
        visitor(ExportedStat::float(
            "touch",
            "average_ball_speed_change",
            StatUnit::UnrealUnitsPerSecond,
            self.average_ball_speed_change(),
        ));
        visitor(ExportedStat::float(
            "touch",
            "max_ball_speed_change",
            StatUnit::UnrealUnitsPerSecond,
            self.max_ball_speed_change,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_export_includes_labeled_touch_count_stats() {
        let mut stats = TouchStats {
            touch_count: 2,
            ..Default::default()
        };
        stats.labeled_touch_counts.increment([
            StatLabel::new("kind", "hard_hit"),
            StatLabel::new("height_band", "high_air"),
        ]);
        stats.labeled_touch_counts.increment([
            StatLabel::new("kind", "hard_hit"),
            StatLabel::new("height_band", "high_air"),
        ]);

        let labeled_touch_stats: Vec<_> = stats
            .stat_fields()
            .into_iter()
            .filter(|stat| {
                stat.descriptor.name == "touch_count"
                    && stat.descriptor.variant == LABELED_STAT_VARIANT
            })
            .collect();

        assert_eq!(labeled_touch_stats.len(), 12);
        assert_eq!(
            labeled_touch_stats
                .iter()
                .find(|stat| {
                    stat.descriptor.labels
                        == vec![
                            StatLabel::new("height_band", "high_air"),
                            StatLabel::new("kind", "hard_hit"),
                        ]
                })
                .unwrap()
                .descriptor
                .labels,
            vec![
                StatLabel::new("height_band", "high_air"),
                StatLabel::new("kind", "hard_hit"),
            ]
        );
        assert_eq!(
            labeled_touch_stats
                .iter()
                .find(|stat| {
                    stat.descriptor.labels
                        == vec![
                            StatLabel::new("height_band", "high_air"),
                            StatLabel::new("kind", "hard_hit"),
                        ]
                })
                .unwrap()
                .value,
            StatValue::Unsigned(2)
        );
        assert_eq!(
            labeled_touch_stats
                .iter()
                .find(|stat| {
                    stat.descriptor.labels
                        == vec![
                            StatLabel::new("height_band", "ground"),
                            StatLabel::new("kind", "dribble"),
                        ]
                })
                .unwrap()
                .value,
            StatValue::Unsigned(0)
        );
    }
}
