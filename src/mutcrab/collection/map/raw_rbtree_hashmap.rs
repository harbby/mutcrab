use crate::base::numbers::next_power_of_two;
use crate::collection::map::allocator::Allocator;
use crate::collection::map::list_bucket::{EntryNode, ListBucket};
use crate::collection::map::map::Entry;
use crate::collection::map::map_bucket::{Bucket, BucketIter};

const DEFAULT_LOAD_FACTOR: f32 = 0.75;

#[derive(Debug)]
pub struct RBTreeHashMap<K, V> {
    allocator: Allocator<EntryNode<K, V>>,
    pub tab: Vec<Bucket<K, V>>,
    threshold: usize,
    load_factor: f32,
}

impl<K, V> RBTreeHashMap<K, V> {
    pub fn new() -> RBTreeHashMap<K, V> {
        Self::with_capacity(16)
    }

    pub fn with_capacity(init_cap: usize) -> RBTreeHashMap<K, V> {
        Self::with_capacity_factor(init_cap, DEFAULT_LOAD_FACTOR)
    }

    pub fn with_capacity_factor(init_cap: usize, load_factor: f32) -> RBTreeHashMap<K, V> {
        let capacity = next_power_of_two(init_cap);
        RBTreeHashMap {
            allocator: Allocator::new(),
            tab: Vec::new(),
            threshold: capacity,
            load_factor: load_factor,
        }
    }

    // hasher: impl Fn(&K) -> u64
    fn reserve(&mut self, additional: usize, hasher: impl Fn(&K) -> u64) {
        if !self.tab.is_empty() && self.size() + additional <= self.threshold {
            return;
        }
        // resize
        let old_capacity = self.tab.len();
        let capacity = if old_capacity > 0 { old_capacity * 2 } else { self.threshold };

        let mut new_tab: Vec<Bucket<K, V>> = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            new_tab.push(Bucket::new_list());
        }
        Bucket::transfer(&mut self.tab, &mut new_tab, hasher);
        self.tab = new_tab;
        self.threshold = (capacity as f32 * self.load_factor) as usize;
    }

    // iterator
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            tab: &self.tab,
            index: 0,
            cur_iter: BucketIter::empty(),
        }
    }

    #[inline]
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

impl<K, V> RBTreeHashMap<K, V>
where
    K: Eq,
{
    pub fn get(&self, hash: u64, key: &K) -> Option<&V> {
        let node = self.get_node(hash, key)?;
        return Some(node);
    }

    pub fn get_mut(&mut self, hash: u64, key: &K) -> Option<&mut V> {
        return self.get_node(hash, key);
    }

    fn get_node(&self, hash: u64, key: &K) -> Option<&mut V> {
        debug_assert!(!self.tab.is_empty(), "map not initialized");
        let mask = self.tab.len() - 1;
        let index = hash as usize & mask;
        return self.tab[index].get(hash, key);
    }

    pub fn entry(&mut self, hash: u64, key: K, hasher: impl Fn(&K) -> u64) -> Entry<'_, K, V, ListBucket<K, V>>
    {
        self.reserve(1, hasher);
        let mask = self.tab.len() - 1;
        let index = hash as usize & mask;
        return self.tab[index].entry(hash, key, &mut self.allocator)
    }

    pub fn put(&mut self, hash: u64, key: K, value: V, hasher: impl Fn(&K) -> u64) -> Option<V> {
        self.reserve(1, &hasher);
        let mask = self.tab.len() - 1;
        let index = hash as usize & mask;
        return self.tab[index].write(hash, key, value, &mut self.allocator, &hasher);
    }

    pub fn remove(&mut self, hash: u64, key: &K) -> Option<V> {
        let mask = self.tab.len() - 1;
        let index = hash as usize & mask;
        return self.tab[index].remove(hash, key, &mut self.allocator);
    }
}

// iterator
pub struct Iter<'a, K, V> {
    tab: &'a Vec<Bucket<K, V>>,
    index: usize,
    cur_iter: BucketIter<'a, K, V>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.tab.len();
        loop {
            if let Some(mut item) = self.cur_iter.next() {
                let node = unsafe { item.as_mut() };
                return Some((&node.key, &mut node.value));
            }

            if self.index >= len {
                return None;
            }

            // unsafe：转换生命周期，因为我们知道 tab 是 'a 生命周期
            // let iter: BucketIter<'a, K, V> = unsafe { std::mem::transmute(iter) };
            self.cur_iter = self.tab[self.index].iter();
            self.index += 1;
        }
    }
}


#[cfg(test)]
mod test {
    use crate::collection::map::raw_rbtree_hashmap::RBTreeHashMap;

    #[test]
    fn base_test1() {
        let mut map = RBTreeHashMap::<i32, &str>::new();
        let hasher = |x:&i32 | -> u64 { *x as u64 };
        map.put(1, 1, &"a", hasher);
        map.put(2, 2, &"b", hasher);
        map.put(3, 3, &"c", hasher);
        map.put(4, 4, &"d", hasher);
        map.put(5, 5, &"e", hasher);

        assert_eq!(map.get(1, &1), Some(&"a"));
        assert_eq!(map.get(2, &2), Some(&"b"));
        assert_eq!(map.get(3, &3), Some(&"c"));
        assert_eq!(map.get(4, &4), Some(&"d"));
        assert_eq!(map.get(5, &5), Some(&"e"));
    }

    #[test]
    fn base_tree_test1() {
        let mut map = RBTreeHashMap::<u8, char>::new();
        let hasher = |x:&u8 | -> u64 { *x as u64 / 10 + 1 };
        for i in 0..10 {
            let hash = hasher(&i);
            let ch = (b'a' + i) as char;
            map.put(hash, i, ch, hasher);
            assert_eq!(map.size(), i as usize + 1);
            assert_eq!(map.get(hash, &i), Some(&ch));
        }
        for i in 10..16 {
            let ch = (b'a' + i) as char;
            map.put(hasher(&i), i, ch, hasher);
            assert_eq!(map.size(), i as usize + 1);
        }

        for i in 0..16 {
            let ch = (b'a' + i) as char;
            assert_eq!(map.get(hasher(&i), &i), Some(&ch));
        }
    }

    #[test]
    fn test_foreach() {
        let mut map = RBTreeHashMap::<u8, char>::new();
        let hasher = |x:&u8 | -> u64 { *x as u64 / 10 + 1 };

        for i in 0..10 {
            let ch = (b'a' + i) as char;
            map.put(hasher(&i), i, ch, hasher);
        }
        for i in 10..16 {
            let ch = (b'a' + i) as char;
            map.put(hasher(&i), i, ch, hasher);
        }
        let mut keys: Vec<u8> = map.iter().map(|(k, _v)| *k).collect();
        let mut values: Vec<char> = map.iter().map(|(_k, v)| *v).collect();
        keys.sort();
        values.sort();
        assert_eq!(
            keys,
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
        );
        assert_eq!(
            values,
            vec![
                'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p'
            ]
        );
    }
}
