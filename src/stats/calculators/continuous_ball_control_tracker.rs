use super::*;

#[derive(Debug, Clone)]
pub struct ContinuousBallControlTracker<K> {
    pub(crate) active_sequence: Option<ActiveBallControlSequence<K>>,
    pub(crate) pending_takeoff_touches: HashMap<PlayerId, u32>,
}

impl<K> Default for ContinuousBallControlTracker<K> {
    fn default() -> Self {
        Self {
            active_sequence: None,
            pending_takeoff_touches: HashMap::new(),
        }
    }
}
