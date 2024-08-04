use dashmap::DashMap;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

pub(crate) trait MapWriter<K, V> {
    fn insert(&mut self, k: K, v: V) -> Option<V>
    where
        K: Hash + Eq;

    fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq;
}

impl<K: Hash + Eq, V> MapWriter<K, V> for HashMap<K, V> {
    fn insert(&mut self, k: K, v: V) -> Option<V> {
        HashMap::<K, V>::insert(self, k, v)
    }

    fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        HashMap::<K, V>::remove(self, k)
    }
}

impl<K: Hash + Eq, V> MapWriter<K, V> for Arc<DashMap<K, V>> {
    fn insert(&mut self, k: K, v: V) -> Option<V> {
        DashMap::<K, V>::insert(self, k, v)
    }

    fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        DashMap::<K, V>::remove(self, k).map(|(_, v)| v)
    }
}
