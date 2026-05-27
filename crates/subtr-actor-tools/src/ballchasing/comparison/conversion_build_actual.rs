use super::super::super::comparable_types::{ComparablePlayerStats, ComparableReplayStats};
use super::super::super::model::TeamColor;
use super::super::conversion_collect::ComputedComparableStats;
use super::super::conversion_stats::{
    comparable_boost_from_stats, comparable_core_from_player, comparable_core_from_team,
    comparable_demo_from_player, comparable_demo_from_team, comparable_movement_from_stats,
    comparable_positioning_from_stats, sum_present,
};

pub(crate) fn build_actual_comparable_stats(
    stats: &ComputedComparableStats,
) -> ComparableReplayStats {
    let mut comparable = ComparableReplayStats::default();

    for (team_color, players) in [
        (TeamColor::Blue, &stats.replay_meta.team_zero),
        (TeamColor::Orange, &stats.replay_meta.team_one),
    ] {
        let team_stats = comparable.team_mut(team_color);
        team_stats.core = comparable_core_from_team(&match team_color {
            TeamColor::Blue => stats.match_stats.team_zero_stats(),
            TeamColor::Orange => stats.match_stats.team_one_stats(),
        });
        let mut team_boost = comparable_boost_from_stats(match team_color {
            TeamColor::Blue => stats.boost.team_zero_stats(),
            TeamColor::Orange => stats.boost.team_one_stats(),
        });
        team_stats.movement = comparable_movement_from_stats(
            match team_color {
                TeamColor::Blue => stats.movement.team_zero_stats(),
                TeamColor::Orange => stats.movement.team_one_stats(),
            },
            match team_color {
                TeamColor::Blue => stats.powerslide.team_zero_stats(),
                TeamColor::Orange => stats.powerslide.team_one_stats(),
            },
        );
        team_stats.demo = comparable_demo_from_team(match team_color {
            TeamColor::Blue => stats.demo.team_zero_stats(),
            TeamColor::Orange => stats.demo.team_one_stats(),
        });

        let player_match_stats = stats.match_stats.player_stats();
        let player_boost_source_stats = stats.boost.player_stats();
        let player_movement_stats = stats.movement.player_stats();
        let player_powerslide_stats = stats.powerslide.player_stats();
        let player_positioning_stats = stats.positioning.player_stats();
        let player_demo_stats = stats.demo.player_stats();
        let mut player_boost_stats = Vec::new();
        for player in players {
            let player_boost = comparable_boost_from_stats(
                &player_boost_source_stats
                    .get(&player.remote_id)
                    .cloned()
                    .unwrap_or_default(),
            );
            player_boost_stats.push(player_boost.clone());
            let player_stats = ComparablePlayerStats {
                core: comparable_core_from_player(
                    &player_match_stats
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
                boost: player_boost,
                movement: comparable_movement_from_stats(
                    &player_movement_stats
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                    &player_powerslide_stats
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
                positioning: comparable_positioning_from_stats(
                    &player_positioning_stats
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
                demo: comparable_demo_from_player(
                    &player_demo_stats
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
            };
            team_stats.players.insert(player.name.clone(), player_stats);
        }

        team_boost.avg_amount =
            sum_present(player_boost_stats.iter().map(|stats| stats.avg_amount));
        team_boost.bpm = sum_present(player_boost_stats.iter().map(|stats| stats.bpm));
        team_stats.boost = team_boost;
    }

    comparable
}
