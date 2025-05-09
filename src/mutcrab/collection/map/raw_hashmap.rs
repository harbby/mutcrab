use crate::base::numbers::next_power_of_two;
use crate::collection::map::list_bucket::{EntryNode, ListBucket, ListBucketIter};
use crate::collection::map::map::Entry;
use crate::collection::map::allocator::Allocator;

const DEFAULT_LOAD_FACTOR:f32 = 0.75;

pub struct HashTable<K, V>
{
    tab: Vec<ListBucket<K, V>>,
    threshold: usize,
    load_factor:f32,
    allocator: Allocator<EntryNode<K, V>>
}

impl<K, V> Default for HashTable<K, V>
{
    fn default() -> Self {
        HashTable::new()
    }
}

impl<K, V> HashTable<K, V>
{
    pub fn new() -> HashTable<K, V> {
        Self::with_capacity(16)
    }

    pub fn with_capacity(init_cap: usize) -> HashTable<K, V> {
        Self::with_capacity_factor(init_cap, DEFAULT_LOAD_FACTOR)
    }

    pub fn with_capacity_factor(init_cap: usize, load_factor: f32) -> HashTable<K, V> {
        let capacity = next_power_of_two(init_cap);
        HashTable {
            tab: Vec::new(),
            allocator: Allocator::new(),
            threshold: capacity,
            load_factor: load_factor,
        }
    }

    // iterator
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter { tab: &self.tab, index: 0 , cur: ListBucketIter::empty() }
    }

    // hasher: impl Fn(&K) -> u64
    pub fn reserve(&mut self, additional: usize, hasher: impl Fn(&K) -> u64) {
        if !self.tab.is_empty() && self.size() + additional <= self.threshold {
            return;
        }
        // resize
        let old_capacity = if self.tab.is_empty() {0} else {self.tab.len()};
        let capacity = if old_capacity > 0 {old_capacity * 2} else {self.threshold};

        let mut new_tab: Vec<ListBucket<K, V>> = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            new_tab.push(ListBucket::new());
        }

        for i in 0..old_capacity {
            let mut node = self.tab[i].take();
            while !node.is_null() {
                let ptr = unsafe { &mut (*node) };
                node = ptr.next;
                // 将当前节点插入到新的桶中
                let hash = hasher(&ptr.key);
                let index = hash as usize & (capacity - 1);
                new_tab[index].push(ptr);
            }
        }

        self.tab = new_tab;
        self.threshold = (capacity as f32 * self.load_factor) as usize;
    }

    pub fn size(&self) -> usize {
        self.allocator.size
    }

    pub fn foreach<F: FnMut(&K, &mut V)>(&mut self, mut f: F) {
        if self.tab.is_empty() {
            return;
        }
        for i in 0..self.tab.len() {
            self.tab[i].foreach(&mut f);
        }
    }
}

impl<K, V> HashTable<K, V>
where K: Eq,
{
    pub fn get(&self, hash: u64, key: &K) -> Option<&V> {
        Some(self.get_node(hash, key)?)
    }

    pub fn get_mut(&mut self, hash: u64, key: &K) -> Option<&mut V> {
        return self.get_node(hash, key);
    }

    #[inline]
    fn get_node(&self, hash: u64, key: &K) -> Option<&mut V> {
        debug_assert!(!self.tab.is_empty(), "map not initialized");
        let mask = self.tab.len() - 1;
        let index = hash as usize & mask;
        self.tab[index].get(key)
    }

    pub fn put(&mut self, hash: u64, key: K, value: V, hasher: impl Fn(&K) -> u64) -> Option<V> {
        self.reserve(1, hasher);
        let mask = self.tab.len() - 1;
        let index = hash as usize & mask;
        return self.tab[index].write(key, value, &mut self.allocator).1;
    }

    pub fn remove(&mut self, hash: u64, key: &K) -> Option<V> {
        debug_assert!(!self.tab.is_empty(), "map not initialized");
        let mask = self.tab.len() - 1;
        let index = hash as usize & mask;
        return self.tab[index].remove(key, &mut self.allocator);
    }

    pub fn entry(&mut self, hash: u64, key: K, hasher: impl Fn(&K) -> u64) -> Entry<'_, K, V, ListBucket<K, V>> {
        self.reserve(1, hasher);
        let mask = self.tab.len() - 1;
        let index = hash as usize & mask;
        let bucket = &mut self.tab[index];
        return bucket.entry(hash, key, &mut self.allocator);
    }
}

// iterator
impl<'a, K, V> IntoIterator for &'a HashTable<K, V>
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
    }
}

pub struct Iter<'a, K, V> {
    tab: &'a Vec<ListBucket<K, V>>,
    index: usize,
    cur: ListBucketIter<K, V>
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
{
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        let tab = self.tab;
        loop {
            if let Some(mut ptr) = self.cur.next() {
                let node = unsafe {  ptr.as_mut() };
                return Some((&node.key, &mut node.value))
            }
            if self.index >= tab.len() {
                return None;
            }
            self.cur = self.tab[self.index].iter();
            self.index +=1;
        }
    }
}
