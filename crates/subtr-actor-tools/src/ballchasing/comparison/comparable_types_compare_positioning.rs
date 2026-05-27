use super::super::config::{MatchConfig, StatMatcher};
use super::super::model::{ComparisonTarget, StatDomain, StatKey, StatScope};
use super::structs::ComparablePositioningStats;

impl ComparablePositioningStats {
    pub(super) fn compare(
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
