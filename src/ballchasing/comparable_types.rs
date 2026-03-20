use std::collections::BTreeMap;

use super::config::{MatchConfig, StatMatcher};
use super::model::{ComparisonTarget, StatDomain, StatKey, StatScope, TeamColor};

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct ComparableCoreStats {
    pub(super) score: Option<f64>,
    pub(super) goals: Option<f64>,
    pub(super) assists: Option<f64>,
    pub(super) saves: Option<f64>,
    pub(super) shots: Option<f64>,
    pub(super) shooting_percentage: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct ComparableBoostStats {
    pub(super) bpm: Option<f64>,
    pub(super) avg_amount: Option<f64>,
    pub(super) amount_collected: Option<f64>,
    pub(super) amount_stolen: Option<f64>,
    pub(super) amount_collected_big: Option<f64>,
    pub(super) amount_stolen_big: Option<f64>,
    pub(super) amount_collected_small: Option<f64>,
    pub(super) amount_stolen_small: Option<f64>,
    pub(super) count_collected_big: Option<f64>,
    pub(super) count_stolen_big: Option<f64>,
    pub(super) count_collected_small: Option<f64>,
    pub(super) count_stolen_small: Option<f64>,
    pub(super) amount_overfill: Option<f64>,
    pub(super) amount_overfill_stolen: Option<f64>,
    pub(super) amount_used_while_supersonic: Option<f64>,
    pub(super) time_zero_boost: Option<f64>,
    pub(super) percent_zero_boost: Option<f64>,
    pub(super) time_full_boost: Option<f64>,
    pub(super) percent_full_boost: Option<f64>,
    pub(super) time_boost_0_25: Option<f64>,
    pub(super) time_boost_25_50: Option<f64>,
    pub(super) time_boost_50_75: Option<f64>,
    pub(super) time_boost_75_100: Option<f64>,
    pub(super) percent_boost_0_25: Option<f64>,
    pub(super) percent_boost_25_50: Option<f64>,
    pub(super) percent_boost_50_75: Option<f64>,
    pub(super) percent_boost_75_100: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct ComparableMovementStats {
    pub(super) avg_speed: Option<f64>,
    pub(super) total_distance: Option<f64>,
    pub(super) time_supersonic_speed: Option<f64>,
    pub(super) time_boost_speed: Option<f64>,
    pub(super) time_slow_speed: Option<f64>,
    pub(super) time_ground: Option<f64>,
    pub(super) time_low_air: Option<f64>,
    pub(super) time_high_air: Option<f64>,
    pub(super) time_powerslide: Option<f64>,
    pub(super) count_powerslide: Option<f64>,
    pub(super) avg_powerslide_duration: Option<f64>,
    pub(super) avg_speed_percentage: Option<f64>,
    pub(super) percent_slow_speed: Option<f64>,
    pub(super) percent_boost_speed: Option<f64>,
    pub(super) percent_supersonic_speed: Option<f64>,
    pub(super) percent_ground: Option<f64>,
    pub(super) percent_low_air: Option<f64>,
    pub(super) percent_high_air: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct ComparablePositioningStats {
    pub(super) avg_distance_to_ball: Option<f64>,
    pub(super) avg_distance_to_ball_possession: Option<f64>,
    pub(super) avg_distance_to_ball_no_possession: Option<f64>,
    pub(super) avg_distance_to_mates: Option<f64>,
    pub(super) time_defensive_third: Option<f64>,
    pub(super) time_neutral_third: Option<f64>,
    pub(super) time_offensive_third: Option<f64>,
    pub(super) time_defensive_half: Option<f64>,
    pub(super) time_offensive_half: Option<f64>,
    pub(super) time_behind_ball: Option<f64>,
    pub(super) time_infront_ball: Option<f64>,
    pub(super) time_most_back: Option<f64>,
    pub(super) time_most_forward: Option<f64>,
    pub(super) time_closest_to_ball: Option<f64>,
    pub(super) time_farthest_from_ball: Option<f64>,
    pub(super) percent_defensive_third: Option<f64>,
    pub(super) percent_neutral_third: Option<f64>,
    pub(super) percent_offensive_third: Option<f64>,
    pub(super) percent_defensive_half: Option<f64>,
    pub(super) percent_offensive_half: Option<f64>,
    pub(super) percent_behind_ball: Option<f64>,
    pub(super) percent_infront_ball: Option<f64>,
    pub(super) percent_most_back: Option<f64>,
    pub(super) percent_most_forward: Option<f64>,
    pub(super) percent_closest_to_ball: Option<f64>,
    pub(super) percent_farthest_from_ball: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct ComparableDemoStats {
    pub(super) inflicted: Option<f64>,
    pub(super) taken: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct ComparablePlayerStats {
    pub(super) core: ComparableCoreStats,
    pub(super) boost: ComparableBoostStats,
    pub(super) movement: ComparableMovementStats,
    pub(super) positioning: ComparablePositioningStats,
    pub(super) demo: ComparableDemoStats,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct ComparableTeamStats {
    pub(super) core: ComparableCoreStats,
    pub(super) boost: ComparableBoostStats,
    pub(super) movement: ComparableMovementStats,
    pub(super) demo: ComparableDemoStats,
    pub(super) players: BTreeMap<String, ComparablePlayerStats>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct ComparableReplayStats {
    pub(super) blue: ComparableTeamStats,
    pub(super) orange: ComparableTeamStats,
}

impl ComparableReplayStats {
    pub(super) fn team(&self, color: TeamColor) -> &ComparableTeamStats {
        match color {
            TeamColor::Blue => &self.blue,
            TeamColor::Orange => &self.orange,
        }
    }

    pub(super) fn team_mut(&mut self, color: TeamColor) -> &mut ComparableTeamStats {
        match color {
            TeamColor::Blue => &mut self.blue,
            TeamColor::Orange => &mut self.orange,
        }
    }

    pub(super) fn compare(&self, actual: &Self, matcher: &mut StatMatcher, config: &MatchConfig) {
        for team in [TeamColor::Blue, TeamColor::Orange] {
            self.team(team)
                .compare(team, actual.team(team), matcher, config);
        }
    }
}

impl ComparableTeamStats {
    fn compare(
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
    fn compare(
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

impl ComparableCoreStats {
    fn compare(
        &self,
        scope: &StatScope,
        domain: StatDomain,
        actual: &Self,
        matcher: &mut StatMatcher,
        config: &MatchConfig,
    ) {
        matcher.compare_field(
            actual.score,
            self.score,
            ComparisonTarget {
                scope: scope.clone(),
                domain,
                key: StatKey::Score,
            },
            config,
        );
        matcher.compare_field(
            actual.goals,
            self.goals,
            ComparisonTarget {
                scope: scope.clone(),
                domain,
                key: StatKey::Goals,
            },
            config,
        );
        matcher.compare_field(
            actual.assists,
            self.assists,
            ComparisonTarget {
                scope: scope.clone(),
                domain,
                key: StatKey::Assists,
            },
            config,
        );
        matcher.compare_field(
            actual.saves,
            self.saves,
            ComparisonTarget {
                scope: scope.clone(),
                domain,
                key: StatKey::Saves,
            },
            config,
        );
        matcher.compare_field(
            actual.shots,
            self.shots,
            ComparisonTarget {
                scope: scope.clone(),
                domain,
                key: StatKey::Shots,
            },
            config,
        );
        matcher.compare_field(
            actual.shooting_percentage,
            self.shooting_percentage,
            ComparisonTarget {
                scope: scope.clone(),
                domain,
                key: StatKey::ShootingPercentage,
            },
            config,
        );
    }
}

impl ComparableBoostStats {
    fn compare(
        &self,
        scope: &StatScope,
        domain: StatDomain,
        actual: &Self,
        matcher: &mut StatMatcher,
        config: &MatchConfig,
    ) {
        macro_rules! compare {
            ($field:ident, $key:ident) => {
                matcher.compare_field(
                    actual.$field,
                    self.$field,
                    ComparisonTarget {
                        scope: scope.clone(),
                        domain,
                        key: StatKey::$key,
                    },
                    config,
                );
            };
        }

        compare!(bpm, Bpm);
        compare!(avg_amount, AvgAmount);
        compare!(amount_collected, AmountCollected);
        compare!(amount_stolen, AmountStolen);
        compare!(amount_collected_big, AmountCollectedBig);
        compare!(amount_stolen_big, AmountStolenBig);
        compare!(amount_collected_small, AmountCollectedSmall);
        compare!(amount_stolen_small, AmountStolenSmall);
        compare!(count_collected_big, CountCollectedBig);
        compare!(count_stolen_big, CountStolenBig);
        compare!(count_collected_small, CountCollectedSmall);
        compare!(count_stolen_small, CountStolenSmall);
        compare!(amount_overfill, AmountOverfill);
        compare!(amount_overfill_stolen, AmountOverfillStolen);
        compare!(amount_used_while_supersonic, AmountUsedWhileSupersonic);
        compare!(time_zero_boost, TimeZeroBoost);
        compare!(percent_zero_boost, PercentZeroBoost);
        compare!(time_full_boost, TimeFullBoost);
        compare!(percent_full_boost, PercentFullBoost);
        compare!(time_boost_0_25, TimeBoost0To25);
        compare!(time_boost_25_50, TimeBoost25To50);
        compare!(time_boost_50_75, TimeBoost50To75);
        compare!(time_boost_75_100, TimeBoost75To100);
        compare!(percent_boost_0_25, PercentBoost0To25);
        compare!(percent_boost_25_50, PercentBoost25To50);
        compare!(percent_boost_50_75, PercentBoost50To75);
        compare!(percent_boost_75_100, PercentBoost75To100);
    }
}

impl ComparableMovementStats {
    fn compare(
        &self,
        scope: &StatScope,
        domain: StatDomain,
        actual: &Self,
        matcher: &mut StatMatcher,
        config: &MatchConfig,
    ) {
        macro_rules! compare {
            ($field:ident, $key:ident) => {
                matcher.compare_field(
                    actual.$field,
                    self.$field,
                    ComparisonTarget {
                        scope: scope.clone(),
                        domain,
                        key: StatKey::$key,
                    },
                    config,
                );
            };
        }

        compare!(avg_speed, AvgSpeed);
        compare!(total_distance, TotalDistance);
        compare!(time_supersonic_speed, TimeSupersonicSpeed);
        compare!(time_boost_speed, TimeBoostSpeed);
        compare!(time_slow_speed, TimeSlowSpeed);
        compare!(time_ground, TimeGround);
        compare!(time_low_air, TimeLowAir);
        compare!(time_high_air, TimeHighAir);
        compare!(time_powerslide, TimePowerslide);
        compare!(count_powerslide, CountPowerslide);
        compare!(avg_powerslide_duration, AvgPowerslideDuration);
        compare!(avg_speed_percentage, AvgSpeedPercentage);
        compare!(percent_slow_speed, PercentSlowSpeed);
        compare!(percent_boost_speed, PercentBoostSpeed);
        compare!(percent_supersonic_speed, PercentSupersonicSpeed);
        compare!(percent_ground, PercentGround);
        compare!(percent_low_air, PercentLowAir);
        compare!(percent_high_air, PercentHighAir);
    }
}

impl ComparablePositioningStats {
    fn compare(
        &self,
        scope: &StatScope,
        domain: StatDomain,
        actual: &Self,
        matcher: &mut StatMatcher,
        config: &MatchConfig,
    ) {
        macro_rules! compare {
            ($field:ident, $key:ident) => {
                matcher.compare_field(
                    actual.$field,
                    self.$field,
                    ComparisonTarget {
                        scope: scope.clone(),
                        domain,
                        key: StatKey::$key,
                    },
                    config,
                );
            };
        }

        compare!(avg_distance_to_ball, AvgDistanceToBall);
        compare!(avg_distance_to_ball_possession, AvgDistanceToBallPossession);
        compare!(
            avg_distance_to_ball_no_possession,
            AvgDistanceToBallNoPossession
        );
        compare!(avg_distance_to_mates, AvgDistanceToMates);
        compare!(time_defensive_third, TimeDefensiveThird);
        compare!(time_neutral_third, TimeNeutralThird);
        compare!(time_offensive_third, TimeOffensiveThird);
        compare!(time_defensive_half, TimeDefensiveHalf);
        compare!(time_offensive_half, TimeOffensiveHalf);
        compare!(time_behind_ball, TimeBehindBall);
        compare!(time_infront_ball, TimeInfrontBall);
        compare!(time_most_back, TimeMostBack);
        compare!(time_most_forward, TimeMostForward);
        compare!(time_closest_to_ball, TimeClosestToBall);
        compare!(time_farthest_from_ball, TimeFarthestFromBall);
        compare!(percent_defensive_third, PercentDefensiveThird);
        compare!(percent_neutral_third, PercentNeutralThird);
        compare!(percent_offensive_third, PercentOffensiveThird);
        compare!(percent_defensive_half, PercentDefensiveHalf);
        compare!(percent_offensive_half, PercentOffensiveHalf);
        compare!(percent_behind_ball, PercentBehindBall);
        compare!(percent_infront_ball, PercentInfrontBall);
        compare!(percent_most_back, PercentMostBack);
        compare!(percent_most_forward, PercentMostForward);
        compare!(percent_closest_to_ball, PercentClosestToBall);
        compare!(percent_farthest_from_ball, PercentFarthestFromBall);
    }
}

impl ComparableDemoStats {
    fn compare_player(
        &self,
        scope: &StatScope,
        actual: &Self,
        matcher: &mut StatMatcher,
        config: &MatchConfig,
    ) {
        matcher.compare_field(
            actual.inflicted,
            self.inflicted,
            ComparisonTarget {
                scope: scope.clone(),
                domain: StatDomain::Demo,
                key: StatKey::DemoInflicted,
            },
            config,
        );
        matcher.compare_field(
            actual.taken,
            self.taken,
            ComparisonTarget {
                scope: scope.clone(),
                domain: StatDomain::Demo,
                key: StatKey::DemoTaken,
            },
            config,
        );
    }

    fn compare_team(
        &self,
        scope: &StatScope,
        actual: &Self,
        matcher: &mut StatMatcher,
        config: &MatchConfig,
    ) {
        matcher.compare_field(
            actual.inflicted,
            self.inflicted,
            ComparisonTarget {
                scope: scope.clone(),
                domain: StatDomain::Demo,
                key: StatKey::DemoInflicted,
            },
            config,
        );
    }
}
