use std::marker::PhantomData;
use crate::collection::list::LockFreeQueue;
use std::sync::{Condvar, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use crate::collection::list::lock_free_queue::Iter;

/**
This is the rust imitation of JAVA `LinkedBlockingQueue`, which implements the classic double-lock blocking queue.
This queue is very useful in balancing production and consumption rates in IO-intensive scenarios.
It does not cause extra CPU consumption due to excessive `CAS` spins.
*/
#[derive(Debug)]
pub struct LinkedBlockingQueue<T> {
    queue: LockFreeQueue<T>,
    capacity: u32,             // 0 表示无界队列
    not_empty: Condvar,        // 当队列非空时通知消费者
    not_full: Condvar,         // 当队列未满时通知生产者
    put_lock: Mutex<()>,
    take_lock: Mutex<()>,
    count: AtomicU32,
    _marker: PhantomData<T>,
}

unsafe impl<T: Send> Send for LinkedBlockingQueue<T> {}
unsafe impl<T: Send> Sync for LinkedBlockingQueue<T> {}


impl<T> LinkedBlockingQueue<T> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: u32) -> Self {
        let capacity = if capacity == 0 {u32::MAX} else {capacity};
        let not_empty = Condvar::new();
        let not_full = Condvar::new();
        LinkedBlockingQueue {
            queue: LockFreeQueue::new(),
            capacity,
            not_empty,
            not_full,
            put_lock: Mutex::new(()),
            take_lock: Mutex::new(()),
            count: AtomicU32::new(0),
            _marker: PhantomData,
        }
    }

    pub fn offer(&self, data: T) -> Option<T> {
        if self.len() == self.capacity {
            return Some(data);
        }
        let c;
        {
            let _guard = self.put_lock.lock().unwrap();
            let queue = &self.queue;
            // 检查队列是否满，若满则等待
            while self.len() == self.capacity {
                return Some(data);           // 队列满了
            }
            queue.push(data); // 执行队列操作，推送数据
            c = self.count.fetch_add(1, Ordering::AcqRel);
            if c + 1 < self.capacity {
                self.not_full.notify_one();
            }
        } //auto unlock
        if c == 0 {
            self.signal_not_empty(); // 唤醒等待的消费者
        }
        return None
    }

    pub fn push(&self, data: T) {
        let c;
        {
            let mut guard = self.put_lock.lock().unwrap();
            let queue = &self.queue;
            // 检查队列是否满，若满则等待
            while self.len() == self.capacity {
                guard = self.not_full.wait(guard).unwrap();           // 队列满了，等待空位
            }
            queue.push(data); // 执行队列操作，推送数据
            c = self.count.fetch_add(1, Ordering::AcqRel);
            if c + 1 < self.capacity {
                self.not_full.notify_one();
            }
        } //auto unlock
        if c == 0 {
            self.signal_not_empty(); // 唤醒等待的消费者
        }
    }

    pub fn take(&self) -> T {
        // 获取消费者锁,保证可见性
        let c;
        let value;
        {
            let mut guard = self.take_lock.lock().unwrap();
            while self.len() == 0 {
                guard = self.not_empty.wait(guard).unwrap();
            }
            value = self.queue.pop().unwrap();
            c = self.count.fetch_sub(1, Ordering::AcqRel);
            if c > 1 {
                self.not_empty.notify_one();
            }
        }
        if c == self.capacity {
            self.notify_not_full();
        }
        return value
    }

    fn notify_not_full(&self) {
        let _guard = self.put_lock.lock().unwrap();
        self.not_full.notify_one();
    }

    fn signal_not_empty(&self) {
        let _guard = self.take_lock.lock().unwrap();
        self.not_empty.notify_one();
    }

    pub fn poll(&self) -> Option<T> {
        if self.len() == 0 {
            return None;
        }
        let value:Option<T>;
        let c:u32;
        {
            let _guard = self.take_lock.lock().unwrap();
            let queue = &self.queue;
            value = queue.pop();
            if value.is_none() {
                return None;
            }
            c = self.count.fetch_sub(1, Ordering::AcqRel);
            if c > 1 {
                self.not_empty.notify_one();
            }
        }
        if c == self.capacity {
            self.notify_not_full();
        }
        return value
    }

    pub fn poll_timeout(&self, timeout: Duration) -> Option<T> {
        let value:Option<T>;
        let c:u32;
        {
            let mut guard = self.take_lock.lock().unwrap();
            let mut remaining = timeout;
            while self.len() == 0 {
                let begin = Instant::now();
                (guard, _) = self.not_empty.wait_timeout(guard, timeout).unwrap();
                let elapsed = begin.elapsed();
                if elapsed >= remaining {
                    return None;
                }
                remaining -= elapsed;
            }
            value = self.queue.pop();
            c = self.count.fetch_sub(1, Ordering::AcqRel);
            if c > 1 {
                self.not_empty.notify_one();
            }
        }
        if c == self.capacity {
            self.notify_not_full();
        }
        return value
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.queue.iter()
    }

    pub fn len(&self) -> u32 {
        self.count.load(Ordering::Acquire)
    }
}

// iterator
impl<'a, T> IntoIterator for &'a LinkedBlockingQueue<T>
{
    type Item = &'a mut T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}
