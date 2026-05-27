use super::super::config::{MatchConfig, StatMatcher};
use super::super::model::{StatDomain, StatScope, TeamColor};
use super::structs::{ComparablePlayerStats, ComparableReplayStats, ComparableTeamStats};

impl ComparableReplayStats {
    pub(crate) fn compare(&self, actual: &Self, matcher: &mut StatMatcher, config: &MatchConfig) {
        for team in [TeamColor::Blue, TeamColor::Orange] {
            self.team(team)
                .compare(team, actual.team(team), matcher, config);
        }
    }
}

impl ComparableTeamStats {
    pub(super) fn compare(
        &self,
        team: TeamColor,
        actual: &Self,
        matcher: &mut StatMatcher,
        config: &MatchConfig,
    ) {
        let team_scope = StatScope::Team(team);
        self.core
            .compare(&team_scope, StatDomain::Core, &actual.core, matcher, config);
        self.boost.compare(
            &team_scope,
            StatDomain::Boost,
            &actual.boost,
            matcher,
            config,
        );
        self.movement.compare(
            &team_scope,
            StatDomain::Movement,
            &actual.movement,
            matcher,
            config,
        );
        self.demo
            .compare_team(&team_scope, &actual.demo, matcher, config);

        for (name, expected_player) in &self.players {
            let scope = StatScope::Player {
                team,
                name: name.clone(),
            };
            let Some(actual_player) = actual.players.get(name) else {
                matcher.missing_player(&scope);
                continue;
            };
            expected_player.compare(&scope, actual_player, matcher, config);
        }
    }
}

impl ComparablePlayerStats {
    pub(super) fn compare(
        &self,
        scope: &StatScope,
        actual: &Self,
        matcher: &mut StatMatcher,
        config: &MatchConfig,
    ) {
        self.core
            .compare(scope, StatDomain::Core, &actual.core, matcher, config);
        self.boost
            .compare(scope, StatDomain::Boost, &actual.boost, matcher, config);
        self.movement.compare(
            scope,
            StatDomain::Movement,
            &actual.movement,
            matcher,
            config,
        );
        self.positioning.compare(
            scope,
            StatDomain::Positioning,
            &actual.positioning,
            matcher,
            config,
        );
        self.demo
            .compare_player(scope, &actual.demo, matcher, config);
    }
}
