#[path = "config_match_config.rs"]
mod match_config;
#[path = "config_matcher.rs"]
mod matcher;
#[path = "config_recommended.rs"]
mod recommended;
#[path = "config_recommended_predicates.rs"]
mod recommended_predicates;

pub use match_config::MatchConfig;
pub(crate) use matcher::StatMatcher;
pub use recommended::recommended_match_config;

#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
