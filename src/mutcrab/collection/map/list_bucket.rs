use crate::collection::map::map::{OccupiedEntry, VacantEntry};
use crate::collection::map::tree_bucket::TreeBucket;
use crate::collection::map::{Entry, RawTable};
use std::ptr::NonNull;
use std::{mem, ptr};
use crate::collection::map::map_bucket::Bucket;
use crate::collection::map::allocator::Allocator;

/**
The maximum length of the linked list is 8. If the length exceeds this,
the `convert_to_tree` function will be triggered to convert it to RedBlackTree
*/
const MAX_LIST_LENGTH: u32 = 8;

#[derive(Debug)]
pub struct EntryNode<K, V> {
    pub(crate) key: K,
    pub(crate) value: V,
    pub(crate) next: *mut EntryNode<K, V>,
}

#[derive(Debug)]
pub struct ListBucket<K, V> {
    root: *mut EntryNode<K, V>,
}

impl<K, V> Drop for ListBucket<K, V> {
    fn drop(&mut self) {
        let mut ptr = self.root;
        while !ptr.is_null() {
            ptr = unsafe { Box::from_raw(ptr).next }; // free ptr
        }
    }
}

impl<K, V> ListBucket<K, V> {
    pub fn new() -> Self {
        Self {
            root: std::ptr::null_mut(),
        }
    }

    pub fn count(&self) -> usize {
        let mut ptr = self.root;
        let mut count = 0;
        while !ptr.is_null() {
            count += 1;
            ptr = unsafe { (*ptr).next };
        }
        count
    }

    pub fn merge(&mut self, other: ListBucket<K, V>) {
        let mut ptr = other.root;
        while !ptr.is_null() {
            let node = unsafe { &mut *ptr };
            ptr = node.next;
            node.next = self.root;
            self.root = ptr;
        }
    }

    pub unsafe fn from_raw(head: *mut EntryNode<K, V>) -> Self {
        Self { root: head }
    }

    pub fn take(&mut self) -> *mut EntryNode<K, V> {
        let ptr = self.root;
        self.root = ptr::null_mut();
        return ptr;
    }

    pub fn push(&mut self, ptr: &mut EntryNode<K, V>) {
        ptr.next = self.root;
        self.root = ptr;
    }

    pub fn convert_to_tree(&mut self, hasher: impl Fn(&K) -> u64) -> TreeBucket<K, V> {
        let mut tree_bucket = TreeBucket::<K, V>::new();
        let mut ptr: *mut EntryNode<K, V> = mem::replace(&mut self.root, std::ptr::null_mut());
        while !ptr.is_null() {
            let node = unsafe { &mut *ptr };
            ptr = mem::replace(&mut node.next, std::ptr::null_mut());
            let hash = hasher(&node.key);
            tree_bucket.push(hash, node);
        }
        tree_bucket
    }

    pub fn iter(&self) -> ListBucketIter<K, V> {
        ListBucketIter::new(self.root)
    }

    pub fn foreach(&mut self, mut f: impl FnMut(&K, &mut V)) {
        let mut ptr = self.root;
        while !ptr.is_null() {
            let node = unsafe { &mut (*ptr) };
            f(&node.key, &mut node.value);
            ptr = node.next;
        }
    }

    pub fn transfer<F>(&mut self, old_cap: usize, index: usize, new_tab:&mut Vec<Bucket<K, V>>, hasher: F)
    where F: Fn(&K) -> u64
    {
        let new_cap = new_tab.len();
        let mut ptr = self.take(); //move
        if ptr.is_null() {
            return;
        }
        let node = unsafe { &mut (*ptr) };
        if node.next.is_null() {
            let hash = hasher(&node.key);
            new_tab[hash as usize & (new_cap - 1)].push(hash, node);
            return;
        }
        //--- keep order
        let mut lo_head:*mut EntryNode<K, V> = ptr::null_mut();
        let mut lo_tail:*mut EntryNode<K, V> = ptr::null_mut();
        let mut hi_head:*mut EntryNode<K, V> = ptr::null_mut();
        let mut hi_tail:*mut EntryNode<K, V> = ptr::null_mut();
        loop {
            let node = unsafe { &mut (*ptr) };
            ptr = node.next;
            if hasher(&node.key) as usize & old_cap == 0 {
                if lo_tail.is_null() {
                    lo_head = node;
                } else {
                    unsafe { (*lo_tail).next = node; }
                }
                lo_tail = node;
            } else {
                if hi_tail.is_null() {
                    hi_head = node;
                } else {
                    unsafe { (*hi_tail).next = node; }
                }
                hi_tail = node;
            }
            if ptr.is_null() {
                break;
            }
        }
        if !lo_tail.is_null() {
            unsafe { (*lo_tail).next = std::ptr::null_mut() };
            new_tab[index] = Bucket::with_list(ListBucket{root: lo_head});
        }
        if !hi_tail.is_null() {
            unsafe { (*hi_tail).next = std::ptr::null_mut() };
            new_tab[index + old_cap] = Bucket::with_list(ListBucket{root: hi_head});
        }
    }
}

