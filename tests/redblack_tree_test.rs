use mutcrab::collection::tree::RBTree;

#[test]
fn test_tree_put() {
    let mut tree = RBTree::new();
    tree.put(1,1);
    tree.put(2,2);
    tree.put(3,3);
    tree.put(4,4);
    assert_eq!(tree.get(&100).map(|x|x), None);
    assert_eq!(tree.get(&3).map(|x| *x), Some(3));
}

#[test]
fn test_tree_iterator() {
    let mut tree = RBTree::new();
    tree.put(1,2);
    tree.put(2,2);
    tree.put(3,3);
    tree.put(4,4);
    let arr = tree.into_iter().map(|x| *x.0).collect::<Vec<i32>>();
    assert_eq!(arr, vec![2, 3, 4, 1]);
}

#[test]
fn test_tree_put2() {
    let mut tree = RBTree::new();
    let arr = [
        12, 23, 45, 34, 40, 67, 78, 89, 90, 100, 110, 120, 130, 140, 150, 160, 170, 180,
    ];
    for i in &arr {
        tree.put(i.clone(), i.clone());
    }
    let rs = tree.into_iter().map(|x| *x.0).collect::<Vec<i32>>();
    assert_eq!(
        rs,
        vec![89, 90, 100, 110, 120, 130, 140, 150, 160, 170, 180, 78, 67, 45, 40, 34, 23, 12]
    );
}
