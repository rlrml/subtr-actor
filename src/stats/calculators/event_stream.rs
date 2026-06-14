use std::ops::{Deref, DerefMut};

use serde::{Serialize, Serializer};

/// Append-only buffer of emitted events exposing all events and newly added ones.
#[derive(Debug, Clone, PartialEq)]
pub struct EventStream<E> {
    events: Vec<E>,
    update_start: usize,
}

impl<E> Default for EventStream<E> {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            update_start: 0,
        }
    }
}

impl<E> EventStream<E> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_vec(events: Vec<E>) -> Self {
        Self {
            update_start: 0,
            events,
        }
    }

    pub fn begin_update(&mut self) {
        self.update_start = self.events.len();
    }

    pub fn push(&mut self, event: E) {
        self.events.push(event);
    }

    pub fn extend(&mut self, events: impl IntoIterator<Item = E>) {
        self.events.extend(events);
    }

    pub fn replace_all_assuming_append_only(&mut self, events: Vec<E>) {
        self.update_start = self.events.len().min(events.len());
        self.events = events;
    }

    pub fn all(&self) -> &[E] {
        &self.events
    }

    pub fn new_events(&self) -> &[E] {
        &self.events[self.update_start..]
    }

    pub fn into_vec(self) -> Vec<E> {
        self.events
    }
}

impl<E> From<Vec<E>> for EventStream<E> {
    fn from(events: Vec<E>) -> Self {
        Self::from_vec(events)
    }
}

impl<E: Serialize> Serialize for EventStream<E> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.events.serialize(serializer)
    }
}

impl<'a, E> IntoIterator for &'a EventStream<E> {
    type Item = &'a E;
    type IntoIter = std::slice::Iter<'a, E>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.iter()
    }
}

impl<E> Deref for EventStream<E> {
    type Target = [E];

    fn deref(&self) -> &Self::Target {
        self.all()
    }
}

impl<E> DerefMut for EventStream<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_events_tracks_events_after_update_start() {
        let mut stream = EventStream::new();
        stream.push(1);
        stream.push(2);
        assert_eq!(stream.all(), &[1, 2]);
        assert_eq!(stream.new_events(), &[1, 2]);

        stream.begin_update();
        assert!(stream.new_events().is_empty());
        stream.push(3);
        stream.extend([4, 5]);
        assert_eq!(stream.all(), &[1, 2, 3, 4, 5]);
        assert_eq!(stream.new_events(), &[3, 4, 5]);
    }

    #[test]
    fn append_only_replacement_exposes_suffix_after_previous_len() {
        let mut stream = EventStream::from_vec(vec![1, 2]);
        stream.replace_all_assuming_append_only(vec![1, 2, 3, 4]);
        assert_eq!(stream.all(), &[1, 2, 3, 4]);
        assert_eq!(stream.new_events(), &[3, 4]);
    }
}
