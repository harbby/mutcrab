use crate::collection::tree::tree_node::TreeCleaner;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::ptr;

#[derive(Debug)]
pub struct TreeNode<K, V> {
    pub(crate) key: K,
    pub(crate) value: V,     // duplicated list
    pub(crate) is_red: bool, //new node default color is red
    pub(crate) left: *mut TreeNode<K, V>,
    pub(crate) right: *mut TreeNode<K, V>,
    pub(crate) parent: *mut TreeNode<K, V>,
    // Added `prev` and `next` for non-recursive traversal
    pub(crate) prev: *mut TreeNode<K, V>,
    pub(crate) next: *mut TreeNode<K, V>,
}

#[derive(Debug)]
pub struct RBTree<K, V> {
    pub(crate) root: *mut TreeNode<K, V>,
    _marker: PhantomData<(K, V)>,
}

impl<K, V> RBTree<K, V> {
    pub fn new() -> Self {
        Self {
            root: ptr::null_mut(),
            _marker: PhantomData,
        }
    }
}

impl<K, V> Drop for RBTree<K, V> {
    fn drop(&mut self) {
        self.clean_transfer(|node| {
            drop(node)
        })
    }
}

impl<K, V> RBTree<K, V>
where
    K: Ord,
{
    pub fn get(&self, key: &K) -> Option<&mut V> {
        let mut ptr = self.root;
        while !ptr.is_null() {
            let node = unsafe { &mut *ptr };
            ptr = match key.cmp(&node.key) {
                Ordering::Equal => {
                    return Some(&mut node.value);
                },
                Ordering::Greater => node.right,
                Ordering::Less => node.left,
            };
        }
        return None;
    }

    pub fn remove_if<F>(&mut self, key: &K, f: F)
    where F: FnOnce(&mut V) -> bool
    {
        let mut ptr = self.root;
        while !ptr.is_null() {
            let node = unsafe { &mut *ptr };
            ptr = match key.cmp(&node.key) {
                Ordering::Equal => {
                    if f(&mut node.value) {
                        todo!("to be remove");
                    }
                    return;
                },
                Ordering::Greater => node.right,
                Ordering::Less => node.left,
            };
        }
        return;
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let mut ptr = self.root;
        while !ptr.is_null() {
            let node = unsafe { &mut *ptr };
            ptr = match key.cmp(&node.key) {
                Ordering::Equal => {
                    todo!("to be remove");
                    unsafe { return Some(Box::from_raw(ptr).value); }
                },
                Ordering::Greater => node.right,
                Ordering::Less => node.left,
            };
        }
        return None;
    }

    pub fn get_or_insert(&mut self, key: K, value: V) -> &mut V
    {
        if self.root.is_null() {
            self.root = self.create_node(key, value, false);
            return unsafe { &mut (*self.root).value };
        }

        let mut node = unsafe { &mut *self.root };
        loop {
            match key.cmp(&node.key) {
                Ordering::Equal => loop {
                    return &mut node.value;
                },
                Ordering::Greater => {
                    if node.right.is_null() {
                        let new_node = unsafe { &mut *self.create_node(key, value, true) };
                        self.insert_right(node, new_node);
                        return &mut new_node.value;
                    } else {
                        node = unsafe { &mut *node.right };
                    }
                }
                Ordering::Less => {
                    if node.left.is_null() {
                        let new_node = unsafe { &mut *self.create_node(key, value, true) };
                        self.insert_left(node, new_node);
                        return &mut new_node.value;
                    } else {
                        node = unsafe { &mut *node.left };
                    }
                }
            };
        }
    }

    pub fn merge_node(&mut self, mut other: Box<TreeNode<K, V>>) -> Option<V> {
        // init link
        other.next = ptr::null_mut();
        other.prev = ptr::null_mut();
        other.parent = ptr::null_mut();
        other.left = ptr::null_mut();
        other.right = ptr::null_mut();

        if self.root.is_null() {
            other.is_red = false;
            self.root = Box::into_raw(other);
            return None;
        }

        other.is_red = true;
        let mut node = unsafe { &mut *self.root };
        loop {
            match other.key.cmp(&node.key) {
                Ordering::Equal => {
                    return Some(std::mem::replace(&mut node.value, other.value));
                },
                Ordering::Greater => {
                    if node.right.is_null() {
                        let new_node = unsafe { &mut *Box::into_raw(other) };
                        self.insert_right(node, new_node);
                        return None;
                    }
                    node = unsafe { &mut *node.right };
                }
                Ordering::Less => {
                    if node.left.is_null() {
                        let new_node = unsafe { &mut *Box::into_raw(other) };
                        self.insert_left(node, new_node);
                        return None;
                    }
                    node = unsafe { &mut *node.left };
                }
            }
        }
    }

    pub fn put(&mut self, key: K, value: V) -> Option<V>
    {
        if self.root.is_null() {
            self.root = self.create_node(key, value, false);
            return None;
        }

        let mut node = unsafe { &mut *self.root };
        loop {
            match key.cmp(&node.key) {
                Ordering::Equal => {
                    return Some(std::mem::replace(&mut node.value, value));
                },
                Ordering::Greater => {
                    if node.right.is_null() {
                        let new_node = unsafe { &mut *self.create_node(key, value, true) };
                        self.insert_right(node, new_node);
                        return None;
                    }
                    node = unsafe { &mut *node.right };
                }
                Ordering::Less => {
                    if node.left.is_null() {
                        let new_node = unsafe { &mut *self.create_node(key, value, true) };
                        self.insert_left(node, new_node);
                        return None;
                    }
                    node = unsafe { &mut *node.left };
                }
            }
        }
    }

    pub fn contains(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
}

impl<K, V> TreeCleaner<K, V> for RBTree<K, V> {
    fn clean_transfer<F>(&mut self, mut transfer: F)
    where
        F: FnMut(Box<TreeNode<K, V>>),
    {
        if self.root.is_null() {
            return;
        }
        let mut ptr = self.root;
        loop {
            let mut node = unsafe { Box::from_raw(ptr) };
            ptr = node.next;
            node.right =  ptr::null_mut();
            node.left =  ptr::null_mut();
            node.parent = ptr::null_mut();
            node.next = ptr::null_mut();
            node.prev = ptr::null_mut();
            transfer(node);  // move node ownership
            if ptr.is_null() {
                break;
            }
        }
        let mut ptr = unsafe { (*self.root).prev };
        while !ptr.is_null() {
            let mut node = unsafe { Box::from_raw(ptr) };
            node.right =  ptr::null_mut();
            node.left =  ptr::null_mut();
            node.parent = ptr::null_mut();
            ptr = node.prev;
            transfer(node);  // move node ownership
        }
        self.root = ptr::null_mut();
    }
}

impl<K, V> RBTree<K, V> {
    pub fn iter(&self) -> RBIter<K, V> {
        self.into_iter()
    }

    fn create_node(&self, key: K, value: V, is_red: bool) -> *mut TreeNode<K, V> {
        Box::into_raw(Box::new(TreeNode::new(key, value, is_red)))
    }

    fn insert_right(&mut self, parent: &mut TreeNode<K, V>, new_node: &mut TreeNode<K, V>) {
        parent.right = new_node;
        new_node.parent = parent;
        // In-order link
        self.insert_right_in_order_link(new_node, parent);
        // balance_insert
        if parent.is_red {
            self.balance_insert(new_node, false);
        }
    }
    #[inline]
    fn insert_right_in_order_link(&mut self, new_node: &mut TreeNode<K, V>, parent: &mut TreeNode<K, V>, ) {
        new_node.prev = parent;
        new_node.next = parent.next;
        if !parent.next.is_null() {
            unsafe { (*parent.next).prev = new_node; }
        }
        parent.next = new_node;
    }
    #[inline]
    fn insert_left_in_order_link(&mut self, new_node: &mut TreeNode<K, V>, parent: &mut TreeNode<K, V>) {
        new_node.next = parent;
        new_node.prev = parent.prev;
        if !parent.prev.is_null() {
            unsafe { (*parent.prev).next = new_node; }
        }
        parent.prev = new_node;
    }

    fn insert_left(&mut self, parent: &mut TreeNode<K, V>, new_node: &mut TreeNode<K, V>) {
        parent.left = new_node;
        new_node.parent = parent;
        // In-order link
        self.insert_left_in_order_link(new_node, parent);
        // balance_insert
        if parent.is_red {
            self.balance_insert(new_node, true);
        }
    }

    fn rotate_left(&mut self, node: &mut TreeNode<K, V>) {
        unsafe {
            let right_child = &mut *node.right;
            node.right = right_child.left;
            if !right_child.left.is_null() {
                (*right_child.left).parent = node;
            }
            right_child.left = node;

            let parent = node.parent;
            right_child.parent = parent;
            if !parent.is_null() {
                if node.is_left_node() {
                    (*parent).left = right_child;
                } else {
                    (*parent).right = right_child;
                }
            }
            node.parent = right_child;
            if node as *mut TreeNode<K, V> == self.root {
                self.root = right_child;
            }
        }
    }

    fn rotate_right(&mut self, node: &mut TreeNode<K, V>) {
        unsafe {
            let left_child = &mut *node.left;
            node.left = left_child.right;
            if !left_child.right.is_null() {
                (*left_child.right).parent = node;
            }
            left_child.right = node;

            let parent = node.parent;
            left_child.parent = parent;
            if !parent.is_null() {
                if node.is_left_node() {
                    (*parent).left = left_child;
                } else {
                    (*parent).right = left_child;
                }
            }
            node.parent = left_child;
            if node as *mut TreeNode<K, V> == self.root {
                self.root = left_child;
            }
        }
    }

    fn balance_insert(
        &mut self,
        inserted_node: &mut TreeNode<K, V>,
        is_left_insert: bool,
    ) {
        let mut n: *mut TreeNode<K, V> = inserted_node;
        let mut is_left = is_left_insert;
        let mut parent;
        let mut grand_parent;
        unsafe {
            loop {
                //assert n.isRed;
                //assert n.parent.isRed;
                parent = &mut *(*n).parent; //NotNULL
                grand_parent = &mut *parent.parent; //NotNULL
                let uncle = (*n).get_uncle();
                if !(!uncle.is_null() && (*uncle).is_red) {
                    break
                }
                //Parent and uncle is red
                parent.is_red = false;
                (*uncle).is_red = false;
                grand_parent.is_red = true;
                if grand_parent as *mut TreeNode<K, V> == self.root {
                    grand_parent.is_red = false;
                    return;
                } else if !grand_parent.parent.is_null() && (*(grand_parent.parent)).is_red {
                    n = grand_parent;
                    is_left = grand_parent.is_left_node();
                    continue;
                }
                return;
            }
            //Parent is red but uncle is black
            if parent.is_left_node() == is_left {
                //Parent and N are on the same side
                if is_left {
                    //Parent and N are on the left
                    self.rotate_right(grand_parent);
                } else {
                    //Parent and N are on the right
                    self.rotate_left(grand_parent);
                }
                parent.is_red = false;
                grand_parent.is_red = true;
            } else {
                //rotate at twice
                if is_left {
                    //Parent is right and N is left
                    self.rotate_right(parent);
                    //Parent and N are on the right
                    self.rotate_left(grand_parent);
                } else {
                    //Parent is left and N is right
                    self.rotate_left(parent);
                    //Parent and N are on the left
                    self.rotate_right(grand_parent);
                }
                (*n).is_red = false;
                grand_parent.is_red = true;
            }
        }
    }

    fn swap_color(n1: &mut TreeNode<K, V>, n2: &mut TreeNode<K, V>) {
        let color = n1.is_red;
        n1.is_red = n2.is_red;
        n2.is_red = color;
    }
}

impl<'a, K, V> IntoIterator for &'a RBTree<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = RBIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        RBIter::new(self.root)
    }
}

