use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct TeamFirstManTracker {
    pub(crate) stable_first_man: Option<PlayerId>,
    pending_first_man: Option<PlayerId>,
    pending_seconds: f32,
}

impl TeamFirstManTracker {
    pub(crate) fn reset(&mut self) {
        self.stable_first_man = None;
        self.pending_first_man = None;
        self.pending_seconds = 0.0;
    }

    pub(crate) fn update(
        &mut self,
        raw_first_man: Option<&PlayerId>,
        dt: f32,
        debounce_seconds: f32,
    ) -> Option<(PlayerId, PlayerId)> {
        let Some(raw_first_man) = raw_first_man else {
            self.pending_first_man = None;
            self.pending_seconds = 0.0;
            return None;
        };

        match self.stable_first_man.as_ref() {
            None => {
                self.stable_first_man = Some(raw_first_man.clone());
                self.pending_first_man = None;
                self.pending_seconds = 0.0;
                None
            }
            Some(stable_first_man) if stable_first_man == raw_first_man => {
                self.pending_first_man = None;
                self.pending_seconds = 0.0;
                None
            }
            Some(stable_first_man) => self.track_pending_change(
                raw_first_man,
                dt,
                debounce_seconds,
                stable_first_man.clone(),
            ),
        }
    }

    fn track_pending_change(
        &mut self,
        raw_first_man: &PlayerId,
        dt: f32,
        debounce_seconds: f32,
        stable_first_man: PlayerId,
    ) -> Option<(PlayerId, PlayerId)> {
        if self.pending_first_man.as_ref() == Some(raw_first_man) {
            self.pending_seconds += dt;
        } else {
            self.pending_first_man = Some(raw_first_man.clone());
            self.pending_seconds = dt;
        }

        if self.pending_seconds < debounce_seconds {
            return None;
        }

        let previous = stable_first_man;
        let next = raw_first_man.clone();
        self.stable_first_man = Some(next.clone());
        self.pending_first_man = None;
        self.pending_seconds = 0.0;
        Some((previous, next))
    }
}
