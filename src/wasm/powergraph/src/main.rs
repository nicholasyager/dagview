use powergraph::{Edge, Node, PowerGraph};

fn main() {
    let nodes: Vec<Node> = vec![
        Node::new("u".to_string(), "foo".to_string()),
        Node::new("v".to_string(), "foo".to_string()),
        Node::new("y".to_string(), "foo".to_string()),
        Node::new("w".to_string(), "bar".to_string()),
        Node::new("s".to_string(), "baz".to_string()),
        Node::new("x".to_string(), "fizz".to_string()),
        Node::new("t".to_string(), "Boo!".to_string()),
        Node::new("z".to_string(), "Boo!".to_string()),
    ];

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

    let mut powergraph = PowerGraph::new(nodes, edges);
    powergraph.decompose();
}