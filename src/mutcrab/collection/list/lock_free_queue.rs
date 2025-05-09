use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

struct Node<T> {
    value: MaybeUninit<T>,
    next: AtomicPtr<Node<T>>,
}

unsafe impl<T: Send> Send for Node<T> {} // 确保 Node 可跨线程传输
unsafe impl<T: Send> Sync for Node<T> {} // 仅允许 1 读 1 写，仍需 Sync

/**
rust经典的无锁MPSC FIFO队列
当前的实现，最多允许 多写1读
*/
#[derive(Debug)]
pub struct LockFreeQueue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
    _marker: PhantomData<T>,
}

impl<T> LockFreeQueue<T> {
    pub fn new() -> Self {
        let dummy = Box::into_raw(Box::new(Node {
            value: MaybeUninit::uninit(),
            next: AtomicPtr::new(ptr::null_mut()),
        }));
        Self {
            head: AtomicPtr::new(dummy),
            tail: AtomicPtr::new(dummy),
            _marker: PhantomData,
        }
    }

    /**
    MPSC(multiple Producer, single Consumer)
    allow multiple threads to call simultaneously
    */
    pub fn push(&self, value: T) {
        let new_tail = Box::into_raw(Box::new(Node {
            value: MaybeUninit::new(value),
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        let prev_tail = self.tail.swap(new_tail, Ordering::AcqRel);
        unsafe { (*prev_tail).next.store(new_tail, Ordering::Release); }
    }

    pub fn pop(&self) -> Option<T> {
        let head = self.head.load(Ordering::Acquire);
        let next = unsafe { (*head).next.load(Ordering::Acquire) };

        if next.is_null() {
            return None; // 队列为空
        }

        let value = unsafe {
            // implement Move semantics
            std::mem::replace(&mut (*next).value, MaybeUninit::uninit()).assume_init()
        };

        self.head.store(next, Ordering::Release);
        unsafe { drop(Box::from_raw(head)) }; // 释放旧 head
        Some(value)
    }

    pub fn iter(&self) -> Iter<T> {
        let head = self.head.load(Ordering::Acquire);
        let next = unsafe { (*head).next.load(Ordering::Acquire) };
        Iter {
            cur: next,
            _marker: &self._marker,
        }
    }
}

impl<T> Drop for LockFreeQueue<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {} // 清理剩余节点
        unsafe { drop(Box::from_raw(self.head.load(Ordering::Relaxed))) }; // 释放 dummy
    }
}

impl<'a, T> IntoIterator for &'a LockFreeQueue<T>
{
    type Item = &'a mut T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        let head = self.head.load(Ordering::Acquire);
        let next = unsafe { (*head).next.load(Ordering::Acquire) };
        Iter {
            cur: next,
            _marker: &self._marker,
        }
    }
}

pub struct Iter<'a, T> {
    cur: *mut Node<T>,
    _marker: &'a PhantomData<T>,
}

