use serde_json::Value;

use super::super::super::comparable_types::{ComparablePlayerStats, ComparableReplayStats};
use super::super::super::model::TeamColor;
use super::super::conversion_json::{
    comparable_boost_from_json, comparable_core_from_json, comparable_demo_from_json,
    comparable_movement_from_json, comparable_positioning_from_json,
    comparable_team_demo_from_json,
};

pub(crate) fn build_expected_comparable_stats(expected: &Value) -> ComparableReplayStats {
    let mut comparable = ComparableReplayStats::default();

    for team_color in [TeamColor::Blue, TeamColor::Orange] {
        let Some(team) = expected.get(team_color.team_key()) else {
            continue;
        };

        let team_stats = comparable.team_mut(team_color);
        let team_json_stats = team.get("stats");
        team_stats.core =
            comparable_core_from_json(team_json_stats.and_then(|stats| stats.get("core")));
        team_stats.boost =
            comparable_boost_from_json(team_json_stats.and_then(|stats| stats.get("boost")));
        team_stats.movement =
            comparable_movement_from_json(team_json_stats.and_then(|stats| stats.get("movement")));
        team_stats.demo =
            comparable_team_demo_from_json(team_json_stats.and_then(|stats| stats.get("demo")));

        let Some(players) = team.get("players").and_then(Value::as_array) else {
            continue;
        };

        for player in players {
            let Some(name) = player.get("name").and_then(Value::as_str) else {
                continue;
            };
            let stats = player.get("stats");
            team_stats.players.insert(
                name.to_string(),
                ComparablePlayerStats {
                    core: comparable_core_from_json(stats.and_then(|stats| stats.get("core"))),
                    boost: comparable_boost_from_json(stats.and_then(|stats| stats.get("boost"))),
                    movement: comparable_movement_from_json(
                        stats.and_then(|stats| stats.get("movement")),
                    ),
                    positioning: comparable_positioning_from_json(
                        stats.and_then(|stats| stats.get("positioning")),
                    ),
                    demo: comparable_demo_from_json(stats.and_then(|stats| stats.get("demo"))),
                },
            );
        }
    }

    comparable
}