pub struct RBIter<'a, K, V> {
    root: *mut TreeNode<K, V>,
    current: *mut TreeNode<K, V>,
    forward: bool, //true -> next，false -> prev
    _marker: PhantomData<&'a mut (K, V)>,
}

impl<'a, K, V> RBIter<'a, K, V> {
    fn new(root: *mut TreeNode<K, V>) -> Self {
        RBIter {
            root: root,
            current: root,
            forward: true,
            _marker: PhantomData,
        }
    }
}

impl<'a, K, V> Iterator for RBIter<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.root.is_null() {
            return None;
        }
        while self.current.is_null() {
            if self.forward {
                self.forward = false;
                self.current = unsafe { (*self.root).prev };
            } else {
                return None;
            }
        }

        unsafe {
            let node = &mut *self.current;
            self.current = if self.forward { node.next } else { node.prev };
            Some((&node.key, &mut node.value))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;
    use crate::collection::tree::RBTree;

    #[test]
    fn test_base() {
        let mut tree = RBTree::<i32, PhantomData<i32>>::new();
        let arr = vec![5,3,7,2,4,6,8];   //
        for i in arr.iter() {
            tree.put(i.clone(), PhantomData);
        }

        let mut ptr = tree.root;
        while !ptr.is_null() {
            let node = unsafe { &*ptr };
            println!("{:?}", node.key);
            ptr = node.next;
        }
        let mut ptr = tree.root;
        while !ptr.is_null() {
            let node = unsafe { &*ptr };
            println!("{:?}", node.key);
            ptr = node.prev;
        }
    }
}
