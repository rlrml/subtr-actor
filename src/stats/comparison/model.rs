use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TeamColor {
    Blue,
    Orange,
}

impl TeamColor {
    pub(super) fn team_key(self) -> &'static str {
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
pub(super) enum StatScope {
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
pub(super) enum StatDomain {
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
pub(super) enum StatKey {
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
pub(super) struct ComparisonTarget {
    pub(super) scope: StatScope,
    pub(super) domain: StatDomain,
    pub(super) key: StatKey,
}

impl fmt::Display for ComparisonTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.scope, self.domain, self.key)
    }
}
