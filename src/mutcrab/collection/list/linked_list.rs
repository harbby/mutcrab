use std::marker::PhantomData;
use std::ptr;

struct Node<T> {
    value: T,
    prev: *mut Node<T>,
    next: *mut Node<T>,
}

pub struct LinkedList<T> {
    first: *mut Node<T>,
    last: *mut Node<T>,
    size: u32,
    _marker: PhantomData<T>,
}

impl <T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl <T> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList {
            first: ptr::null_mut(),
            last: ptr::null_mut(),
            size: 0,
            _marker: PhantomData,
        }
    }

    pub fn clear(&mut self) {
        let mut cur = self.first;
        while !cur.is_null() {
            unsafe {
                let node = Box::from_raw(cur);
                cur = node.next;
            }
        }
        self.size = 0;
        self.first = ptr::null_mut();
        self.last = ptr::null_mut();
    }

    pub fn push(&mut self, value: T) {
        self.add_first(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.remove_first()
    }

    pub fn add(&mut self, value: T) {
        self.add_last(value);
    }

    pub fn poll(&mut self) -> Option<T> {
        self.remove_first()
    }

    pub fn add_first(&mut self, value: T) {
        let f = self.first;
        let node = Box::into_raw(Box::new(Node { value, prev:  ptr::null_mut(), next: f}));
        self.first = node;

        if f.is_null() {
            self.last = node;
        } else {
            unsafe { (*f).prev = node; }
        }
        self.size += 1;
    }

    pub fn add_last(&mut self, value: T) {
        let l = self.last;
        let node = Box::into_raw(Box::new(Node { value, prev: l, next: ptr::null_mut() }));
        self.last = node;
        if l.is_null() {
            self.first = node;
        } else {
            unsafe { (*l).next = node; }
        }
        self.size += 1;
    }

    pub fn remove_first(&mut self) -> Option<T> {
        let f = self.first;
        if f.is_null() {
            return None;
        }
        unsafe {
            let node = Box::from_raw(f);
            let next = node.next;
            self.first = next;
            if next.is_null() {
                self.last = ptr::null_mut();
            }
            self.size -= 1;
            Some(node.value)
        }
    }

    pub fn remove_last(&mut self) -> Option<T> {
        let l = self.last;
        if l.is_null() {
            return None;
        }
        unsafe {
            let node = Box::from_raw(l);
            self.size -= 1;
            let prev = node.prev;
            self.last = prev;
            if prev.is_null() {
                self.first = ptr::null_mut();
            }
            Some(node.value)
        }
    }

    pub fn peek_first(&mut self) -> Option<&mut T> {
        let f = self.first;
        if f.is_null() {
            return None;
        }
        unsafe {
            Some(&mut (*f).value)
        }
    }

    pub fn peek_last(&mut self) -> Option<&mut T> {
        let l = self.last;
        if l.is_null() {
            return None;
        }
        unsafe {
            Some(&mut (*l).value)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn remove(&mut self, value: &T) -> bool
    where T: Eq
    {
        let mut cur = self.first;
        while !cur.is_null() {
            let ptr = unsafe { &mut *cur };
            let prev = ptr.prev;
            let next = ptr.next;
            if *value == ptr.value {
                unsafe {
                    if !prev.is_null() {
                        (*prev).next = next;
                    }
                    if !next.is_null() {
                        (*next).prev = prev;
                    }
                    drop(Box::from_raw(cur));
                }
                self.size -= 1;
                return true;
            }
            cur = next;
        }
        false
    }

    pub fn foreach<F: FnMut(&mut T)>(&mut self, mut f: F) {
        let mut cur = self.first;
        while !cur.is_null() {
            let ptr = unsafe { &mut *cur };
            f(&mut ptr.value);
            cur = ptr.next;
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            cur: self.first,
            _marker: &self._marker,
        }
    }
}

// iterator
impl<'a, T> IntoIterator for &'a LinkedList<T>
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

pub struct Iter<'a, T> {
    cur: *mut Node<T>,
    _marker: &'a PhantomData<T>,
}

impl<'a, T> Iterator for Iter<'a, T>
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.cur;
        while !node.is_null() {
            // Safely access the current node and update the pointer to the next node
            let node_ref = unsafe { &*node };
            self.cur = node_ref.next;
            return Some(&node_ref.value);
        }
        None
    }
}

