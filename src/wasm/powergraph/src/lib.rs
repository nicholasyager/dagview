mod clusters;
mod sets;
mod similarity_matrix;
mod utils;

use clusters::Cluster;
use sets::Set;
use similarity_matrix::SimilarityMatrix;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
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
#[derive(Debug)]
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

    fn node_id_to_index(&self) {}

    /// For a given Cluster, find all node names for sibling nodes.
    fn find_siblings(&self, cluster: &Cluster) -> Set<&str> {
        let mut output = Set::from_iter(Vec::new());

        // Find all children of parents. This will be the selected cluster's generation.

        for parent_index in cluster.get_parents() {
            let parent = &self.nodes[parent_index];

            let children: Vec<&str> = self
                .edges
                .iter()
                .filter_map(|edge| {
                    if edge.from == parent.id {
                        return Some(edge.to.as_str());
                    }

                    None
                })
                .collect();

            for child in children {
                output.insert(child);
            }
        }

        let cluster_set = Set::from_iter(
            cluster
                .get_items()
                .iter()
                .map(|node_index| {
                    let node = self.nodes.get(*node_index).unwrap();
                    return node.id.as_str();
                })
                .collect(),
        );

        // Return the list.
        return output.difference(&cluster_set);
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
                vec![index],
                self.edges
                    .iter()
                    .filter_map(|edge| {
                        if edge.to == node.id {
                            let index = self.nodes.iter().position(|r| r.id == edge.from).unwrap();
                            return Some(index);
                        }

                        None
                    })
                    .collect(),
            );

            c.push(cluster_nodes);
        }
        c_prime = c.clone();

        let mut similarity_matrix = SimilarityMatrix::new();

        let comparison_sets: Set<(usize, usize)> = clusters::generate_comparison_set(&c_prime);

        for comparison_set in comparison_sets.to_vec() {
            let cluster = c[comparison_set.0].clone();
            let comparison_cluster = c[comparison_set.1].clone();

            let similarity = cluster.similarity(&comparison_cluster);
            similarity_matrix.set_similarity(comparison_set.0, comparison_set.1, similarity);
        }

        // Find the two clusters with maximum similarity
        let mut max_similarity = similarity_matrix.get_max_similarity();

        while c_prime.len() > 0 && max_similarity.2 > 0_f32 {
            println!("Max similarity: {:?}", max_similarity);
            println!("{:?}", c);
            println!("{:?}", c_prime);
            println!("{:?} <-> {:?}", c[max_similarity.0], c[max_similarity.1]);

            let mut remove_list = vec![max_similarity.0, max_similarity.1];
            remove_list.sort();
            remove_list.reverse();

            for item in remove_list {
                // Get the cluster so we know what we're removing.
                let cluster = &c[item];

                // Now, find the actual position within c_prime.
                let position = c_prime
                    .iter()
                    .position(|prime_cluster| prime_cluster == cluster)
                    .unwrap();

                println!("{:?}", c_prime);
                println!(
                    "We mapped c index {:?} to c_prime index {:?}. Removing index {:?}.",
                    item, position, position
                );
                c_prime.remove(position);
                println!("{:?}", c_prime);

                println!("{:?}", similarity_matrix);
                similarity_matrix.remove_element(item);
                println!("{:?}", similarity_matrix);
            }

            let max_cluster_1 = c[max_similarity.0].clone();

            let unioned_cluster = max_cluster_1.union(&c[max_similarity.1]);

            // Add new cluster to everything!
            c.push(unioned_cluster.clone());
            c_prime.push(unioned_cluster.clone());
            let index = c.len() - 1;
            println!("Adding index {:?} to the similarity matrix.", index);
            similarity_matrix.add_element(index);
            println!("{:?}", similarity_matrix);

            // Calculate new similarities for the added element.
            for (comparison_index, comparison_cluster) in c_prime.iter().enumerate() {
                if unioned_cluster == *comparison_cluster {
                    continue;
                }

                let cluster_parents = Set::from_iter(unioned_cluster.get_parents());
                let comparison_cluster_parents = Set::from_iter(comparison_cluster.get_parents());

                if cluster_parents
                    .intersection(&comparison_cluster_parents)
                    .len()
                    > 0
                {
                    let similarity = unioned_cluster.similarity(comparison_cluster);
                    similarity_matrix.set_similarity(index, comparison_index, similarity);
                }
            }

            println!("{:?}", similarity_matrix);
            max_similarity = similarity_matrix.get_max_similarity();
        }
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
        let cluster = Cluster::new(vec![1], vec![0]);
        let siblings = powergraph.find_siblings(&cluster);

        assert_eq!(siblings, Set::from_iter(vec!["sibling", "sibling2"]));
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
        let cluster = Cluster::new(vec![3], vec![0, 1, 2]);
        let siblings = powergraph.find_siblings(&cluster);

        assert_eq!(siblings, Set::from_iter(vec!["sibling", "sibling2"]));
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
