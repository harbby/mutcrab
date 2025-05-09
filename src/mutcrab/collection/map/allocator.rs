use std::marker::PhantomData;

#[derive(Debug)]
pub struct Allocator<T> {
    pub size: usize,
    _marker: PhantomData<T>,
}

impl<T> Allocator<T> {
    pub fn new() -> Self {
        Allocator {
            size: 0,
            _marker: PhantomData,
        }
    }

    pub fn alloc(&mut self, obj: T) -> Box<T> {
        self.size += 1;
        Box::new(obj)
    }

    pub fn free(&mut self, obj: Box<T>) -> T {
        self.size -= 1;
        let old = *obj;
        // drop(obj);
        return old;
    }
}
