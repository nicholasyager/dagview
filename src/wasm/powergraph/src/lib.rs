mod utils;

use std::collections::HashSet;

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
    pub fn new(from: String, to: String) -> Edge {
        Edge { from, to }
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
    pub fn deconstruct() {}
}
