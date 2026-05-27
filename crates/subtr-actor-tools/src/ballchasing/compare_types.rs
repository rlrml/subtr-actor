use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct BallchasingComparableStats {
    pub actual: Value,
    pub expected: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct BallchasingComparisonBreakdown {
    pub is_match: bool,
    pub mismatches: Vec<String>,
    pub comparable_stats: BallchasingComparableStats,
}
