

use mutcrab::collection::map::{TreeHashMap as HashMap, Map};


#[test]
fn my_map2_test() {
    let mut map = HashMap::<i32, i32>::of(1, 2);
    map.put(2, 3);
    assert_eq!(map.get(&1), Some(&2));
    assert_eq!(map.remove(&2), Some(3));
    assert_eq!(map.get(&2), None);
    assert_eq!(map.put(1, 3), Some(2));
    map.put(2, 3);
    map.put(3, 4);
    assert_eq!(map.size(), 3);
    map.foreach(|k, v| {
        println!("forache key {} value is {}", *k, *v);
    });
}
