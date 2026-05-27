use serde::Serialize;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub(crate) struct ComparableMovementStats {
    pub(crate) avg_speed: Option<f64>,
    pub(crate) total_distance: Option<f64>,
    pub(crate) time_supersonic_speed: Option<f64>,
    pub(crate) time_boost_speed: Option<f64>,
    pub(crate) time_slow_speed: Option<f64>,
    pub(crate) time_ground: Option<f64>,
    pub(crate) time_low_air: Option<f64>,
    pub(crate) time_high_air: Option<f64>,
    pub(crate) time_powerslide: Option<f64>,
    pub(crate) count_powerslide: Option<f64>,
    pub(crate) avg_powerslide_duration: Option<f64>,
    pub(crate) avg_speed_percentage: Option<f64>,
    pub(crate) percent_slow_speed: Option<f64>,
    pub(crate) percent_boost_speed: Option<f64>,
    pub(crate) percent_supersonic_speed: Option<f64>,
    pub(crate) percent_ground: Option<f64>,
    pub(crate) percent_low_air: Option<f64>,
    pub(crate) percent_high_air: Option<f64>,
}
