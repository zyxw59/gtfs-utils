use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    ops::Deref,
};

use derivative::Derivative;

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Copy(bound = ""),
    Debug = "transparent",
    PartialEq(bound = ""),
    PartialOrd(bound = ""),
    Eq(bound = ""),
    Ord(bound = "")
)]
pub struct PtrKey<T>(*const T);

impl<P, T> From<&P> for PtrKey<T>
where
    P: Deref<Target = T>,
{
    fn from(ptr: &P) -> PtrKey<T> {
        PtrKey(ptr.deref() as *const T)
    }
}

#[derive(Debug)]
pub struct Dag<K, V> {
    nodes: BTreeMap<K, Node<K, V>>,
}

impl<K, V> Dag<K, V>
where
    K: for<'r> From<&'r V> + Ord + Copy + std::fmt::Debug,
    V: std::fmt::Debug,
{
    pub fn new() -> Self {
        Dag {
            nodes: BTreeMap::new(),
        }
    }

    pub fn insert_child(&mut self, parent: Option<V>, child: V) -> anyhow::Result<()> {
        let idx = K::from(&child);
        let node = self.nodes.entry(idx).or_insert_with(|| Node::new(child));
        if let Some(parent) = parent {
            let parent = K::from(&parent);
            node.add_parent(parent);
            self.nodes
                .get_mut(&parent)
                .ok_or_else(|| anyhow::anyhow!("parent node not found"))?
                .add_child(idx);
        }
        Ok(())
    }

    pub fn flatten(self) -> anyhow::Result<Vec<V>> {
        // all nodes with no parents
        let mut heads = Vec::new();
        // all other nodes
        let mut tails = BTreeMap::new();
        // initialize heads and tails
        for (idx, node) in self.nodes {
            if node.parents.is_empty() {
                heads.push((idx, node));
            } else {
                tails.insert(idx, node);
            }
        }
        log::debug!("{} heads; {} tails", heads.len(), tails.len());

        let mut output = Vec::new();
        while let Some((idx, node)) = heads.pop() {
            output.push(node.value);
            for ch_idx in node.children {
                match tails.entry(ch_idx) {
                    Entry::Occupied(mut entry) => {
                        if !entry.get_mut().remove_parent(idx) {
                            panic!("child node {:?} missing parent {:?}", ch_idx, idx);
                        }
                        if entry.get().parents.is_empty() {
                            heads.push(entry.remove_entry());
                        }
                    }
                    Entry::Vacant(_) => panic!("failed to find {:?}", ch_idx),
                }
            }
        }

        if tails.is_empty() {
            // successfully processed all nodes
            Ok(output)
        } else {
            // some nodes were never processed, therefore there was a cycle
            Err(anyhow::anyhow!("Cycle in graph"))
        }
    }
}

#[derive(Debug)]
struct Node<K, V> {
    value: V,
    parents: BTreeSet<K>,
    children: BTreeSet<K>,
}

impl<K, V> Node<K, V>
where
    K: Ord,
{
    fn new(value: V) -> Self {
        Node {
            value,
            parents: BTreeSet::new(),
            children: BTreeSet::new(),
        }
    }

    fn add_parent(&mut self, parent: K) {
        self.parents.insert(parent);
    }

    fn remove_parent(&mut self, parent: K) -> bool {
        self.parents.remove(&parent)
    }

    fn add_child(&mut self, child: K) {
        self.children.insert(child);
    }
}
