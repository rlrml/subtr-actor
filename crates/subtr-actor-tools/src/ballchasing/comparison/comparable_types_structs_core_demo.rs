use serde::Serialize;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub(crate) struct ComparableCoreStats {
    pub(crate) score: Option<f64>,
    pub(crate) goals: Option<f64>,
    pub(crate) assists: Option<f64>,
    pub(crate) saves: Option<f64>,
    pub(crate) shots: Option<f64>,
    pub(crate) shooting_percentage: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub(crate) struct ComparableDemoStats {
    pub(crate) inflicted: Option<f64>,
    pub(crate) taken: Option<f64>,
}
