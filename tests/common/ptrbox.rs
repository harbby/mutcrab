use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::{fmt, ptr};

pub struct PtrBox<T> {
    ptr: *mut T,
}

impl<T> PtrBox<T> {
    pub fn new(value: T) -> PtrBox<T> {
        PtrBox { ptr: Box::into_raw(Box::new(value)) }
    }

    pub fn null_ptr() -> PtrBox<T> {
        PtrBox { ptr: ptr::null_mut() }
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    pub fn as_box(&self) -> Box<T> {
        debug_assert!(!self.is_null(), "null pointer");
        unsafe { Box::from_raw(self.ptr) }
    }
}

impl<T> Drop for PtrBox<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(self.ptr);
            }
        }
    }
}

// 实现 Deref 来支持解引用（像 Box 一样使用 *my_box）
impl<T> Deref for PtrBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        debug_assert!(!self.is_null(), "null pointer");
        unsafe { &*self.ptr }
    }
}

// 实现 DerefMut 来支持可变解引用
impl<T> DerefMut for PtrBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        debug_assert!(!self.is_null(), "null pointer");
        unsafe { &mut *self.ptr }
    }
}

impl<T> Display for PtrBox<T>
where T: Display
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.deref())
    }
}