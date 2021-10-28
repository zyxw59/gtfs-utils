use std::collections::{btree_map::Entry, BTreeMap};

#[derive(Debug)]
pub struct MultiMap<K, V> {
    pub map: BTreeMap<K, Vec<V>>,
}

impl<K, V> MultiMap<K, V>
where
    K: Ord,
{
    pub fn new() -> Self {
        MultiMap {
            map: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.map.entry(key).or_default().push(value);
    }

    pub fn insert_bulk(&mut self, key: K, mut values: Vec<V>) {
        match self.map.entry(key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().append(&mut values);
            }
            Entry::Vacant(entry) => {
                entry.insert(values);
            }
        }
    }
}

impl<K, V> FromIterator<(K, V)> for MultiMap<K, V>
where
    K: Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut map = MultiMap::new();
        for (k, v) in iter {
            map.insert(k, v);
        }
        map
    }
}
