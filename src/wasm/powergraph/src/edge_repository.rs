use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::{sets::Set, Edge};

#[derive(Serialize, Clone)]
pub struct EdgeRepository {
    child_map: HashMap<String, HashSet<String>>,
    parent_map: HashMap<String, HashSet<String>>,
}

impl IntoIterator for EdgeRepository {
    type Item = Edge;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let values: Vec<Edge> = self
            .child_map
            .iter()
            .flat_map(|(source, targets)| {
                targets.iter().map(|target| Edge {
                    from: source.clone(),
                    to: target.clone(),
                })
            })
            .collect();
        values.into_iter()
    }
}

impl EdgeRepository {
    pub fn new() -> EdgeRepository {
        let child_map = HashMap::new();
        let parent_map = HashMap::new();
        EdgeRepository {
            child_map,
            parent_map,
        }
    }

    pub fn from_edge_list(edges: Vec<Edge>) -> EdgeRepository {
        let mut edge_repository = EdgeRepository::new();

        for edge in edges {
            edge_repository.add_edge(&edge)
        }

        return edge_repository;
    }

    pub fn add_edge(&mut self, edge: &Edge) {
        let from_set = self
            .child_map
            .entry(edge.from.clone())
            .or_insert(HashSet::new());
        from_set.insert(edge.to.clone());

        let to_set = self
            .parent_map
            .entry(edge.to.clone())
            .or_insert(HashSet::new());
        to_set.insert(edge.from.clone());
    }

    pub fn get_edge(&self, from: &String, to: &String) -> Option<Edge> {
        match self.child_map.get(from) {
            Some(targets) => match targets.get(to) {
                Some(_) => {
                    return Some(Edge {
                        from: from.clone(),
                        to: to.clone(),
                    })
                }
                None => return None,
            },
            None => return None,
        }
    }

    pub fn subgraph(&self, nodes: &Set<String>) -> Vec<Edge> {
        let mut output: Vec<Edge> = Vec::new();

        let source_keys =
            Set::from_iter(self.child_map.clone().into_keys().collect::<Vec<String>>());

        let source_nodes = source_keys.intersection(nodes);

        for source in source_nodes.iter() {
            let targets = self.child_map.get(&source);

            match targets {
                None => continue,
                Some(target_set) => {
                    let common_targets = Set::from_set(target_set.clone()).intersection(nodes);
                    for target in common_targets {
                        output.push(Edge {
                            from: source.clone(),
                            to: target,
                        })
                    }
                }
            }
        }

        return output;
    }

    pub fn parents(&self, node: &String) -> Set<String> {
        match self.parent_map.get(node) {
            Some(parents) => {
                return Set::from_set(parents.clone());
            }
            None => {
                return Set::new();
            }
        }
    }

    pub fn children(&self, node: &String) -> Set<String> {
        match self.child_map.get(node) {
            Some(children) => {
                return Set::from_set(children.clone());
            }
            None => {
                return Set::new();
            }
        }
    }

    pub fn len(self) -> usize {
        self.into_iter().count()
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn subgraph_fetch_success() {
        let edges: Vec<Edge> = vec![
            Edge::new("v", "u"),
            Edge::new("v", "w"),
            Edge::new("u", "w"),
            Edge::new("u", "x"),
            Edge::new("u", "z"),
            Edge::new("y", "s"),
            Edge::new("y", "t"),
            Edge::new("y", "w"),
            Edge::new("w", "s"),
            Edge::new("w", "t"),
            Edge::new("s", "x"),
            Edge::new("s", "z"),
            Edge::new("x", "t"),
            Edge::new("x", "z"),
            Edge::new("t", "z"),
        ];
        let edge_repository = EdgeRepository::from_edge_list(edges);

        let search_nodes = Set::from_iter(vec!["s".to_string(), "t".to_string(), "y".to_string()]);

        let subgraph = edge_repository.subgraph(&search_nodes);

        let subgraph_sorted: Vec<Edge> = subgraph
            .into_iter()
            .sorted_by(|a, b| Ord::cmp(&a.get_id(), &b.get_id()))
            .collect();

        assert_eq!(
            subgraph_sorted,
            vec![Edge::new("y", "s"), Edge::new("y", "t"),]
        )
    }
}
