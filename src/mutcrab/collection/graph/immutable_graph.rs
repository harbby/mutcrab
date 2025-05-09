#![allow(unused)]

use crate::collection::list::LinkedList;
use crate::collection::map::Map;
use crate::collection::map::{Entry, TreeHashMap as HashMap};
use std::hash::Hash;

pub struct ImmutableGraph<N> {
    nodeMap: HashMap<N, NodeIndex<N>>,
    edgeMap: HashMap<NodeIndex<N>, LinkedList<NodeIndex<N>>>,
    rootMap: HashMap<NodeIndex<N>, bool>
}

#[derive(PartialEq, Eq, Hash)]
struct NodeIndex<N> {
    index: u32,
    ptr: *const N,
}

impl<N> Clone for NodeIndex<N> {
    fn clone(&self) -> Self {
        NodeIndex {
            index: self.index,
            ptr: self.ptr,
        }
    }
}

impl<N> Copy for NodeIndex<N> {}


pub struct Builder<N> {
    index: u32,
    nodeMap: HashMap<N, NodeIndex<N>>,
    edgeMap: HashMap<NodeIndex<N>, LinkedList<NodeIndex<N>>>,
    rootMap: HashMap<NodeIndex<N>, bool>
}

impl <N> Builder<N>
where
    N: Eq + Hash
{
    fn new() -> Self {
        Builder { index: 0,
            nodeMap: HashMap::new(),
            edgeMap: HashMap::new(),
            rootMap: HashMap::new()
        }
    }

    fn add_node(&mut self, value: N) -> NodeIndex<N>
    {
        match self.nodeMap.entry(value) {
            Entry::Occupied(x) => { *x.value() }
            Entry::Vacant(entry) => {
                let rs = entry.put_if_absent(|_| {
                    NodeIndex { index: self.index, ptr: std::ptr::null() }
                });
                let k_ref = rs.key();
                let v_ref = rs.value();
                v_ref.ptr = k_ref;
                self.index += 1;

                // self.edgeMap.put(new_index, LinkedList::new());
                self.rootMap.put(v_ref.clone(), true);
                return v_ref.clone()
            }
        }
    }

    fn add_edge(&mut self, src: NodeIndex<N>, dst: NodeIndex<N>) -> &mut Builder<N> {
        let map = &mut self.edgeMap;
        let rs = map.get_mut(&src);
        if let Some(x) = rs {
            x.add(dst)
        } else {
            let mut list = LinkedList::<NodeIndex<N>>::new();
            list.add(dst);
            map.put(src, list);
        }
        self.rootMap.remove(&dst);
        return self;
    }

    fn build(self) -> ImmutableGraph<N> {
        // let mut node_list: Vec<*const N> = Vec::<*const N>::with_capacity(self.nodeMap.len() as usize);
        // unsafe { node_list.set_len(self.nodeMap.len() as usize) };
        // for (k,v) in &self.nodeMap {
        //     node_list[v.index as usize] = k as *const N;
        // }
        ImmutableGraph {
            nodeMap: self.nodeMap,
            edgeMap: self.edgeMap,
            rootMap: self.rootMap
        }
    }
}

impl<N> ImmutableGraph<N>
where N: Eq + Hash,
{
    pub fn builder() -> Builder< N> {
        Builder::new()
    }

    pub fn contains_node(&self, node: &N) -> bool {
        return self.nodeMap.contains_key(node);
    }

    pub fn size(&self) -> usize {
        self.nodeMap.size()
    }
}

#[test]
fn test1() {
    let mut builder = ImmutableGraph::<&str>::builder();
    let a = builder.add_node("a");
    let b = builder.add_node("b");
    let c = builder.add_node("c");
    let d = builder.add_node("d");
    builder.add_edge(a, b)
        .add_edge(b, c)
        .add_edge(b, d);

    unsafe  {
        println!("{}", *a.ptr);
        println!("{}", *b.ptr);
    }

    let graph = builder.build();
    assert_eq!(graph.size(), 4);
    assert_eq!(graph.contains_node(&"a"), true);
    assert_eq!(graph.contains_node(&"aaa"), false);
}
