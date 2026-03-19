use crate::*;

use super::*;

impl StatFieldProvider for PressureStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::float(
            "pressure",
            "time",
            StatUnit::Seconds,
            self.tracked_time,
        ));
        for entry in &self.labeled_time.entries {
            visitor(ExportedStat::float_labeled(
                "pressure",
                "time",
                StatUnit::Seconds,
                entry.labels.clone(),
                entry.value,
            ));
        }
        visitor(ExportedStat::float(
            "pressure",
            "team_zero_side_time",
            StatUnit::Seconds,
            self.team_zero_side_time,
        ));
        visitor(ExportedStat::float(
            "pressure",
            "team_one_side_time",
            StatUnit::Seconds,
            self.team_one_side_time,
        ));
        visitor(ExportedStat::float(
            "pressure",
            "neutral_time",
            StatUnit::Seconds,
            self.neutral_time,
        ));
        visitor(ExportedStat::float(
            "pressure",
            "team_zero_side_pct",
            StatUnit::Percent,
            self.team_zero_side_pct(),
        ));
        visitor(ExportedStat::float(
            "pressure",
            "team_one_side_pct",
            StatUnit::Percent,
            self.team_one_side_pct(),
        ));
        visitor(ExportedStat::float(
            "pressure",
            "neutral_pct",
            StatUnit::Percent,
            self.neutral_pct(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressure_export_includes_labeled_time_stats() {
        let mut stats = PressureStats {
            tracked_time: 4.0,
            ..Default::default()
        };
        stats
            .labeled_time
            .add([StatLabel::new("field_half", "team_zero_side")], 1.5);

        let labeled_stats: Vec<_> = stats
            .stat_fields()
            .into_iter()
            .filter(|stat| {
                stat.descriptor.domain == "pressure"
                    && stat.descriptor.name == "time"
                    && stat.descriptor.variant == LABELED_STAT_VARIANT
            })
            .collect();

        assert_eq!(labeled_stats.len(), 1);
        assert_eq!(
            labeled_stats[0].descriptor.labels,
            vec![StatLabel::new("field_half", "team_zero_side")]
        );
        assert_eq!(labeled_stats[0].value, StatValue::Float(1.5));
    }
}
