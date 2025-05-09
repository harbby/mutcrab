use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;
use crate::collection::map::list_bucket::EntryNode;
use crate::collection::map::allocator::Allocator;

pub trait Map<K, V>
{
    type Raw: Sized + RawTable<K, V>;

    fn size(&self) -> usize;

    fn len(&self) -> usize {
        self.size()
    }

    fn get(&self, key: &K) -> Option<&V>;

    fn get_mut(&mut self, key: &K) -> Option<&mut V>;

    fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    fn entry(&mut self, key: K) -> Entry<'_, K, V, Self::Raw>;

    fn put(&mut self, key: K, value: V) -> Option<V>;

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        return self.put(key, value)
    }

    fn remove(&mut self, key: &K) -> Option<V>;

    fn foreach<F: FnMut(&K, &mut V)>(&mut self, f: F);
}

pub enum Entry<'a, K, V, M>
where M: RawTable<K, V>
{
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V, M>),
}

impl<'a, K, V, M> Entry<'a, K, V, M>
where
    K: Eq + Hash,
    M: RawTable<K, V>
{
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(x) => x._value,
            Entry::Vacant(x) => x.or_insert(default),
        }
    }

    pub fn take_insert(self, default: V) -> Option<V> {
        match self {
            Entry::Occupied(mut x) => Some(x.take_insert(default)),
            Entry::Vacant(x) => x.take_insert(default),
        }
    }

    pub fn key(&'a self) -> &'a K {
        match *self {
            Entry::Occupied(ref x) => x.key(),
            Entry::Vacant(ref x) => x.key(),
        }
    }

    pub fn and_modify<F>(self, f: F) -> OccupiedEntry<'a, K, V>
    where F: FnOnce(&mut V),
          V: Default
    {
        match self {
            Entry::Occupied(x) => x.and_modify(f),
            Entry::Vacant(x) => x.and_modify(f),
        }
    }
}

pub struct OccupiedEntry<'a, K, V> {
    _key: &'a K,
    _value: &'a mut V,
}

impl <'a, K, V> OccupiedEntry<'a, K, V> {
    pub fn new(key:&'a K, value: &'a mut V) -> OccupiedEntry<'a, K, V> {
        OccupiedEntry {
            _key: key,
            _value: value,
        }
    }

    pub fn key(&self) -> &'a K {
        self._key
    }

    pub fn value(self) -> &'a mut V {
        self._value
    }

    pub fn take_insert(&mut self, value: V) -> V {
        let old = std::mem::replace(self._value, value);
        return old;
    }

    pub fn and_modify<F>(self, f: F) -> Self
    where F: FnOnce(&mut V) {
        f(self._value);
        OccupiedEntry::new(self._key, self._value)
    }
}

pub trait RawTable<K, V> {
    fn add_node(&mut self, key: K, value: V, allocator: &mut Allocator<EntryNode<K, V>>) -> (&K ,&mut V);
}

pub struct VacantEntry<'a, K, V, M: RawTable<K, V>>
{
    pub key: K,
    pub hash: u64,
    pub base: &'a mut M,
    pub allocator: &'a mut Allocator<EntryNode<K, V>>,
    pub _marker: PhantomData<V>,
}

impl <'a, K, V, M: RawTable<K, V>> VacantEntry<'a, K, V, M>
where K: Eq + 'a
{
    pub fn new(key: K, hash:u64, tab: &'a mut M, allocator: &'a mut Allocator<EntryNode<K, V>>) -> VacantEntry<'a, K, V, M> {
        VacantEntry { key: key, hash: hash, base: tab, allocator: allocator,_marker: PhantomData }
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn or_insert(self, value: V) -> &'a mut V {
        return self.base.add_node(self.key, value, self.allocator).1;
    }

    // take_and_insert
    pub fn take_insert(self, value: V) -> Option<V> {
        self.base.add_node(self.key, value, self.allocator);
        return None
    }

    pub fn and_modify<F>(self, f: F) -> OccupiedEntry<'a, K, V>
    where
        F: FnOnce(&mut V),
        V: Default
    {
        let mut value:V = Default::default();
        f(&mut value);

        let (kptr,vptr) = self.base.add_node(self.key, value, self.allocator);
        OccupiedEntry::new(kptr, vptr)
    }

    pub fn put_if_absent<F>(self, f: F) -> OccupiedEntry<'a, K, V>
    where
        F: FnOnce(&K) -> V,
    {
        let value = f(&self.key);
        let (kptr,vptr) = self.base.add_node(self.key, value, self.allocator);
        OccupiedEntry::new(kptr, vptr)
    }
}

#[cfg_attr(feature = "inline-more", inline)]
pub fn make_hash<K, S>(hash_builder: &S, key: &K) -> u64
where K: Hash,
      S: BuildHasher,
{
    hash_builder.hash_one(key)
}

#[cfg_attr(feature = "inline-more", inline)]
pub fn make_hasher<K, S>(hash_builder: &S) -> impl Fn(&K) -> u64 + '_
where K: Hash,
      S: BuildHasher,
{
    move | k:&K | ->u64 { make_hash(hash_builder, k) }
}
