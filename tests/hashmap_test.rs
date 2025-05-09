use mutcrab::collection::map::{HashMap, Map};
use std::hash::BuildHasherDefault;

#[test]
fn my_map2_test() {

    let mut map = HashMap::<i32, i32>::of(1, 2);
    map.put(2, 3);
    assert_eq!(map.get(&1), Some(&2));
    assert_eq!(map.remove(&2), Some(3));
    assert_eq!(map.get(&2), None);
    assert_eq!(map.put(1, 2), Some(2));
    map.put(2, 3);
    map.put(3, 4);
    assert_eq!(map.size(), 3);
    map.foreach(|k, v| {
        println!("forache key {} value is {}", *k, *v);
    });
}

#[test]
fn hashmap_test() {
    let mut map = HashMap::<&str, bool>::new();
    map.insert("a", true);
    map.insert("b", false);
    map.insert("c", false);
    map.insert("d", false);
    assert_eq!(map.len(), 4);
    map.foreach(|k, v| {
        println!("hashmap key {} value is {}", *k, *v);
    });
    assert_eq!(map.get(&"a"), Some(&true));
    assert_eq!(map.get(&"b"), Some(&false));
    assert_eq!(map.get(&"c"), Some(&false));
    assert_eq!(map.get(&"d"), Some(&false));
    assert_eq!(map.contains_key(&"a"), true);
}

#[test]
fn remove_test() {
    let mut map = HashMap::<&str, i32>::new();
    map.insert("a", 1);
    map.insert("b", 2);
    map.insert("c", 3);
    map.insert("d", 4);
    assert_eq!(map.len(), 4);
    assert_eq!(map.remove(&"a"), Some(1));
    assert_eq!(map.len(), 3);
    assert_eq!(map.remove(&"b"), Some(2));
    assert_eq!(map.len(), 2);
    assert_eq!(map.remove(&"c"), Some(3));
    assert_eq!(map.len(), 1);
    assert_eq!(map.remove(&"d"), Some(4));
    assert_eq!(map.len(), 0);
}

#[test]
fn entry_get_test() {
    let mut map = HashMap::<&str, i32>::new();
    map.insert("a", 1);
    map.insert("b", 2);
    map.insert("c", 1);
    map.insert("d", 1);
    assert_eq!(map.len(), 4);
    let entry = map.entry("a");
    assert_eq!(*entry.key(), "a");

    assert_eq!(map.entry("a").take_insert(2), Some(1));
    assert_eq!(map.get(&"a"), Some(&2));
    assert_eq!(*map.entry("a").or_insert(2), 2);

    let v = map.entry("b").and_modify(|x| *x = *x * 10 + 2).value();
    assert_eq!(*v, 22);
}


#[test]
fn entry_insert_test() {
    let mut map = HashMap::<&str, i32>::new();
    map.entry("a").or_insert(1);
    map.entry("b").and_modify(|x| *x = 2).value();
    assert_eq!(map.get(&"a"), Some(&1));
    assert_eq!(map.get(&"b"), Some(&2));
}

#[test]
fn map_iterator_test() {
    let mut map = HashMap::<&str, i32>::new();
    map.put("a", 1);
    map.put("b", 2);
    map.put("c", 3);
    map.put("d", 4);

    for (k, v) in &map {
        println!("map iterator key {} value is {}", *k, *v);
    }

    map.iter().for_each(|(k, v)| {
        println!("map iterator key {} value is {}", *k, *v);
    });

    let key_list: Vec<_> = map.iter().map(|(k, _)| {*k}).collect();
    println!("key list: {:?}", key_list);
    let value_list: Vec<_> = map.iter().map(|(_, v)| {*v}).collect();
    println!("value list: {:?}", value_list);
}

#[test]
fn test_modify_value_iterator() {
    let mut map = HashMap::<&str, i32>::new();
    map.put("a", 1);
    map.put("b", 2);
    map.put("c", 3);
    map.put("d", 4);
    for (k, v) in &map {
        *v += 1;
    }
    assert_eq!(map.get(&"a"), Some(&2));
    assert_eq!(map.get(&"b"), Some(&3));
    assert_eq!(map.get(&"c"), Some(&4));
    assert_eq!(map.get(&"d"), Some(&5));
}

#[derive(Default)]
struct MyIntegerHasher(u64);

impl std::hash::Hasher for MyIntegerHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0 = match bytes.len() {
            4 => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64,
            8 => u64::from_le_bytes(bytes.try_into().unwrap()),
            _ => panic!("Invalid byte length: {}. Only 4 or 8 byte lengths are supported.", bytes.len()),
        }
    }
}

#[test]
fn test_hasher_test() {
    type BuildHasher = BuildHasherDefault<MyIntegerHasher>;
    let mut map:HashMap<i32, &str, BuildHasher> = HashMap::with_hasher(BuildHasher::new());
    map.insert(1, "a");
    map.insert(2, "b");
    map.insert(3, "c");
    assert_eq!(map.get(&1), Some(&"a"));
    assert_eq!(map.get(&2), Some(&"b"));
    assert_eq!(map.get(&3), Some(&"c"));
}
