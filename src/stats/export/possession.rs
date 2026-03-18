use crate::*;

use super::*;

impl StatFieldProvider for PossessionStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::float(
            "possession",
            "time",
            StatUnit::Seconds,
            self.tracked_time,
        ));
        for entry in &self.labeled_time.entries {
            visitor(ExportedStat::float_labeled(
                "possession",
                "time",
                StatUnit::Seconds,
                entry.labels.clone(),
                entry.value,
            ));
        }
        visitor(ExportedStat::float(
            "possession",
            "team_zero_time",
            StatUnit::Seconds,
            self.team_zero_time,
        ));
        visitor(ExportedStat::float(
            "possession",
            "team_one_time",
            StatUnit::Seconds,
            self.team_one_time,
        ));
        visitor(ExportedStat::float(
            "possession",
            "neutral_time",
            StatUnit::Seconds,
            self.neutral_time,
        ));
        visitor(ExportedStat::float(
            "possession",
            "team_zero_pct",
            StatUnit::Percent,
            self.team_zero_pct(),
        ));
        visitor(ExportedStat::float(
            "possession",
            "team_one_pct",
            StatUnit::Percent,
            self.team_one_pct(),
        ));
        visitor(ExportedStat::float(
            "possession",
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
    fn possession_export_includes_labeled_time_stats() {
        let mut stats = PossessionStats {
            tracked_time: 5.0,
            ..Default::default()
        };
        stats
            .labeled_time
            .add([StatLabel::new("possession_state", "team_zero")], 2.5);

        let labeled_stats: Vec<_> = stats
            .stat_fields()
            .into_iter()
            .filter(|stat| {
                stat.descriptor.domain == "possession"
                    && stat.descriptor.name == "time"
                    && stat.descriptor.variant == LABELED_STAT_VARIANT
            })
            .collect();

        assert_eq!(labeled_stats.len(), 1);
        assert_eq!(
            labeled_stats[0].descriptor.labels,
            vec![StatLabel::new("possession_state", "team_zero")]
        );
        assert_eq!(labeled_stats[0].value, StatValue::Float(2.5));
    }
}
