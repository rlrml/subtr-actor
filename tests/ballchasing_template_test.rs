use std::collections::BTreeMap;
use std::fmt;

use serde_json::Value;
use subtr_actor::*;

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {path}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TeamColor {
    Blue,
    Orange,
}

impl TeamColor {
    fn ballchasing_key(self) -> &'static str {
        match self {
            Self::Blue => "blue",
            Self::Orange => "orange",
        }
    }
}

impl fmt::Display for TeamColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blue => write!(f, "blue"),
            Self::Orange => write!(f, "orange"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StatScope {
    Team(TeamColor),
    Player { team: TeamColor, name: String },
}

impl fmt::Display for StatScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Team(team) => write!(f, "team.{team}"),
            Self::Player { team, name } => write!(f, "player.{team}.{name}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatDomain {
    Core,
    Boost,
    Movement,
    Positioning,
    Demo,
}

impl fmt::Display for StatDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core => write!(f, "core"),
            Self::Boost => write!(f, "boost"),
            Self::Movement => write!(f, "movement"),
            Self::Positioning => write!(f, "positioning"),
            Self::Demo => write!(f, "demo"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatKey {
    Score,
    Goals,
    Assists,
    Saves,
    Shots,
    ShootingPercentage,
    Bpm,
    AvgAmount,
    AmountCollected,
    AmountStolen,
    AmountCollectedBig,
    AmountStolenBig,
    AmountCollectedSmall,
    AmountStolenSmall,
    CountCollectedBig,
    CountStolenBig,
    CountCollectedSmall,
    CountStolenSmall,
    AmountOverfill,
    AmountOverfillStolen,
    AmountUsedWhileSupersonic,
    TimeZeroBoost,
    PercentZeroBoost,
    TimeFullBoost,
    PercentFullBoost,
    TimeBoost0To25,
    TimeBoost25To50,
    TimeBoost50To75,
    TimeBoost75To100,
    PercentBoost0To25,
    PercentBoost25To50,
    PercentBoost50To75,
    PercentBoost75To100,
    AvgSpeed,
    TotalDistance,
    TimeSupersonicSpeed,
    TimeBoostSpeed,
    TimeSlowSpeed,
    TimeGround,
    TimeLowAir,
    TimeHighAir,
    TimePowerslide,
    CountPowerslide,
    AvgPowerslideDuration,
    AvgSpeedPercentage,
    PercentSlowSpeed,
    PercentBoostSpeed,
    PercentSupersonicSpeed,
    PercentGround,
    PercentLowAir,
    PercentHighAir,
    AvgDistanceToBall,
    AvgDistanceToBallPossession,
    AvgDistanceToBallNoPossession,
    AvgDistanceToMates,
    TimeDefensiveThird,
    TimeNeutralThird,
    TimeOffensiveThird,
    TimeDefensiveHalf,
    TimeOffensiveHalf,
    TimeBehindBall,
    TimeInfrontBall,
    TimeMostBack,
    TimeMostForward,
    TimeClosestToBall,
    TimeFarthestFromBall,
    PercentDefensiveThird,
    PercentNeutralThird,
    PercentOffensiveThird,
    PercentDefensiveHalf,
    PercentOffensiveHalf,
    PercentBehindBall,
    PercentInfrontBall,
    PercentMostBack,
    PercentMostForward,
    PercentClosestToBall,
    PercentFarthestFromBall,
    DemoInflicted,
    DemoTaken,
}

impl fmt::Display for StatKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Score => "score",
            Self::Goals => "goals",
            Self::Assists => "assists",
            Self::Saves => "saves",
            Self::Shots => "shots",
            Self::ShootingPercentage => "shooting_percentage",
            Self::Bpm => "bpm",
            Self::AvgAmount => "avg_amount",
            Self::AmountCollected => "amount_collected",
            Self::AmountStolen => "amount_stolen",
            Self::AmountCollectedBig => "amount_collected_big",
            Self::AmountStolenBig => "amount_stolen_big",
            Self::AmountCollectedSmall => "amount_collected_small",
            Self::AmountStolenSmall => "amount_stolen_small",
            Self::CountCollectedBig => "count_collected_big",
            Self::CountStolenBig => "count_stolen_big",
            Self::CountCollectedSmall => "count_collected_small",
            Self::CountStolenSmall => "count_stolen_small",
            Self::AmountOverfill => "amount_overfill",
            Self::AmountOverfillStolen => "amount_overfill_stolen",
            Self::AmountUsedWhileSupersonic => "amount_used_while_supersonic",
            Self::TimeZeroBoost => "time_zero_boost",
            Self::PercentZeroBoost => "percent_zero_boost",
            Self::TimeFullBoost => "time_full_boost",
            Self::PercentFullBoost => "percent_full_boost",
            Self::TimeBoost0To25 => "time_boost_0_25",
            Self::TimeBoost25To50 => "time_boost_25_50",
            Self::TimeBoost50To75 => "time_boost_50_75",
            Self::TimeBoost75To100 => "time_boost_75_100",
            Self::PercentBoost0To25 => "percent_boost_0_25",
            Self::PercentBoost25To50 => "percent_boost_25_50",
            Self::PercentBoost50To75 => "percent_boost_50_75",
            Self::PercentBoost75To100 => "percent_boost_75_100",
            Self::AvgSpeed => "avg_speed",
            Self::TotalDistance => "total_distance",
            Self::TimeSupersonicSpeed => "time_supersonic_speed",
            Self::TimeBoostSpeed => "time_boost_speed",
            Self::TimeSlowSpeed => "time_slow_speed",
            Self::TimeGround => "time_ground",
            Self::TimeLowAir => "time_low_air",
            Self::TimeHighAir => "time_high_air",
            Self::TimePowerslide => "time_powerslide",
            Self::CountPowerslide => "count_powerslide",
            Self::AvgPowerslideDuration => "avg_powerslide_duration",
            Self::AvgSpeedPercentage => "avg_speed_percentage",
            Self::PercentSlowSpeed => "percent_slow_speed",
            Self::PercentBoostSpeed => "percent_boost_speed",
            Self::PercentSupersonicSpeed => "percent_supersonic_speed",
            Self::PercentGround => "percent_ground",
            Self::PercentLowAir => "percent_low_air",
            Self::PercentHighAir => "percent_high_air",
            Self::AvgDistanceToBall => "avg_distance_to_ball",
            Self::AvgDistanceToBallPossession => "avg_distance_to_ball_possession",
            Self::AvgDistanceToBallNoPossession => "avg_distance_to_ball_no_possession",
            Self::AvgDistanceToMates => "avg_distance_to_mates",
            Self::TimeDefensiveThird => "time_defensive_third",
            Self::TimeNeutralThird => "time_neutral_third",
            Self::TimeOffensiveThird => "time_offensive_third",
            Self::TimeDefensiveHalf => "time_defensive_half",
            Self::TimeOffensiveHalf => "time_offensive_half",
            Self::TimeBehindBall => "time_behind_ball",
            Self::TimeInfrontBall => "time_infront_ball",
            Self::TimeMostBack => "time_most_back",
            Self::TimeMostForward => "time_most_forward",
            Self::TimeClosestToBall => "time_closest_to_ball",
            Self::TimeFarthestFromBall => "time_farthest_from_ball",
            Self::PercentDefensiveThird => "percent_defensive_third",
            Self::PercentNeutralThird => "percent_neutral_third",
            Self::PercentOffensiveThird => "percent_offensive_third",
            Self::PercentDefensiveHalf => "percent_defensive_half",
            Self::PercentOffensiveHalf => "percent_offensive_half",
            Self::PercentBehindBall => "percent_behind_ball",
            Self::PercentInfrontBall => "percent_infront_ball",
            Self::PercentMostBack => "percent_most_back",
            Self::PercentMostForward => "percent_most_forward",
            Self::PercentClosestToBall => "percent_closest_to_ball",
            Self::PercentFarthestFromBall => "percent_farthest_from_ball",
            Self::DemoInflicted => "inflicted",
            Self::DemoTaken => "taken",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ComparisonTarget {
    scope: StatScope,
    domain: StatDomain,
    key: StatKey,
}

impl fmt::Display for ComparisonTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.scope, self.domain, self.key)
    }
}

