use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PossessionCalculator {
    pub(super) stats: PossessionStats,
    pub(super) tracker: PossessionTracker,
    pub(super) events: Vec<PossessionEvent>,
    pub(super) last_emitted_event_state: Option<PossessionEventState>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PossessionEventState {
    pub(super) active: bool,
    pub(super) possession_state: PossessionStateLabel,
    pub(super) field_third: Option<FieldThirdLabel>,
}

impl PossessionCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &PossessionStats {
        &self.stats
    }

    pub fn events(&self) -> &[PossessionEvent] {
        &self.events
    }
}
