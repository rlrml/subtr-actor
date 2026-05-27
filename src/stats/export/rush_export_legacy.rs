use crate::{ExportedStat, RushStats, StatUnit};

pub(super) fn visit_legacy_rush_fields(stats: &RushStats, visitor: &mut dyn FnMut(ExportedStat)) {
    for (name, count) in [
        ("team_zero_count", stats.team_zero_count),
        ("team_zero_two_v_one_count", stats.team_zero_two_v_one_count),
        ("team_zero_two_v_two_count", stats.team_zero_two_v_two_count),
        (
            "team_zero_two_v_three_count",
            stats.team_zero_two_v_three_count,
        ),
        (
            "team_zero_three_v_one_count",
            stats.team_zero_three_v_one_count,
        ),
        (
            "team_zero_three_v_two_count",
            stats.team_zero_three_v_two_count,
        ),
        (
            "team_zero_three_v_three_count",
            stats.team_zero_three_v_three_count,
        ),
        ("team_one_count", stats.team_one_count),
        ("team_one_two_v_one_count", stats.team_one_two_v_one_count),
        ("team_one_two_v_two_count", stats.team_one_two_v_two_count),
        (
            "team_one_two_v_three_count",
            stats.team_one_two_v_three_count,
        ),
        (
            "team_one_three_v_one_count",
            stats.team_one_three_v_one_count,
        ),
        (
            "team_one_three_v_two_count",
            stats.team_one_three_v_two_count,
        ),
        (
            "team_one_three_v_three_count",
            stats.team_one_three_v_three_count,
        ),
    ] {
        visitor(ExportedStat::unsigned("rush", name, StatUnit::Count, count));
    }
}