type MatchSelector = dyn Fn(&ComparisonTarget) -> bool;
type MatchPredicate = dyn Fn(f64, f64, &ComparisonTarget) -> bool;

struct MatchRule {
    description: String,
    selector: Box<MatchSelector>,
    predicate: Box<MatchPredicate>,
}

#[derive(Default)]
struct MatchConfig {
    rules: Vec<MatchRule>,
}

struct MatchOutcome<'a> {
    matches: bool,
    description: &'a str,
}

impl MatchConfig {
    fn exact() -> Self {
        Self::default()
    }

    fn with_rule<S, P>(mut self, description: impl Into<String>, selector: S, predicate: P) -> Self
    where
        S: Fn(&ComparisonTarget) -> bool + 'static,
        P: Fn(f64, f64, &ComparisonTarget) -> bool + 'static,
    {
        self.rules.push(MatchRule {
            description: description.into(),
            selector: Box::new(selector),
            predicate: Box::new(predicate),
        });
        self
    }

    fn evaluate<'a>(
        &'a self,
        actual: f64,
        expected: f64,
        target: &ComparisonTarget,
    ) -> MatchOutcome<'a> {
        let default = MatchOutcome {
            matches: actual == expected,
            description: "exact",
        };

        self.rules
            .iter()
            .rev()
            .find(|rule| (rule.selector)(target))
            .map(|rule| MatchOutcome {
                matches: (rule.predicate)(actual, expected, target),
                description: &rule.description,
            })
            .unwrap_or(default)
    }
}

fn approx_abs(abs_tol: f64) -> impl Fn(f64, f64, &ComparisonTarget) -> bool {
    move |actual, expected, _| (actual - expected).abs() <= abs_tol
}

