
use crate::collection::tree::{RBTree, TreeCleaner, RBIter, TreeNode};
use std::ptr::NonNull;
use crate::collection::map::Entry;
use crate::collection::map::map_bucket::Bucket;
use crate::collection::map::list_bucket::{EntryNode, ListBucket, ListBucketIter};
use crate::collection::map::allocator::Allocator;


#[derive(Debug)]
pub struct TreeBucket<K, V> {
    pub tree: RBTree<u64, ListBucket<K, V>>,
}

impl<K, V> TreeBucket<K, V> {
    pub fn new() -> Self {
        TreeBucket {
            tree: RBTree::<u64, ListBucket<K, V>>::new(),
        }
    }
}

impl<K, V> TreeBucket<K, V> {
    pub fn write(&mut self, hash: u64, key: K, value: V, allocator: &mut Allocator<EntryNode<K, V>>) -> Option<V>
    where
        K: Eq,
    {
        let list = self.tree.get_or_insert(hash, ListBucket::new());
        return list.write(key, value, allocator).1;
    }

    pub fn push(&mut self, hash: u64, ptr: &mut EntryNode<K, V>) {
        self.tree.get_or_insert(hash, ListBucket::new()).push(ptr);
    }

    pub fn get(&self, hash: u64, key: &K) -> Option<&mut V>
    where
        K: Eq,
    {
        return self.tree.get(&hash)?.get(key);
    }

    pub fn entry<'a>(&'a mut self, hash: u64, key: K, allocator: &'a mut Allocator<EntryNode<K, V>>) -> Entry<'a, K, V, ListBucket<K, V>>
    where K: Eq {
        let list:&mut ListBucket<K, V> = self.tree.get_or_insert(hash, ListBucket::new());
        return list.entry(hash, key, allocator);
    }

    pub fn split_transfer(&mut self, old_cap: usize, i: usize, new_tab: &mut Vec<Bucket<K, V>>) {
        let mut lo_head: *mut TreeNode<u64, ListBucket<K, V>> = std::ptr::null_mut();
        let mut hi_head: *mut TreeNode<u64, ListBucket<K, V>> = std::ptr::null_mut();
        let mut lo_count: usize = 0;
        let mut hi_count: usize = 0;
        self.tree.clean_transfer(|mut tree_node| {
            let hash = tree_node.key;
            let count = tree_node.value.count(); // move
            debug_assert!(count > 0, "assert failed, key is hash: {hash},but value not found");
            if hash as usize & old_cap == 0 {
                tree_node.next = lo_head;
                lo_head = Box::into_raw(tree_node);
                lo_count += count;
            } else {
                tree_node.next = hi_head;
                hi_head = Box::into_raw(tree_node);
                hi_count += count;
            }
        });

        if lo_count > 0 {
            new_tab[i] = Self::build_branch(lo_head, lo_count);
        }
        if hi_count > 0 {
            new_tab[i + old_cap] = Self::build_branch(hi_head, hi_count);
        }
    }

    fn build_branch(
        head: *mut TreeNode<u64, ListBucket<K, V>>,
        count: usize,
    ) -> Bucket<K, V> {
        if count <= 6 {
            let mut ptr = head;
            let mut list = ListBucket::new();
            while !ptr.is_null() {
                let node = unsafe { Box::from_raw(ptr) };
                ptr = node.next;
                list.merge(node.value);
            }
            Bucket::with_list(list)
        } else {
            let mut tree = TreeBucket::new();
            let mut ptr = head;
            while !ptr.is_null() {
                let node = unsafe { Box::from_raw(ptr) };
                ptr = node.next;
                tree.tree.merge_node(node);
            }
            Bucket::with_tree(tree)
        }
    }

    pub fn iter(&self) -> TreeBucketIter<'_, K, V> {
        TreeBucketIter {
            tree_iter: self.tree.iter(),
            cur: ListBucketIter::empty()
        }
    }
}

pub struct TreeBucketIter<'a, K, V> {
    tree_iter: RBIter<'a, u64, ListBucket<K, V>>,
    cur: ListBucketIter<K, V>,
}

impl<K, V> Iterator for TreeBucketIter<'_, K, V> {
    type Item = NonNull<EntryNode<K, V>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ptr) = self.cur.next() {
                return Some(ptr);
            }
            let list = self.tree_iter.next()?;
            self.cur = list.1.iter()
        }
    }
}
