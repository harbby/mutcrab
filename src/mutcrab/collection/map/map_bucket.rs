use std::ptr::NonNull;
use crate::collection::map::Entry;
use crate::collection::map::list_bucket::{EntryNode, ListBucket, ListBucketIter};
use crate::collection::map::tree_bucket::{TreeBucket, TreeBucketIter};
use crate::collection::map::allocator::Allocator;

#[derive(Debug)]
pub struct Bucket<K, V> {
    bucket: BucketEnum<K, V>,
}

#[derive(Debug)]
pub enum BucketEnum<K, V> {
    List(ListBucket<K, V>),
    Tree(TreeBucket<K, V>),
}

impl<K, V> Bucket<K, V> {
    pub fn new_list() -> Self {
        Self {
            bucket: BucketEnum::List(ListBucket::new()),
        }
    }

    pub fn with_list(bucket: ListBucket<K, V>) -> Self {
        Self {
            bucket: BucketEnum::List(bucket),
        }
    }

    pub fn with_tree(bucket: TreeBucket<K, V>) -> Self {
        Self {
            bucket: BucketEnum::Tree(bucket),
        }
    }

    pub fn take(self) -> BucketEnum<K, V> {
       self.bucket
    }

    pub fn remove(&mut self, hash: u64, key: &K, allocator: &mut Allocator<EntryNode<K, V>>) -> Option<V>
    where
        K: Eq,
    {
        match &mut self.bucket {
            BucketEnum::List(list) => {
                list.remove(key, allocator)
            }
            BucketEnum::Tree(tree) => {
                tree.tree.remove_if(&hash, |x| {true});
                todo!();
            }
        }
    }

    pub fn write(&mut self, hash: u64, key: K, value: V, allocator: &mut Allocator<EntryNode<K, V>>, hasher: impl Fn(&K) -> u64) -> Option<V>
    where
        K: Eq,
    {
        match &mut self.bucket {
            BucketEnum::Tree(tree) => tree.write(hash, key, value, allocator),
            BucketEnum::List(list) => {
                let (is_gt8, option) = list.write(key, value, allocator);
                if is_gt8 == true {
                    let tree_bucket: TreeBucket<K, V> = list.convert_to_tree(hasher);
                    self.bucket = BucketEnum::Tree(tree_bucket);
                }
                option
            }
        }
    }

    pub fn push(&mut self, hash: u64, node: &mut EntryNode<K, V>) {
        match &mut self.bucket {
            BucketEnum::List(bucket) => {
                bucket.push(node);
            }
            BucketEnum::Tree(bucket) => {
                bucket.push(hash, node);
            }
        }
    }

    pub fn entry<'a>(&'a mut self, hash: u64, key: K, allocator: &'a mut Allocator<EntryNode<K, V>>) -> Entry<'a, K, V, ListBucket<K, V>>
    where K: Eq,
    {
        match &mut self.bucket {
            BucketEnum::List(list) => list.entry(hash, key, allocator),
            BucketEnum::Tree(tree) => tree.entry(hash, key, allocator),
        }
    }

    pub fn get(&self, hash: u64, key: &K) -> Option<&mut V>
    where
        K: Eq,
    {
        return match &self.bucket {
            BucketEnum::List(list) => list.get(key),
            BucketEnum::Tree(tree) => tree.get(hash, key),
        };
    }

    pub fn foreach<F: FnMut(&K, &mut V)>(&mut self, mut f: F) {
        match &mut self.bucket {
            BucketEnum::List(list) => {
                list.foreach(&mut f);
            }
            BucketEnum::Tree(tree) => {
                for (_, v) in &tree.tree {
                    v.foreach(&mut f);
                }
            }
        }
    }

    pub fn transfer<F>(tab: &mut Vec<Bucket<K, V>>, new_tab: &mut Vec<Bucket<K, V>>, hasher: F)
    where F: Fn(&K) -> u64
    {
        let old_cap = tab.len();
        for i in 0..old_cap {
            match &mut tab[i].bucket {
                BucketEnum::List(bucket) =>  { bucket.transfer(old_cap, i, new_tab, &hasher) },
                BucketEnum::Tree(tree) => tree.split_transfer(old_cap, i, new_tab),
            }
        }
    }

    pub fn iter(&self) -> BucketIter<'_, K, V> {
        match &self.bucket {
            BucketEnum::List(list) => BucketIter::List(list.iter()),
            BucketEnum::Tree(tree) => BucketIter::Tree(tree.iter()),
        }
    }
}

pub enum BucketIter<'a, K, V> {
    List(ListBucketIter<K, V>),
    Tree(TreeBucketIter<'a, K, V>),
}

impl<'a, K, V> BucketIter<'a, K, V>  {
    pub fn empty() -> Self {
        Self::List(ListBucketIter::empty())
    }
}

impl<'a, K, V> Iterator for BucketIter<'a, K, V> {
    type Item = NonNull<EntryNode<K, V>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            BucketIter::List(iter) => iter.next(),
            BucketIter::Tree(iter) => iter.next(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::collection::map::map_bucket::{Bucket, BucketEnum};
    use crate::collection::map::allocator::Allocator;

    #[test]
    fn test_convert_to_tree() {
        let mut allocator = Allocator::new();
        let mut bucket:Bucket<i32, &str> = Bucket::new_list();
        let hasher = |x:&i32 | -> u64 { 1 };
        for i in 0..8 {
            bucket.write(hasher(&i), i, "a", &mut allocator, hasher);
            assert!(matches!(&bucket.bucket, BucketEnum::List(_)));
            assert_eq!(allocator.size, i as usize + 1);
        }
        bucket.write(hasher(&8), 8, "a", &mut allocator, hasher);
        assert_eq!(allocator.size, 9);
        assert!(matches!(&bucket.bucket, BucketEnum::Tree(_)));
    }
}