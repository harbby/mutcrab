// mod common;
//
// use mutcrab::collection::map::{Map};
//
// #[test]
// fn std_map_test() {
//     use std::collections::HashMap;
//     let mut map = HashMap::<i32, i32>::new();
//     map.insert(1, 2);
//     let entry = map.entry(2);
//     entry.or_insert(3);
//
//     let v: &i32 = map.get(&1).unwrap();
//     println!("key 1 value is {}", v);
//     println!("{:?}", map);
// }
//
// #[test]
// fn my_map_test() {
//     use mutcrab::collection::map::BoxHashMap as HashMap;
//     let mut map = HashMap::<i32, i32>::new();
//     map.put(1, 1);
//     map.put(1, 2);
//     map.put(2, 3);
//     assert_eq!(map.get(&1), Some(&2));
//     println!("key 1 value is {}", *map.get(&1).unwrap());
//     map.foreach(|k, v| {
//         println!("boxmap foreach key {} value is {}", *k, *v);
//     });
// }
//
// #[test]
// fn remove_test() {
//     use mutcrab::collection::map::BoxHashMap as HashMap;
//     let mut map = HashMap::<&str, i32>::new();
//     map.insert("a", 1);
//     map.insert("b", 2);
//     map.insert("c", 3);
//     map.insert("d", 4);
//     assert_eq!(map.len(), 4);
//     assert_eq!(map.remove(&"a"), Some(1));
//     assert_eq!(map.len(), 3);
//     assert_eq!(map.remove(&"NotKey"), None);
//     assert_eq!(map.len(), 3);
//     assert_eq!(map.remove(&"b"), Some(2));
//     assert_eq!(map.len(), 2);
//     assert_eq!(map.remove(&"c"), Some(3));
//     assert_eq!(map.len(), 1);
//     assert_eq!(map.remove(&"d"), Some(4));
//     assert_eq!(map.len(), 0);
// }