mod clusters;
mod sets;
mod similarity_matrix;
mod unordered_tuple;
mod utils;

use clusters::Cluster;
use sets::Set;
use similarity_matrix::SimilarityMatrix;
use unordered_tuple::UnorderedTuple;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    // #[wasm_bindgen(js_namespace = console)]
    // fn log(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub fn greet(value: &str) {
    alert(&format!("Hello, {}!", &value));
}

type NodeId = String;
#[wasm_bindgen]
#[derive(Debug, Clone)]
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
#[derive(Debug)]
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

type PowerNodeId = String;

#[wasm_bindgen]
#[derive(Debug)]
pub struct PowerNode {
    id: PowerNodeId,
    nodes: Vec<Cluster>,
}

#[wasm_bindgen]
#[derive(Debug)]
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

    fn get_node(&self, node_id: &NodeId) -> Option<Node> {
        for node in self.nodes.iter() {
            if node.id == *node_id {
                return Some(node.clone());
            }
        }

        println!("Unable to find {:?} in {:?}", node_id, self.nodes);

        return None;
    }

    fn node_id_to_index(&self) {}

    /// For a given Cluster, find all node names for sibling nodes.
    fn find_siblings(&self, cluster: &Cluster) -> Set<String> {
        let mut output = Set::from_iter(Vec::new());

        // Find all children of parents. This will be the selected cluster's generation.

        for neighbor_id in cluster.get_neighbors() {
            let neighbor = self.get_node(&neighbor_id).unwrap();

            let children: Vec<String> = self
                .edges
                .iter()
                .filter_map(|edge| {
                    if edge.from == neighbor.id {
                        return Some(edge.to.clone());
                    }

                    None
                })
                .collect();

            for child in children {
                output.insert(child);
            }
        }

        let cluster_set = Set::from_iter(cluster.get_items());

        // Return the list.
        return (output.difference(&cluster_set)).clone();
    }

    fn neighbors(&self, node_id: &NodeId) -> Set<String> {
        let node_ids = self
            .edges
            .iter()
            .filter_map(|edge| {
                if edge.from == *node_id {
                    return Some(edge.to.clone());
                } else if edge.to == *node_id {
                    return Some(edge.from.clone());
                }
                None
            })
            .collect();

        Set::from_iter(node_ids)
    }

    #[wasm_bindgen]
    pub fn decompose(&mut self) {
        let mut c: Vec<Cluster> = Vec::new();
        let mut c_prime: Vec<Cluster> = Vec::new();

        // Add all nodes to c and c_prime as singleton clusters.
        for (index, node) in (&self.nodes).into_iter().enumerate() {
            println!("Node: {:?}", node);
            // console_log!("Node: {:?}", node);

            let cluster_nodes = Cluster::new(
                Set::from_iter(vec![node.id.clone()]),
                Set::from_iter(
                    self.neighbors(&node.id)
                        .to_vec()
                        .into_iter()
                        .map(|item| item.to_string())
                        .collect(),
                ),
            );

            c.push(cluster_nodes);
        }
        c_prime = c.clone();

        let mut similarity_matrix = SimilarityMatrix::new();

        let comparison_sets: Set<UnorderedTuple> = clusters::generate_comparison_set(&c_prime);

        for comparison_set in comparison_sets.to_vec() {
            let cluster_index = c
                .iter()
                .position(|item| item.get_id() == comparison_set.one)
                .unwrap();

            let cluster = c[cluster_index].clone();

            let comparison_cluster_index = c
                .iter()
                .position(|item| item.get_id() == comparison_set.two)
                .unwrap();
            let comparison_cluster = c[comparison_cluster_index].clone();

            let similarity = cluster.similarity(&comparison_cluster);
            similarity_matrix.set_similarity(
                UnorderedTuple {
                    one: comparison_set.one,
                    two: comparison_set.two,
                },
                similarity,
            );
        }

        // Find the two clusters with maximum similarity
        let mut max_similarity = similarity_matrix.get_max_similarity();

        while c_prime.len() > 0 && max_similarity.1 >= 0.5_f32 {
            println!("Max similarity: {:?}", max_similarity);
            println!("{:?}", c);
            println!("{:?}", c_prime);

            let cluster_index = c_prime
                .iter()
                .position(|item| item.get_id() == max_similarity.0.one)
                .unwrap();

            let cluster = c_prime[cluster_index].clone();

            let comparison_cluster_index = c_prime
                .iter()
                .position(|item| item.get_id() == max_similarity.0.two)
                .unwrap();
            let comparison_cluster = c_prime[comparison_cluster_index].clone();

            println!("{:?} <-> {:?}", cluster, comparison_cluster);

            let mut remove_list = vec![cluster_index, comparison_cluster_index];
            remove_list.sort();
            remove_list.reverse();

            for item in remove_list {
                c_prime.remove(item);
            }

            similarity_matrix.remove_element(cluster.get_id());
            similarity_matrix.remove_element(comparison_cluster.get_id());

            let unioned_cluster = cluster.union(&comparison_cluster);

            // Add new cluster to everything!
            c.push(unioned_cluster.clone());
            c_prime.push(unioned_cluster.clone());

            // Calculate new similarities for the added element.
            let cluster_parents = Set::from_iter(unioned_cluster.get_neighbors());
            for comparison_cluster in c_prime.iter() {
                if unioned_cluster == *comparison_cluster {
                    continue;
                }

                let comparison_cluster_parents = Set::from_iter(comparison_cluster.get_neighbors());

                if cluster_parents
                    .intersection(&comparison_cluster_parents)
                    .len()
                    == 0
                {
                    continue;
                }
                let similarity = unioned_cluster.similarity(&comparison_cluster);
                similarity_matrix.set_similarity(
                    UnorderedTuple {
                        one: unioned_cluster.get_id(),
                        two: comparison_cluster.get_id(),
                    },
                    similarity,
                );
            }

            println!("{:?}", similarity_matrix);
            max_similarity = similarity_matrix.get_max_similarity();
        }

        println!("{:?}", c_prime);
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

    #[test]
    fn find_siblings_trivial_case() {
        let nodes: Vec<Node> = vec![
            Node::new("parent".to_string(), "foo".to_string()),
            Node::new("child".to_string(), "bar".to_string()),
            Node::new("sibling".to_string(), "baz".to_string()),
            Node::new("sibling2".to_string(), "fizz".to_string()),
            Node::new("child2".to_string(), "Boo!".to_string()),
        ];

        let edges: Vec<Edge> = vec![
            Edge::new("parent", "child"),
            Edge::new("parent", "sibling"),
            Edge::new("parent", "sibling2"),
            Edge::new("child", "child2"),
        ];

        let powergraph = PowerGraph::new(nodes, edges);
        let cluster = Cluster::new(
            Set::from_iter(vec!["child".to_string()]),
            powergraph.neighbors(&"child".to_string()),
        );
        let siblings = powergraph.find_siblings(&cluster);

        assert_eq!(
            siblings,
            Set::from_iter(vec!["sibling".to_string(), "sibling2".to_string()])
        );
    }

    #[test]
    fn find_siblings_non_trivial_case() {
        let nodes: Vec<Node> = vec![
            Node::new("parent1".to_string(), "foo".to_string()),
            Node::new("parent2".to_string(), "foo".to_string()),
            Node::new("parent3".to_string(), "foo".to_string()),
            Node::new("child".to_string(), "bar".to_string()),
            Node::new("sibling".to_string(), "baz".to_string()),
            Node::new("sibling2".to_string(), "fizz".to_string()),
            Node::new("child2".to_string(), "Boo!".to_string()),
        ];

        let edges: Vec<Edge> = vec![
            Edge::new("parent1", "child"),
            Edge::new("parent2", "child"),
            Edge::new("parent3", "child"),
            Edge::new("parent2", "sibling"),
            Edge::new("parent3", "sibling2"),
            Edge::new("child", "child2"),
        ];

        let powergraph = PowerGraph::new(nodes, edges);
        let cluster = Cluster::new(
            Set::from_iter(vec!["child".to_string()]),
            powergraph.neighbors(&"child".to_string()),
        );
        let siblings = powergraph.find_siblings(&cluster);

        assert_eq!(
            siblings,
            Set::from_iter(vec!["sibling".to_string(), "sibling2".to_string()])
        );
    }

    // #[test]
    fn decompose() {
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
}