impl<'a, T> Iterator for Iter<'a, T>
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.cur;
        while !node.is_null() {
            // Safely access the current node and update the pointer to the next node
        unsafe {
            self.cur = (*node).next.load(Ordering::Acquire);
            return Some((*node).value.assume_init_mut());
        }
    }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::collection::list::lock_free_queue::LockFreeQueue;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn single_readwrite_test() {
        use std::{sync::Arc, thread};
        use std::time::Duration;

        let queue0 = Arc::new(LockFreeQueue::<i32>::new());
        let queue = Arc::clone(&queue0);
        let p1 = thread::spawn(move || {
            println!("hello thread 1");
            for i in 0..1000 {
                queue.push(i);
            }
        });
        //-----add consumer
        let queue = Arc::clone(&queue0);
        let c1 = thread::spawn(move || {
            println!("hello consumer1");
            let mut vec: Vec<i32> = Vec::new();
            loop {
                if let Some(num) = queue.pop() {
                    if num == -1 {
                        break;
                    }
                    vec.push(num);
                } else {
                    thread::sleep(Duration::from_millis(1));
                }
            }
            return vec;
        });

        // 等待生产者完成
        p1.join().unwrap();
        // 发送终止信号，让消费者线程退出
        queue0.push(-1);
        // 等待消费者完成
        let v1 = c1.join().unwrap();
        assert_eq!(v1, (0..1000).collect::<Vec<_>>());
    }

    #[test]
    fn mut_write_and_single_read_test() {
        let queue0 = Arc::new(LockFreeQueue::<i32>::new());
        // add Producer
        let queue = Arc::clone(&queue0);
        let p1 = thread::spawn(move || {
            println!("hello thread 1");
            for i in 0..1000 {
                queue.push(i);
            }
        });
        let queue = Arc::clone(&queue0);
        let p2 = thread::spawn(move || {
            println!("hello thread 2");
            for i in 1000..2000 {
                queue.push(i);
            }
        });
        let queue = Arc::clone(&queue0);
        let p3 = thread::spawn(move || {
            println!("hello thread 3");
            for i in 2000..3000 {
                queue.push(i);
            }
        });
        //-----add consumer
        let queue = Arc::clone(&queue0);
        let c1 = thread::spawn(move || {
            println!("hello consumer1");
            let mut vec:Vec<i32> = Vec::new();
            loop {
                if let Some(num) = queue.pop() {
                    if num == -1 {
                        break;
                    }
                    vec.push(num);
                } else {
                    thread::sleep(Duration::from_millis(1));
                }
            }
            return vec;
        });

        // 等待生产者完成
        p1.join().unwrap();
        p2.join().unwrap();
        p3.join().unwrap();
        // 发送终止信号，让消费者线程退出
        queue0.push(-1);

        // 等待消费者完成
        let mut all_data = c1.join().unwrap();

        // 验证所有数据是否被正确消费
        all_data.sort();
        assert_eq!(all_data, (0..3000).collect::<Vec<_>>());
    }

    /**
    this is UB
    add #[test] verify
    */
    #[warn(dead_code)]
    fn mut_readwrite_test() {
        let queue0 = Arc::new(LockFreeQueue::<i32>::new());
        let queue = Arc::clone(&queue0);
        let p1 = thread::spawn(move || {
            println!("hello thread 1");
            for i in 0..1000 {
                queue.push(i);
            }
        });
        let queue = Arc::clone(&queue0);
        let p2 = thread::spawn(move || {
            println!("hello thread 2");
            for i in 1000..2000 {
                queue.push(i);
            }
        });
        let queue = Arc::clone(&queue0);
        let p3 = thread::spawn(move || {
            println!("hello thread 3");
            for i in 2000..3000 {
                queue.push(i);
            }
        });
        //-----add consumer
        let queue = Arc::clone(&queue0);
        let c1 = thread::spawn(move || {
            println!("hello consumer1");
            let mut vec:Vec<i32> = Vec::new();
            loop {
                if let Some(num) = queue.pop() {
                    if num == -1 {
                        break;
                    }
                    vec.push(num);
                } else {
                    thread::sleep(Duration::from_millis(1));
                }
            }
            return vec;
        });
        //-- add consumer2
        let queue = Arc::clone(&queue0);
        let c2 = thread::spawn(move || {
            println!("hello consumer1");
            let mut vec:Vec<i32> = Vec::new();
            loop {
                if let Some(num) = queue.pop() {
                    if num == -1 {
                        break;
                    }
                    vec.push(num);
                } else {
                    thread::sleep(Duration::from_millis(1));
                }
            }
            return vec;
        });

        // 等待生产者完成
        p1.join().unwrap();
        p2.join().unwrap();
        p3.join().unwrap();
        // 发送终止信号，让消费者线程退出
        queue0.push(-1);
        queue0.push(-1);

        // 等待消费者完成
        let v1 = c1.join().unwrap();
        let v2 = c2.join().unwrap();

        // 验证所有数据是否被正确消费
        let mut all_data = vec![v1, v2].concat();
        all_data.sort();
        assert_eq!(all_data, (0..3000).collect::<Vec<_>>());
    }
}
