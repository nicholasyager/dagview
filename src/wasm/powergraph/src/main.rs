use powergraph::{Edge, Node, PowerGraph};

fn main() {
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
    simple_logger::SimpleLogger::new().env().init().unwrap();

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

    let nodes: Vec<Node> = vec![
        Node::new("a".to_string(), "foo".to_string()),
        Node::new("b".to_string(), "foo".to_string()),
        Node::new("c".to_string(), "foo".to_string()),
        Node::new("d".to_string(), "bar".to_string()),
        Node::new("e".to_string(), "bar".to_string()),
    ];

    let edges: Vec<Edge> = vec![
        Edge::new("a", "c"),
        Edge::new("a", "d"),
        Edge::new("a", "e"),
        Edge::new("b", "c"),
        Edge::new("b", "d"),
        Edge::new("b", "e"),
    ];

    let mut powergraph = PowerGraph::new(nodes, edges);
    powergraph.decompose();
}
