use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;
use mutcrab::collection::list::LinkedBlockingQueue;

#[test]
fn test_push_take_basic() {
    // 创建一个容量为 3 的队列
    let queue = LinkedBlockingQueue::<i32>::with_capacity(3);

    // 测试 push 操作
    queue.push(1);
    queue.push(2);
    queue.push(3);

    // 现在队列已经满了，接下来执行 take 操作
    assert_eq!(queue.take(), 1);
    assert_eq!(queue.take(), 2);
    assert_eq!(queue.take(), 3);
}

#[test]
fn test_take_on_empty_queue() {
    let queue = Arc::new(LinkedBlockingQueue::<i32>::with_capacity(3));

    // 这个测试模拟 take 在空队列时的行为
    // take 应该阻塞直到有数据
    assert_eq!(queue.poll().is_none(), true);
    let queue1= Arc::clone(&queue);
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));  //block time
        queue1.push(1);
    });
    // take block
    assert_eq!(queue.take(), 1);
}

#[test]
fn test_push_more_than_capacity() {
    let queue = Arc::new(LinkedBlockingQueue::<i32>::with_capacity(2));

    // 队列容量为 2，插入 3 个元素，最后一个应该会阻塞
    queue.push(1);
    queue.push(2);
    let queue1= Arc::clone(&queue);
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));  //block time
        queue1.take()
    });
    // put block
    queue.push(3);
}

#[test]
fn test_foreach() {
    let queue = Arc::new(LinkedBlockingQueue::<i32>::with_capacity(10));

    // 队列容量为 2，插入 3 个元素，最后一个应该会阻塞
    queue.push(1);
    queue.push(2);
    let queue1= Arc::clone(&queue);
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));  //block time
        queue1.push(4);
    });
    // put block
    queue.push(3);
    let mut i = 1;
    for num in queue.iter() {
        assert_eq!(i, *num);
        i += 1;
    }
}

#[test]
fn test_offer() {
    let queue = Arc::new(LinkedBlockingQueue::<i32>::with_capacity(2));
    assert_eq!(queue.offer(1), None);
    assert_eq!(queue.offer(2), None);
    assert_eq!(queue.offer(3), Some(3));

    let mut rs = queue.offer(3);
    while let Some(i) = rs {
        queue.take();
        rs = queue.offer(i);
    }
    assert_eq!(queue.poll(), Some(2));
    assert_eq!(queue.poll(), Some(3));
    assert_eq!(queue.poll(), None);
}

#[test]
fn single_read_write_test() {
    let queue0 = Arc::new(LinkedBlockingQueue::<i32>::with_capacity(10));
    let queue = Arc::clone(&queue0);
    let p1 = thread::spawn(move || {
        println!("hello thread 1");
        for i in 0..10000 {
            queue.push(i);
        }
    });
    //-----add consumer
    let queue = Arc::clone(&queue0);
    let c1 = thread::spawn(move || {
        println!("hello consumer1");
        let mut vec:Vec<i32> = Vec::new();
        loop {
            let num = queue.take();
            if num != -1 {
                vec.push(num);
            } else {
                break;
            }
        }
        return vec;
    });
    let queue = Arc::clone(&queue0);
    let (sc, rc) = mpsc::sync_channel(1);
    thread::spawn(move || {
        loop {
            if rc.try_recv().is_ok() {
                println!("this water shutdown");
                break
            }
            let len = queue.len();
            println!("this water {}", len);
            thread::sleep(Duration::from_millis(500));
        }
    });

    // 等待生产者完成
    p1.join().unwrap();
    // 发送终止信号，让消费者线程退出
    queue0.push(-1);
    // 等待消费者完成
    let v1 = c1.join().unwrap();
    assert_eq!(v1, (0..10000).collect::<Vec<_>>());
    sc.send(1);
}

#[ignore]
#[test]
fn bench_mark() {
    for _ in 0..10000 {
        single_read_write_test()
    }

    for _ in 0..10000 {
        mut_readwrite_test()
    }
}

#[test]
fn mut_readwrite_test() {
    let queue0 = Arc::new(LinkedBlockingQueue::<i32>::with_capacity(10));
    let queue = Arc::clone(&queue0);
    let p1 = thread::spawn(move || {
        // println!("hello thread 1");
        for i in 0..1000 {
            queue.push(i);
        }
    });
    let queue = Arc::clone(&queue0);
    let p2 = thread::spawn(move || {
        // println!("hello thread 2");
        for i in 1000..2000 {
            queue.push(i);
        }
    });
    let queue = Arc::clone(&queue0);
    let p3 = thread::spawn(move || {
        // println!("hello thread 3");
        for i in 2000..3000 {
            queue.push(i);
        }
    });
    //-----add consumer
    let queue = Arc::clone(&queue0);
    let c1 = thread::spawn(move || {
        // println!("hello consumer1");
        let mut vec:Vec<i32> = Vec::new();
        loop {
            let num = queue.take();
            if num != -1 {
                vec.push(num);
            } else {
                break;
            }
        }
        return vec;
    });
    //-- add consumer2
    let queue = Arc::clone(&queue0);
    let c2 = thread::spawn(move || {
        // println!("hello consumer1");
        let mut vec:Vec<i32> = Vec::new();
        loop {
            let num = queue.take();
            if num != -1 {
                vec.push(num);
            } else {
                break;
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