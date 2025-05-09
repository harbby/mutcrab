use mutcrab::collection::list::LinkedList;

#[test]
fn stack_test() {
    let mut list = LinkedList::<i32>::new();
    list.push(1);
    list.push(2);
    list.push(3);
    assert_eq!(list.size(), 3);
    assert_eq!(list.pop(), Some(3));
    assert_eq!(list.pop(), Some(2));
    assert_eq!(list.pop(), Some(1));
    assert_eq!(list.pop(), None);
}

#[test]
fn queue_test() {
    let mut list = LinkedList::<i32>::new();
    list.add(1);
    list.add(2);
    list.add(3);
    assert_eq!(list.size(), 3);
    assert_eq!(list.poll(), Some(1));
    assert_eq!(list.poll(), Some(2));
    assert_eq!(list.poll(), Some(3));
    assert_eq!(list.poll(), None);
}


#[test]
fn remove_test() {
    let mut list = LinkedList::<i32>::new();
    list.add(1);
    list.add(2);
    list.add(3);
    assert_eq!(list.size(), 3);
    assert_eq!(list.remove(&2), true);
    assert_eq!(list.size(), 2);
}

#[test]
fn peek_test() {
    let mut list = LinkedList::<i32>::new();
    list.add(1);
    list.add(2);
    assert_eq!(list.size(), 2);

    assert_eq!(list.peek_first(), Some(&mut 1));
    *list.peek_first().unwrap() = 111;
    assert_eq!(list.peek_first(), Some(&mut 111));
    assert_eq!(list.peek_last(), Some(&mut 2));
}

#[test]
fn foreach_test() {
    let mut list = LinkedList::<i32>::new();
    list.add(1);
    list.push(2);
    list.add(3);

    let mut arr = vec![1, 2, 3];
    let mut i = 0;
    list.foreach(|v| {
        arr[i] = *v;
        i += 1;
    });
    assert_eq!(arr, vec![2, 1, 3]);
    println!("{:?}", arr);
}

#[test]
fn iter_test() {
    let mut list = LinkedList::<&str>::new();
    list.add("a");
    list.add("b");
    list.add("c");
    list.add("d");
    for i in &list {
        println!("list iter: {}", i);
    }
    for i in list.iter() {
        println!("list iter: {}", i);
    }
    let vec:Vec<&str> = list.iter().map(|x|{*x}).collect();
    assert_eq!(vec, vec!["a", "b", "c", "d"]);
}
