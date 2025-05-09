use crate::collection::tree::redblack_tree::TreeNode;
use std::fmt::{Display, Formatter};
use std::{ptr};

pub trait TreeCleaner<K, V> {
    fn clean_transfer<F>(&mut self, transfer: F)
    where F: FnMut(Box<TreeNode<K, V>>);
}

impl<K, V> TreeNode<K, V> {
    pub fn new(key: K, value: V, is_red: bool) -> Self {
        Self {
            key: key,
            value: value,
            is_red: is_red,
            left: ptr::null_mut(),
            right: ptr::null_mut(),
            parent: ptr::null_mut(),
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }

    // pub fn set_value(&mut self, value: V) -> V {
    //     mem::replace(&mut self.value, value)
    // }

    pub fn is_left_node(&self) -> bool {
        let p_left: *const TreeNode<K, V> = unsafe { (*self.parent).left };
        let this = self as *const TreeNode<K, V>;
        p_left == this
    }

    pub fn get_uncle(&self) -> *mut TreeNode<K, V> {
        unsafe {
            let grand_parent = &*(*self.parent).parent;
            if (*self.parent).is_left_node() {
                grand_parent.right
            } else {
                grand_parent.left
            }
        }
    }

    pub fn get_brother(&self) -> *mut TreeNode<K, V> {
        unsafe {
            if self.is_left_node() {
                (*self.parent).right
            } else {
                (*self.parent).left
            }
        }
    }
}

impl<K: Display, V> Display for TreeNode<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lv = unsafe { &(*self.left).key };
        let rv = unsafe { &(*self.right).key };
        write!(
            f,
            "value: {}, color: {}, left: {}, right: {}",
            &self.key, self.is_red, lv, rv
        )
    }
}
