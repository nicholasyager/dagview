mod utils;

use std::{collections::HashSet, hash::Hash};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(value: &str) {
    alert(&format!("Hello, {}!", &value));
}

type NodeId = String;
#[wasm_bindgen]
pub struct Node {
    id: NodeId,
    data: String,
}

#[wasm_bindgen]
impl Node {
    #[wasm_bindgen]
    pub fn new(id: NodeId, data: String) -> Node {
        Node { id, data }
    }
}

#[wasm_bindgen]
pub struct Edge {
    from: NodeId,
    to: NodeId,
}

#[wasm_bindgen]
impl Edge {
    #[wasm_bindgen]
    pub fn new(from: &str, to: &str) -> Edge {
        Edge {
            from: from.to_string(),
            to: to.to_string(),
        }
    }
}

#[wasm_bindgen]
pub struct Cluster {
    nodes: HashSet<u32>,
}

type PowerNodeId = String;

#[wasm_bindgen]
pub struct PowerNode {
    id: PowerNodeId,
    nodes: Vec<Cluster>,
}

#[wasm_bindgen]
pub struct PowerEdge {
    from: PowerNodeId,
    to: PowerNodeId,
}

#[wasm_bindgen]
struct SimilarityMatrix {
    matrix: Vec<Vec<f32>>,
    inactive_clusters: HashSet<usize>,
}

#[wasm_bindgen]
impl SimilarityMatrix {
    #[wasm_bindgen(constructor)]
    pub fn new(clusters: Vec<Cluster>) -> SimilarityMatrix {
        let mut matrix: Vec<Vec<f32>> = Vec::new();
        let size = clusters.len();

        for _ in clusters {
            let mut new_vector = vec![0.0_f32; size];
            matrix.push(new_vector);
        }

        SimilarityMatrix {
            matrix: matrix,
            inactive_clusters: HashSet::new(),
        }
    }
}

#[wasm_bindgen]
pub struct PowerGraph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    power_nodes: Vec<PowerNode>,
    power_edges: Vec<PowerEdge>,
}

#[wasm_bindgen]
impl PowerGraph {
    #[wasm_bindgen(constructor)]
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> PowerGraph {
        PowerGraph {
            nodes,
            edges,
            power_edges: Vec::new(),
            power_nodes: Vec::new(),
        }
    }

    #[wasm_bindgen]
    pub fn deconstruct() {
        let c: Vec<Cluster> = Vec::new();
        let c_prime: Vec<Cluster> = Vec::new();
        let similarity_matrix: Vec<Vec<usize>> = Vec::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn powergraph_construction() {
        let nodes = (1..9)
            .map(|id| Node::new(id.to_string(), String::from("")))
            .collect::<Vec<Node>>();

        let edges = vec![
            Edge::new("1", "2"),
            Edge::new("1", "4"),
            Edge::new("2", "3"),
            Edge::new("2", "4"),
            Edge::new("2", "6"),
            Edge::new("2", "8"),
            Edge::new("3", "4"),
            Edge::new("3", "5"),
            Edge::new("3", "7"),
            Edge::new("4", "5"),
            Edge::new("4", "7"),
            Edge::new("5", "6"),
            Edge::new("5", "8"),
            Edge::new("6", "7"),
            Edge::new("6", "8"),
        ];

        let power_graph = PowerGraph::new(nodes, edges);

        assert_eq!(power_graph.nodes.len(), 8);
        assert_eq!(power_graph.edges.len(), 15);
    }
}
