use powergraph::{Edge, Node, PowerGraph};

fn main() {
    let nodes: Vec<Node> = vec![
        Node::new("1".to_string(), "foo".to_string()),
        Node::new("2".to_string(), "foo".to_string()),
        Node::new("3".to_string(), "foo".to_string()),
        Node::new("4".to_string(), "bar".to_string()),
        Node::new("5".to_string(), "baz".to_string()),
        Node::new("6".to_string(), "fizz".to_string()),
        Node::new("7".to_string(), "Boo!".to_string()),
        Node::new("8".to_string(), "Boo!".to_string()),
    ];

    let edges: Vec<Edge> = vec![
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

    let mut powergraph = PowerGraph::new(nodes, edges);
    powergraph.decompose();
}
