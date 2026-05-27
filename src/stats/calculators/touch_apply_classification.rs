use super::*;

impl TouchCalculator {
    pub(crate) fn apply_touch_classification(
        stats: &mut TouchStats,
        classification: TouchClassification,
    ) {
        match classification.height_band {
            PlayerVerticalBand::Ground => {}
            PlayerVerticalBand::LowAir => stats.aerial_touch_count += 1,
            PlayerVerticalBand::HighAir => {
                stats.aerial_touch_count += 1;
                stats.high_aerial_touch_count += 1;
            }
        }

        match classification.kind {
            TouchKind::Control => stats.control_touch_count += 1,
            TouchKind::MediumHit => stats.medium_hit_count += 1,
            TouchKind::HardHit => stats.hard_hit_count += 1,
        }

        if classification.surface == TouchSurface::Wall {
            stats.wall_touch_count += 1;
        }

        stats
            .labeled_touch_counts
            .increment(classification.labels());
    }
}