fn recommended_ballchasing_match_config() -> MatchConfig {
    MatchConfig::exact()
        .with_rule(
            "shooting percentage abs<=0.01",
            |target| target.key == StatKey::ShootingPercentage,
            approx_abs(0.01),
        )
        .with_rule(
            "boost amount style fields abs<=2",
            |target| {
                matches!(
                    target.key,
                    StatKey::AmountCollected
                        | StatKey::AmountStolen
                        | StatKey::AmountCollectedBig
                        | StatKey::AmountStolenBig
                        | StatKey::AmountCollectedSmall
                        | StatKey::AmountStolenSmall
                        | StatKey::AmountOverfill
                        | StatKey::AmountOverfillStolen
                        | StatKey::AmountUsedWhileSupersonic
                )
            },
            approx_abs(2.0),
        )
        .with_rule(
            "boost timing and percentage fields abs<=1",
            |target| {
                matches!(
                    target.key,
                    StatKey::Bpm
                        | StatKey::AvgAmount
                        | StatKey::TimeZeroBoost
                        | StatKey::PercentZeroBoost
                        | StatKey::TimeFullBoost
                        | StatKey::PercentFullBoost
                        | StatKey::TimeBoost0To25
                        | StatKey::TimeBoost25To50
                        | StatKey::TimeBoost50To75
                        | StatKey::TimeBoost75To100
                        | StatKey::PercentBoost0To25
                        | StatKey::PercentBoost25To50
                        | StatKey::PercentBoost50To75
                        | StatKey::PercentBoost75To100
                )
            },
            approx_abs(1.0),
        )
        .with_rule(
            "movement timing and percentage fields abs<=1",
            |target| {
                matches!(
                    target.key,
                    StatKey::TimeSupersonicSpeed
                        | StatKey::TimeBoostSpeed
                        | StatKey::TimeSlowSpeed
                        | StatKey::TimeGround
                        | StatKey::TimeLowAir
                        | StatKey::TimeHighAir
                        | StatKey::TimePowerslide
                        | StatKey::PercentSlowSpeed
                        | StatKey::PercentBoostSpeed
                        | StatKey::PercentSupersonicSpeed
                        | StatKey::PercentGround
                        | StatKey::PercentLowAir
                        | StatKey::PercentHighAir
                )
            },
            approx_abs(1.0),
        )
        .with_rule(
            "movement distance/speed fields tolerate Ballchasing rounding",
            |target| {
                matches!(
                    target.key,
                    StatKey::AvgSpeed
                        | StatKey::AvgSpeedPercentage
                        | StatKey::TotalDistance
                        | StatKey::AvgPowerslideDuration
                )
            },
            |actual, expected, target| {
                let tol = match target.key {
                    StatKey::AvgSpeed => 5.0,
                    StatKey::AvgSpeedPercentage => 0.5,
                    StatKey::TotalDistance => 2500.0,
                    StatKey::AvgPowerslideDuration => 0.1,
                    _ => 0.0,
                };
                (actual - expected).abs() <= tol
            },
        )
        .with_rule(
            "positioning fields abs<=1 or 50 depending on metric",
            |target| target.domain == StatDomain::Positioning,
            |actual, expected, target| {
                let tol = match target.key {
                    StatKey::AvgDistanceToBall
                    | StatKey::AvgDistanceToBallPossession
                    | StatKey::AvgDistanceToBallNoPossession
                    | StatKey::AvgDistanceToMates => 50.0,
                    _ => 1.0,
                };
                (actual - expected).abs() <= tol
            },
        )
}

#[derive(Debug, Default)]
struct StatMatcher {
    mismatches: Vec<String>,
}

impl StatMatcher {
    fn compare_field(
        &mut self,
        actual: Option<f64>,
        expected: Option<f64>,
        target: ComparisonTarget,
        config: &MatchConfig,
    ) {
        let Some(expected_value) = expected else {
            return;
        };
        let Some(actual_value) = actual else {
            self.mismatches
                .push(format!("{target}: missing actual value"));
            return;
        };

        let outcome = config.evaluate(actual_value, expected_value, &target);
        if !outcome.matches {
            self.mismatches.push(format!(
                "{target}: actual={actual_value} expected={expected_value} predicate={}",
                outcome.description
            ));
        }
    }

    fn missing_player(&mut self, scope: &StatScope) {
        self.mismatches
            .push(format!("{scope}: missing actual player"));
    }