impl<K, V> ListBucket<K, V>
where
    K: Eq,
{
    pub fn entry<'a>(&'a mut self, hash: u64, key: K, allocator: &'a mut Allocator<EntryNode<K, V>>) -> Entry<'a, K, V, ListBucket<K, V>> {
        let mut ptr = self.root;
        while !ptr.is_null() {
            let node = unsafe { &mut (*ptr) };
            if key == node.key {
                return Entry::Occupied(OccupiedEntry::new(&node.key, &mut node.value));
            }
            ptr = node.next;
        }
        Entry::Vacant(VacantEntry::new(key, hash, self, allocator))
    }

    pub fn get(&self, key: &K) -> Option<&mut V> {
        let mut node = self.root;
        while !node.is_null() {
            let ptr = unsafe { &mut (*node) };
            if *key == ptr.key {
                return Some(&mut ptr.value);
            }
            node = ptr.next;
        }
        return None;
    }

    pub fn write(&mut self, key: K, value: V, allocator: &mut Allocator<EntryNode<K, V>>) -> (bool, Option<V>) {
        let mut ptr: *mut EntryNode<K, V> = self.root;
        let mut count: u32 = 0;
        while !ptr.is_null() {
            let node = unsafe { &mut *ptr };
            if key == node.key {
                let old = std::mem::replace(&mut node.value, value);
                return (false, Some(old));
            }
            ptr = node.next;
            count += 1;
        }
        // add node
        self.add_node(key, value, allocator);
        return (count + 1 > MAX_LIST_LENGTH, None);
    }

    pub fn remove(&mut self, key: &K, allocator: &mut Allocator<EntryNode<K, V>>) -> Option<V> {
        let mut ptr = self.root;
        let mut last: *mut EntryNode<K, V> = std::ptr::null_mut();
        while !ptr.is_null() {
            let node = unsafe { &mut (*ptr) };
            if node.key == *key {
                unsafe {
                    if last.is_null() {
                        self.root = node.next;
                    } else {
                        (*last).next = node.next;
                    }
                    return Some(allocator.free(Box::from_raw(node)).value);
                }
            }
            last = ptr;
            ptr = node.next;
        }
        return None;
    }
}

impl <K, V> RawTable<K, V> for ListBucket<K, V> {
    fn add_node(&mut self, key: K, value: V, allocator: &mut Allocator<EntryNode<K, V>>) -> (&K, &mut V) {
        let heap_node = allocator.alloc(EntryNode {
            key: key,
            value: value,
            next: self.root,
        });
        let new_node = Box::leak(heap_node);
        self.root = new_node;
        return (&new_node.key, &mut new_node.value);
    }
}

pub struct ListBucketIter<K, V>(*mut EntryNode<K, V>);

impl<K, V> ListBucketIter<K, V> {
    pub fn new (ptr: *mut EntryNode<K, V>) -> Self {
        ListBucketIter(ptr)
    }

    pub fn empty () -> Self {
        ListBucketIter(std::ptr::null_mut())
    }
}

impl<K, V> Iterator for ListBucketIter<K, V> {
    type Item = NonNull<EntryNode<K, V>>;

    fn next(&mut self) -> Option<Self::Item> {
        let ptr = self.0;
        if ptr.is_null() {
            None
        } else {
            unsafe {
                self.0 = (*ptr).next;
                let node = NonNull::new_unchecked(ptr);
                return Some(node);
            }
        }
    }
}
