pub(crate) trait VecMapEntry<K: PartialEq, V> {
    fn get_entry(&mut self, key: K) -> Entry<'_, K, V>;
}

pub(crate) enum Entry<'a, K: PartialEq, V> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K: PartialEq, V> Entry<'a, K, V> {
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(occupied) => &mut occupied.entry.1,
            Entry::Vacant(vacant) => {
                vacant.vec.push((vacant.key, default()));
                &mut vacant.vec.last_mut().unwrap().1
            }
        }
    }
}

pub(crate) struct OccupiedEntry<'a, K: PartialEq, V> {
    entry: &'a mut (K, V),
}

pub(crate) struct VacantEntry<'a, K: PartialEq, V> {
    vec: &'a mut Vec<(K, V)>,
    key: K,
}

impl<K: PartialEq + Clone, V> VecMapEntry<K, V> for Vec<(K, V)> {
    fn get_entry(&mut self, key: K) -> Entry<'_, K, V> {
        match self.iter_mut().position(|(k, _)| k == &key) {
            Some(index) => Entry::Occupied(OccupiedEntry {
                entry: &mut self[index],
            }),
            None => Entry::Vacant(VacantEntry { vec: self, key }),
        }
    }
}