    fn assert_matches(self) {
        if self.mismatches.is_empty() {
            return;
        }

        panic!(
            "Ballchasing comparison failed:\n{}",
            self.mismatches.join("\n")
        );
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ComparableCoreStats {
    score: Option<f64>,
    goals: Option<f64>,
    assists: Option<f64>,
    saves: Option<f64>,
    shots: Option<f64>,
    shooting_percentage: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ComparableBoostStats {
    bpm: Option<f64>,
    avg_amount: Option<f64>,
    amount_collected: Option<f64>,
    amount_stolen: Option<f64>,
    amount_collected_big: Option<f64>,
    amount_stolen_big: Option<f64>,
    amount_collected_small: Option<f64>,
    amount_stolen_small: Option<f64>,
    count_collected_big: Option<f64>,
    count_stolen_big: Option<f64>,
    count_collected_small: Option<f64>,
    count_stolen_small: Option<f64>,
    amount_overfill: Option<f64>,
    amount_overfill_stolen: Option<f64>,
    amount_used_while_supersonic: Option<f64>,
    time_zero_boost: Option<f64>,
    percent_zero_boost: Option<f64>,
    time_full_boost: Option<f64>,
    percent_full_boost: Option<f64>,
    time_boost_0_25: Option<f64>,
    time_boost_25_50: Option<f64>,
    time_boost_50_75: Option<f64>,
    time_boost_75_100: Option<f64>,
    percent_boost_0_25: Option<f64>,
    percent_boost_25_50: Option<f64>,
    percent_boost_50_75: Option<f64>,
    percent_boost_75_100: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ComparableMovementStats {
    avg_speed: Option<f64>,
    total_distance: Option<f64>,
    time_supersonic_speed: Option<f64>,
    time_boost_speed: Option<f64>,
    time_slow_speed: Option<f64>,
    time_ground: Option<f64>,
    time_low_air: Option<f64>,
    time_high_air: Option<f64>,
    time_powerslide: Option<f64>,
    count_powerslide: Option<f64>,
    avg_powerslide_duration: Option<f64>,
    avg_speed_percentage: Option<f64>,
    percent_slow_speed: Option<f64>,
    percent_boost_speed: Option<f64>,
    percent_supersonic_speed: Option<f64>,
    percent_ground: Option<f64>,
    percent_low_air: Option<f64>,
    percent_high_air: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ComparablePositioningStats {
    avg_distance_to_ball: Option<f64>,
    avg_distance_to_ball_possession: Option<f64>,
    avg_distance_to_ball_no_possession: Option<f64>,
    avg_distance_to_mates: Option<f64>,
    time_defensive_third: Option<f64>,
    time_neutral_third: Option<f64>,
    time_offensive_third: Option<f64>,
    time_defensive_half: Option<f64>,
    time_offensive_half: Option<f64>,
    time_behind_ball: Option<f64>,
    time_infront_ball: Option<f64>,
    time_most_back: Option<f64>,
    time_most_forward: Option<f64>,
    time_closest_to_ball: Option<f64>,
    time_farthest_from_ball: Option<f64>,
    percent_defensive_third: Option<f64>,
    percent_neutral_third: Option<f64>,
    percent_offensive_third: Option<f64>,
    percent_defensive_half: Option<f64>,
    percent_offensive_half: Option<f64>,
    percent_behind_ball: Option<f64>,
    percent_infront_ball: Option<f64>,
    percent_most_back: Option<f64>,
    percent_most_forward: Option<f64>,
    percent_closest_to_ball: Option<f64>,
    percent_farthest_from_ball: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ComparableDemoStats {
    inflicted: Option<f64>,
    taken: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ComparablePlayerStats {
    core: ComparableCoreStats,
    boost: ComparableBoostStats,
    movement: ComparableMovementStats,
    positioning: ComparablePositioningStats,
    demo: ComparableDemoStats,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ComparableTeamStats {
    core: ComparableCoreStats,
    boost: ComparableBoostStats,
    movement: ComparableMovementStats,
    demo: ComparableDemoStats,
    players: BTreeMap<String, ComparablePlayerStats>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ComparableReplayStats {
    blue: ComparableTeamStats,
    orange: ComparableTeamStats,
}

impl ComparableReplayStats {
    fn team(&self, color: TeamColor) -> &ComparableTeamStats {
        match color {
            TeamColor::Blue => &self.blue,
            TeamColor::Orange => &self.orange,
        }
    }

    fn team_mut(&mut self, color: TeamColor) -> &mut ComparableTeamStats {
        match color {
            TeamColor::Blue => &mut self.blue,
            TeamColor::Orange => &mut self.orange,
        }
    }

    fn compare(&self, actual: &Self, matcher: &mut StatMatcher, config: &MatchConfig) {
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

fn json_number(stats: Option<&Value>, field: &str) -> Option<f64> {
    stats
        .and_then(|stats| stats.get(field))
        .and_then(Value::as_f64)
}

fn comparable_core_from_json(stats: Option<&Value>) -> ComparableCoreStats {
    ComparableCoreStats {
        score: json_number(stats, "score"),
        goals: json_number(stats, "goals"),
        assists: json_number(stats, "assists"),
        saves: json_number(stats, "saves"),
        shots: json_number(stats, "shots"),
        shooting_percentage: json_number(stats, "shooting_percentage"),
    }
}

fn comparable_boost_from_json(stats: Option<&Value>) -> ComparableBoostStats {
    ComparableBoostStats {
        bpm: json_number(stats, "bpm"),
        avg_amount: json_number(stats, "avg_amount"),
        amount_collected: json_number(stats, "amount_collected"),
        amount_stolen: json_number(stats, "amount_stolen"),
        amount_collected_big: json_number(stats, "amount_collected_big"),
        amount_stolen_big: json_number(stats, "amount_stolen_big"),
        amount_collected_small: json_number(stats, "amount_collected_small"),
        amount_stolen_small: json_number(stats, "amount_stolen_small"),
        count_collected_big: json_number(stats, "count_collected_big"),
        count_stolen_big: json_number(stats, "count_stolen_big"),
        count_collected_small: json_number(stats, "count_collected_small"),
        count_stolen_small: json_number(stats, "count_stolen_small"),
        amount_overfill: json_number(stats, "amount_overfill"),
        amount_overfill_stolen: json_number(stats, "amount_overfill_stolen"),
        amount_used_while_supersonic: json_number(stats, "amount_used_while_supersonic"),
        time_zero_boost: json_number(stats, "time_zero_boost"),
        percent_zero_boost: json_number(stats, "percent_zero_boost"),
        time_full_boost: json_number(stats, "time_full_boost"),
        percent_full_boost: json_number(stats, "percent_full_boost"),
        time_boost_0_25: json_number(stats, "time_boost_0_25"),
        time_boost_25_50: json_number(stats, "time_boost_25_50"),
        time_boost_50_75: json_number(stats, "time_boost_50_75"),
        time_boost_75_100: json_number(stats, "time_boost_75_100"),
        percent_boost_0_25: json_number(stats, "percent_boost_0_25"),
        percent_boost_25_50: json_number(stats, "percent_boost_25_50"),
        percent_boost_50_75: json_number(stats, "percent_boost_50_75"),
        percent_boost_75_100: json_number(stats, "percent_boost_75_100"),
    }
}

fn comparable_movement_from_json(stats: Option<&Value>) -> ComparableMovementStats {
    ComparableMovementStats {
        avg_speed: json_number(stats, "avg_speed"),
        total_distance: json_number(stats, "total_distance"),
        time_supersonic_speed: json_number(stats, "time_supersonic_speed"),
        time_boost_speed: json_number(stats, "time_boost_speed"),
        time_slow_speed: json_number(stats, "time_slow_speed"),
        time_ground: json_number(stats, "time_ground"),
        time_low_air: json_number(stats, "time_low_air"),
        time_high_air: json_number(stats, "time_high_air"),
        time_powerslide: json_number(stats, "time_powerslide"),
        count_powerslide: json_number(stats, "count_powerslide"),
        avg_powerslide_duration: json_number(stats, "avg_powerslide_duration"),
        avg_speed_percentage: json_number(stats, "avg_speed_percentage"),
        percent_slow_speed: json_number(stats, "percent_slow_speed"),
        percent_boost_speed: json_number(stats, "percent_boost_speed"),
        percent_supersonic_speed: json_number(stats, "percent_supersonic_speed"),
        percent_ground: json_number(stats, "percent_ground"),
        percent_low_air: json_number(stats, "percent_low_air"),
        percent_high_air: json_number(stats, "percent_high_air"),
    }
}

fn comparable_positioning_from_json(stats: Option<&Value>) -> ComparablePositioningStats {
    ComparablePositioningStats {
        avg_distance_to_ball: json_number(stats, "avg_distance_to_ball"),
        avg_distance_to_ball_possession: json_number(stats, "avg_distance_to_ball_possession"),
        avg_distance_to_ball_no_possession: json_number(
            stats,
            "avg_distance_to_ball_no_possession",
        ),
        avg_distance_to_mates: json_number(stats, "avg_distance_to_mates"),
        time_defensive_third: json_number(stats, "time_defensive_third"),
        time_neutral_third: json_number(stats, "time_neutral_third"),
        time_offensive_third: json_number(stats, "time_offensive_third"),
        time_defensive_half: json_number(stats, "time_defensive_half"),
        time_offensive_half: json_number(stats, "time_offensive_half"),
        time_behind_ball: json_number(stats, "time_behind_ball"),
        time_infront_ball: json_number(stats, "time_infront_ball"),
        time_most_back: json_number(stats, "time_most_back"),
        time_most_forward: json_number(stats, "time_most_forward"),
        time_closest_to_ball: json_number(stats, "time_closest_to_ball"),
        time_farthest_from_ball: json_number(stats, "time_farthest_from_ball"),
        percent_defensive_third: json_number(stats, "percent_defensive_third"),
        percent_neutral_third: json_number(stats, "percent_neutral_third"),
        percent_offensive_third: json_number(stats, "percent_offensive_third"),
        percent_defensive_half: json_number(stats, "percent_defensive_half"),
        percent_offensive_half: json_number(stats, "percent_offensive_half"),
        percent_behind_ball: json_number(stats, "percent_behind_ball"),
        percent_infront_ball: json_number(stats, "percent_infront_ball"),
        percent_most_back: json_number(stats, "percent_most_back"),
        percent_most_forward: json_number(stats, "percent_most_forward"),
        percent_closest_to_ball: json_number(stats, "percent_closest_to_ball"),
        percent_farthest_from_ball: json_number(stats, "percent_farthest_from_ball"),
    }
}

fn comparable_demo_from_json(stats: Option<&Value>) -> ComparableDemoStats {
    ComparableDemoStats {
        inflicted: json_number(stats, "inflicted"),
        taken: json_number(stats, "taken"),
    }
}

fn comparable_team_demo_from_json(stats: Option<&Value>) -> ComparableDemoStats {
    ComparableDemoStats {
        inflicted: json_number(stats, "inflicted"),
        taken: None,
    }
}

fn comparable_core_from_player(stats: &CorePlayerStats) -> ComparableCoreStats {
    ComparableCoreStats {
        score: Some(stats.score as f64),
        goals: Some(stats.goals as f64),
        assists: Some(stats.assists as f64),
        saves: Some(stats.saves as f64),
        shots: Some(stats.shots as f64),
        shooting_percentage: Some(stats.shooting_percentage() as f64),
    }
}

fn comparable_core_from_team(stats: &CoreTeamStats) -> ComparableCoreStats {
    ComparableCoreStats {
        score: Some(stats.score as f64),
        goals: Some(stats.goals as f64),
        assists: Some(stats.assists as f64),
        saves: Some(stats.saves as f64),
        shots: Some(stats.shots as f64),
        shooting_percentage: Some(stats.shooting_percentage() as f64),
    }
}

fn comparable_boost_from_stats(stats: &BoostStats) -> ComparableBoostStats {
    ComparableBoostStats {
        bpm: Some(stats.bpm() as f64),
        avg_amount: Some(stats.average_boost_amount() as f64),
        amount_collected: Some(stats.amount_collected as f64),
        amount_stolen: Some(stats.amount_stolen as f64),
        amount_collected_big: Some(stats.amount_collected_big as f64),
        amount_stolen_big: Some(stats.amount_stolen_big as f64),
        amount_collected_small: Some(stats.amount_collected_small as f64),
        amount_stolen_small: Some(stats.amount_stolen_small as f64),
        count_collected_big: Some(stats.big_pads_collected as f64),
        count_stolen_big: Some(stats.big_pads_stolen as f64),
        count_collected_small: Some(stats.small_pads_collected as f64),
        count_stolen_small: Some(stats.small_pads_stolen as f64),
        amount_overfill: Some(stats.overfill_total as f64),
        amount_overfill_stolen: Some(stats.overfill_from_stolen as f64),
        amount_used_while_supersonic: Some(stats.amount_used_while_supersonic as f64),
        time_zero_boost: Some(stats.time_zero_boost as f64),
        percent_zero_boost: Some(stats.zero_boost_pct() as f64),
        time_full_boost: Some(stats.time_hundred_boost as f64),
        percent_full_boost: Some(stats.hundred_boost_pct() as f64),
        time_boost_0_25: Some(stats.time_boost_0_25 as f64),
        time_boost_25_50: Some(stats.time_boost_25_50 as f64),
        time_boost_50_75: Some(stats.time_boost_50_75 as f64),
        time_boost_75_100: Some(stats.time_boost_75_100 as f64),
        percent_boost_0_25: Some(stats.boost_0_25_pct() as f64),
        percent_boost_25_50: Some(stats.boost_25_50_pct() as f64),
        percent_boost_50_75: Some(stats.boost_50_75_pct() as f64),
        percent_boost_75_100: Some(stats.boost_75_100_pct() as f64),
    }
}

fn comparable_movement_from_stats(
    movement: &MovementStats,
    powerslide: &PowerslideStats,
) -> ComparableMovementStats {
    ComparableMovementStats {
        avg_speed: Some(movement.average_speed() as f64),
        total_distance: Some(movement.total_distance as f64),
        time_supersonic_speed: Some(movement.time_supersonic_speed as f64),
        time_boost_speed: Some(movement.time_boost_speed as f64),
        time_slow_speed: Some(movement.time_slow_speed as f64),
        time_ground: Some(movement.time_on_ground as f64),
        time_low_air: Some(movement.time_low_air as f64),
        time_high_air: Some(movement.time_high_air as f64),
        time_powerslide: Some(powerslide.total_duration as f64),
        count_powerslide: Some(powerslide.press_count as f64),
        avg_powerslide_duration: Some(powerslide.average_duration() as f64),
        avg_speed_percentage: Some(movement.average_speed_pct() as f64),
        percent_slow_speed: Some(movement.slow_speed_pct() as f64),
        percent_boost_speed: Some(movement.boost_speed_pct() as f64),
        percent_supersonic_speed: Some(movement.supersonic_speed_pct() as f64),
        percent_ground: Some(movement.on_ground_pct() as f64),
        percent_low_air: Some(movement.low_air_pct() as f64),
        percent_high_air: Some(movement.high_air_pct() as f64),
    }
}

fn comparable_positioning_from_stats(stats: &PositioningStats) -> ComparablePositioningStats {
    ComparablePositioningStats {
        avg_distance_to_ball: Some(stats.average_distance_to_ball() as f64),
        avg_distance_to_ball_possession: Some(
            stats.average_distance_to_ball_has_possession() as f64
        ),
        avg_distance_to_ball_no_possession: Some(
            stats.average_distance_to_ball_no_possession() as f64
        ),
        avg_distance_to_mates: Some(stats.average_distance_to_teammates() as f64),
        time_defensive_third: Some(stats.time_defensive_third as f64),
        time_neutral_third: Some(stats.time_neutral_third as f64),
        time_offensive_third: Some(stats.time_offensive_third as f64),
        time_defensive_half: Some(stats.time_defensive_half as f64),
        time_offensive_half: Some(stats.time_offensive_half as f64),
        time_behind_ball: Some(stats.time_behind_ball as f64),
        time_infront_ball: Some(stats.time_in_front_of_ball as f64),
        time_most_back: Some(stats.time_most_back as f64),
        time_most_forward: Some(stats.time_most_forward as f64),
        time_closest_to_ball: Some(stats.time_closest_to_ball as f64),
        time_farthest_from_ball: Some(stats.time_farthest_from_ball as f64),
        percent_defensive_third: Some(stats.defensive_third_pct() as f64),
        percent_neutral_third: Some(stats.neutral_third_pct() as f64),
        percent_offensive_third: Some(stats.offensive_third_pct() as f64),
        percent_defensive_half: Some(stats.defensive_half_pct() as f64),
        percent_offensive_half: Some(stats.offensive_half_pct() as f64),
        percent_behind_ball: Some(stats.behind_ball_pct() as f64),
        percent_infront_ball: Some(stats.in_front_of_ball_pct() as f64),
        percent_most_back: Some(stats.most_back_pct() as f64),
        percent_most_forward: Some(stats.most_forward_pct() as f64),
        percent_closest_to_ball: Some(stats.closest_to_ball_pct() as f64),
        percent_farthest_from_ball: Some(stats.farthest_from_ball_pct() as f64),
    }
}

fn comparable_demo_from_player(stats: &DemoPlayerStats) -> ComparableDemoStats {
    ComparableDemoStats {
        inflicted: Some(stats.demos_inflicted as f64),
        taken: Some(stats.demos_taken as f64),
    }
}

fn comparable_demo_from_team(stats: &DemoTeamStats) -> ComparableDemoStats {
    ComparableDemoStats {
        inflicted: Some(stats.demos_inflicted as f64),
        taken: None,
    }
}

struct ComputedBallchasingComparableStats {
    replay_meta: ReplayMeta,
    match_stats: MatchStatsReducer,
    boost: BoostReducer,
    movement: MovementReducer,
    positioning: PositioningReducer,
    demo: DemoReducer,
    powerslide: PowerslideReducer,
}

fn compute_ballchasing_comparable_stats(
    replay: &boxcars::Replay,
) -> SubtrActorResult<ComputedBallchasingComparableStats> {
    let mut match_collector = ReducerCollector::new(MatchStatsReducer::new());
    let mut boost_collector = ReducerCollector::new(BoostReducer::new());
    let mut movement_collector = ReducerCollector::new(MovementReducer::new());
    let mut positioning_collector = ReducerCollector::new(PositioningReducer::new());
    let mut demo_collector = ReducerCollector::new(DemoReducer::new());
    let mut powerslide_collector = ReducerCollector::new(PowerslideReducer::new());

    let mut processor = ReplayProcessor::new(replay)?;
    let mut collectors: [&mut dyn Collector; 6] = [
        &mut match_collector,
        &mut boost_collector,
        &mut movement_collector,
        &mut positioning_collector,
        &mut demo_collector,
        &mut powerslide_collector,
    ];
    processor.process_all(&mut collectors)?;

    Ok(ComputedBallchasingComparableStats {
        replay_meta: processor.get_replay_meta()?,
        match_stats: match_collector.into_inner(),
        boost: boost_collector.into_inner(),
        movement: movement_collector.into_inner(),
        positioning: positioning_collector.into_inner(),
        demo: demo_collector.into_inner(),
        powerslide: powerslide_collector.into_inner(),
    })
}

fn build_actual_comparable_stats(
    stats: &ComputedBallchasingComparableStats,
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
        team_stats.boost = comparable_boost_from_stats(match team_color {
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

        for player in players {
            let player_stats = ComparablePlayerStats {
                core: comparable_core_from_player(
                    &stats
                        .match_stats
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
                boost: comparable_boost_from_stats(
                    &stats
                        .boost
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
                movement: comparable_movement_from_stats(
                    &stats
                        .movement
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                    &stats
                        .powerslide
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
                positioning: comparable_positioning_from_stats(
                    &stats
                        .positioning
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
                demo: comparable_demo_from_player(
                    &stats
                        .demo
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
            };
            team_stats.players.insert(player.name.clone(), player_stats);
        }
    }

    comparable
}

fn build_expected_comparable_stats(ballchasing: &Value) -> ComparableReplayStats {
    let mut comparable = ComparableReplayStats::default();

    for team_color in [TeamColor::Blue, TeamColor::Orange] {
        let Some(team) = ballchasing.get(team_color.ballchasing_key()) else {
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

fn compare_replay_against_ballchasing_json(
    replay_path: &str,
    json_path: &str,
    config: &MatchConfig,
) {
    let replay = parse_replay(replay_path);
    let json_file = std::fs::File::open(json_path)
        .unwrap_or_else(|_| panic!("Failed to open ballchasing json: {json_path}"));
    let ballchasing: Value = serde_json::from_reader(json_file)
        .unwrap_or_else(|_| panic!("Failed to parse ballchasing json: {json_path}"));

    let computed =
        compute_ballchasing_comparable_stats(&replay).expect("Failed to compute reducer stats");
    let actual = build_actual_comparable_stats(&computed);
    let expected = build_expected_comparable_stats(&ballchasing);

    let mut matcher = StatMatcher::default();
    expected.compare(&actual, &mut matcher, config);
    matcher.assert_matches();
}

fn compare_fixture_directory(path: &std::path::Path, config: &MatchConfig) {
    let replay_path = path.join("replay.replay");
    let json_path = path.join("ballchasing.json");
    compare_replay_against_ballchasing_json(
        replay_path
            .to_str()
            .expect("Fixture replay path must be valid UTF-8"),
        json_path
            .to_str()
            .expect("Fixture JSON path must be valid UTF-8"),
        config,
    );
}

fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new("assets/ballchasing-fixtures").join(name)
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return (*message).to_string();
    }
    "non-string panic payload".to_string()
}

fn assert_fixture_reaches_stat_comparison(name: &str) {
    let fixture = fixture_path(name);
    let panic_payload = std::panic::catch_unwind(|| {
        compare_fixture_directory(&fixture, &recommended_ballchasing_match_config());
    })
    .expect_err("Fixture comparison should currently fail on stat mismatches");
    let message = panic_message(panic_payload);
    assert!(
        message.contains("Ballchasing comparison failed:"),
        "Expected fixture {name} to reach stat comparison, got panic: {message}"
    );
    assert!(
        !message.contains("Failed to compute reducer stats"),
        "Fixture {name} still failed before stat comparison: {message}"
    );
}

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Expected environment variable {name} to be set"))
}

#[test]
fn test_match_config_defaults_to_exact() {
    let config = MatchConfig::exact().with_rule(
        "time zero boost abs<=1",
        |target| target.key == StatKey::TimeZeroBoost,
        approx_abs(1.0),
    );

    let default_target = ComparisonTarget {
        scope: StatScope::Team(TeamColor::Blue),
        domain: StatDomain::Boost,
        key: StatKey::CountCollectedBig,
    };
    let tolerant_target = ComparisonTarget {
        scope: StatScope::Team(TeamColor::Blue),
        domain: StatDomain::Boost,
        key: StatKey::TimeZeroBoost,
    };

    assert!(!config.evaluate(3.0, 2.0, &default_target).matches);
    assert!(config.evaluate(3.5, 3.0, &tolerant_target).matches);
}

#[test]
fn test_match_config_uses_last_matching_rule() {
    let config = MatchConfig::exact()
        .with_rule(
            "all movement abs<=1",
            |target| target.domain == StatDomain::Movement,
            approx_abs(1.0),
        )
        .with_rule(
            "movement total distance abs<=10",
            |target| target.key == StatKey::TotalDistance,
            approx_abs(10.0),
        );

    let target = ComparisonTarget {
        scope: StatScope::Team(TeamColor::Blue),
        domain: StatDomain::Movement,
        key: StatKey::TotalDistance,
    };

    let outcome = config.evaluate(1008.0, 1000.0, &target);
    assert!(outcome.matches);
    assert_eq!(outcome.description, "movement total distance abs<=10");
}

#[test]
fn test_ranked_doubles_fixture_reaches_stat_comparison() {
    assert_fixture_reaches_stat_comparison("recent-ranked-doubles-2026-03-10");
}

#[test]
fn test_ranked_standard_fixture_a_reaches_stat_comparison() {
    assert_fixture_reaches_stat_comparison("recent-ranked-standard-2026-03-10-a");
}

#[test]
fn test_ranked_standard_fixture_b_reaches_stat_comparison() {
    assert_fixture_reaches_stat_comparison("recent-ranked-standard-2026-03-10-b");
}

#[test]
#[ignore = "Set BALLCHASING_REPLAY_FILE and BALLCHASING_JSON_FILE to compare a local replay/json pair"]
fn template_compare_local_replay_against_ballchasing_json() {
    let replay_path = required_env("BALLCHASING_REPLAY_FILE");
    let json_path = required_env("BALLCHASING_JSON_FILE");

    compare_replay_against_ballchasing_json(
        &replay_path,
        &json_path,
        &recommended_ballchasing_match_config(),
    );
}

#[test]
#[ignore = "Downloaded fixtures still need processor-compatibility verification before running in CI"]
fn compare_downloaded_ballchasing_fixtures() {
    let root = std::path::Path::new("assets/ballchasing-fixtures");
    let mut fixture_dirs: Vec<_> = std::fs::read_dir(root)
        .expect("Failed to read fixture directory")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    fixture_dirs.sort();

    assert!(
        !fixture_dirs.is_empty(),
        "Expected at least one downloaded Ballchasing fixture directory"
    );

    for fixture_dir in fixture_dirs {
        compare_fixture_directory(&fixture_dir, &recommended_ballchasing_match_config());
    }
}
