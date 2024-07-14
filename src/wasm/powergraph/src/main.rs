use std::{collections::HashMap, fs::File, io::BufReader, process::exit};

use powergraph::{Edge, Node, PowerGraph};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
struct ManifestNode {
    unique_id: String,
    resource_type: String,
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    nodes: HashMap<String, ManifestNode>,
    child_map: HashMap<String, Vec<String>>,
    parent_map: HashMap<String, Vec<String>>,
}

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    let f = File::open("manifest.big.json").unwrap();
    let reader = BufReader::new(f);

    let v: Manifest = serde_json::from_reader(reader).unwrap();

    let nodes: Vec<Node> = v
        .nodes
        .into_iter()
        .filter(|(_, node)| node.resource_type != "test")
        .map(|(_, node)| Node::new(node.unique_id, "".to_string()))
        .collect();
    // println!("{:?}", v["nodes"]);

    let mut edges: Vec<Edge> = v
        .child_map
        .iter()
        .flat_map(|(parent, children)| {
            children
                .into_iter()
                .map(|child| Edge::new(parent, child))
                .collect::<Vec<Edge>>()
        })
        .collect();

    let parent_map_edges: Vec<Edge> = v
        .parent_map
        .iter()
        .flat_map(|(child, parents)| {
            parents
                .into_iter()
                .map(|parent| Edge::new(parent, child))
                .collect::<Vec<Edge>>()
        })
        .collect();

    edges.extend(parent_map_edges);

    edges = edges
        .into_iter()
        .filter(|edge| !edge.get_from().starts_with("test.") && !edge.get_to().starts_with("test."))
        .collect();

    // let nodes: Vec<Node> = vec![
    //     Node::new("u".to_string(), "foo".to_string()),
    //     Node::new("v".to_string(), "foo".to_string()),
    //     Node::new("y".to_string(), "foo".to_string()),
    //     Node::new("w".to_string(), "bar".to_string()),
    //     Node::new("s".to_string(), "baz".to_string()),
    //     Node::new("x".to_string(), "fizz".to_string()),
    //     Node::new("t".to_string(), "Boo!".to_string()),
    //     Node::new("z".to_string(), "Boo!".to_string()),
    // ];

    // let edges: Vec<Edge> = vec![
    //     Edge::new("v", "u"),
    //     Edge::new("v", "w"),
    //     Edge::new("u", "w"),
    //     Edge::new("u", "x"),
    //     Edge::new("u", "z"),
    //     Edge::new("y", "s"),
    //     Edge::new("y", "t"),
    //     Edge::new("y", "w"),
    //     Edge::new("w", "s"),
    //     Edge::new("w", "t"),
    //     Edge::new("s", "x"),
    //     Edge::new("s", "z"),
    //     Edge::new("x", "t"),
    //     Edge::new("x", "z"),
    //     Edge::new("t", "z"),
    // ];

    // let nodes: Vec<Node> = vec![
    //     Node::new("a".to_string(), "foo".to_string()),
    //     Node::new("b".to_string(), "foo".to_string()),
    //     Node::new("c".to_string(), "foo".to_string()),
    //     Node::new("d".to_string(), "bar".to_string()),
    // ];

    // let edges: Vec<Edge> = vec![
    //     Edge::new("a", "b"),
    //     Edge::new("a", "c"),
    //     Edge::new("b", "c"),
    //     Edge::new("b", "d"),
    // ];

    // let nodes: Vec<Node> = vec![
    //     Node::new("a".to_string(), "foo".to_string()),
    //     Node::new("b".to_string(), "foo".to_string()),
    //     Node::new("c".to_string(), "foo".to_string()),
    //     Node::new("d".to_string(), "bar".to_string()),
    //     Node::new("e".to_string(), "bar".to_string()),
    // ];

    // let edges: Vec<Edge> = vec![
    //     Edge::new("a", "b"),
    //     Edge::new("a", "c"),
    //     Edge::new("b", "d"),
    //     Edge::new("c", "d"),
    //     Edge::new("d", "e"),
    // ];

    // let nodes: Vec<Node> = vec![
    //     Node::new("a".to_string(), "foo".to_string()),
    //     Node::new("b".to_string(), "foo".to_string()),
    //     Node::new("c".to_string(), "foo".to_string()),
    //     Node::new("d".to_string(), "bar".to_string()),
    //     Node::new("e".to_string(), "bar".to_string()),
    // ];

    // let edges: Vec<Edge> = vec![
    //     Edge::new("a", "c"),
    //     Edge::new("a", "d"),
    //     Edge::new("a", "e"),
    //     Edge::new("b", "c"),
    //     Edge::new("b", "d"),
    //     Edge::new("b", "e"),
    // ];

    let mut powergraph = PowerGraph::new(nodes, edges);
    powergraph.decompose();
}
