use crate::collection::map::list_bucket::ListBucket;
use crate::collection::map::map::{make_hash, make_hasher};
use crate::collection::map::raw_rbtree_hashmap::{Iter, RBTreeHashMap as RawMap};
use crate::collection::map::{Entry, Map};
use std::hash::{BuildHasher, Hash, RandomState};

pub struct HashMap<K, V, S = RandomState>(RawMap<K, V>, S);

impl<K, V> HashMap<K, V, RandomState>
{
    pub fn new() -> Self {
        Self(RawMap::new(), Default::default())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_hasher(capacity, Default::default())
    }

    pub fn with_capacity_factor(capacity: usize, factor:f32) -> Self {
        Self::with_capacity_factor_hasher(capacity, factor, Default::default())
    }

    pub fn of(key: K, value: V) -> HashMap<K, V> where K: Hash + Eq
    {
        let mut map = Self::with_capacity(1);
        map.put(key, value);
        map
    }

    pub fn iter(&self) -> Iter<K, V> {
        self.0.iter()
    }
}

impl<K, V, S> HashMap<K, V, S>
where S: BuildHasher
{
    pub fn with_hasher(hash_builder: S) -> Self
    {
        Self(RawMap::new(), hash_builder)
    }

    pub fn with_capacity_hasher(capacity: usize, hash_builder: S) -> Self {
        Self(RawMap::with_capacity(capacity), hash_builder)
    }

    pub fn with_capacity_factor_hasher(capacity: usize, factor:f32, hash_builder: S) -> Self {
        Self(RawMap::with_capacity_factor(capacity, factor), hash_builder)
    }
}

impl<K, V, S> Map<K, V> for HashMap<K, V, S>
where K: Hash + Eq,
      S: BuildHasher,
{
    type Raw = ListBucket<K, V>;

    fn size(&self) -> usize {
        self.0.size()
    }

    fn get(&self, key: &K) -> Option<&V>
    {
        if self.is_empty() {
            return None;
        }
        let hash = make_hash(&self.1, key);
        self.0.get(hash, key)
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if self.is_empty() {
            return None;
        }
        let hash = make_hash(&self.1, key);
        self.0.get_mut(hash, key)
    }

    fn entry(&mut self, key: K) -> Entry<'_, K, V, Self::Raw>
    {
        let hash = make_hash(&self.1, &key);
        self.0.entry(hash, key, make_hasher(&self.1))
    }

    fn put(&mut self, key: K, value: V) -> Option<V> {
        let hash = make_hash(&self.1, &key);
        let hasher = make_hasher(&self.1);
        self.0.put(hash, key, value, hasher)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        if self.is_empty() {
            return None;
        }
        let hash = make_hash(&self.1, key);
        self.0.remove(hash, key)
    }

    fn foreach<F: FnMut(&K, &mut V)>(&mut self, f: F) {
        self.0.foreach(f)
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V>
where K: Eq + Hash,
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        self.0.iter()
    }
}
