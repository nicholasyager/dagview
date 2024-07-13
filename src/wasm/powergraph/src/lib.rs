mod clusters;
mod sets;
mod similarity_matrix;
mod unordered_tuple;
mod utils;

use clusters::Cluster;
use itertools::Itertools;
use serde::Serialize;
use sets::Set;
use similarity_matrix::SimilarityMatrix;
use unordered_tuple::UnorderedTuple;
use wasm_bindgen::prelude::*;

use log::{trace, warn};

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
    ($($t:tt)*) => {
        let value  = format_args!($($t)*).to_string();
        log(&value);
        trace!("{}", &value);
    }
}

#[wasm_bindgen]
pub fn greet(value: &str) {
    alert(&format!("Hello, {}!", &value));
}

type NodeId = String;
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize)]
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
#[derive(Debug, Clone, Serialize)]
pub struct PowerNode {
    id: PowerNodeId,
    cluster: Cluster,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct PowerEdge {
    from: PowerNodeId,
    to: PowerNodeId,
}

#[derive(Debug)]
pub struct PowerEdgeCandidate {
    from: Cluster,
    to: Cluster,
    size: f32,
}

#[derive(Debug)]
enum PowerEdgeCandidateProcessorOutput {
    NewPowerEdgeCandidate(PowerEdgeCandidate),
    NewPowerNode(PowerNode),
    NewPowerEdge(PowerEdge),
}

#[wasm_bindgen]
#[derive(Serialize)]
pub struct PowerGraph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    power_nodes: Vec<PowerNode>,
    power_edges: Vec<PowerEdge>,
    clusters: Vec<Cluster>,
}

#[wasm_bindgen]
impl PowerGraph {
    #[wasm_bindgen(constructor)]
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> PowerGraph {
        // console::log_1(&"Hello using web-sys".into());

        PowerGraph {
            nodes,
            edges,
            power_edges: Vec::new(),
            power_nodes: Vec::new(),
            clusters: Vec::new(),
        }
    }

    #[wasm_bindgen]
    pub fn to_object(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self).unwrap()
    }

    // fn get_node(&self, node_id: &NodeId) -> Option<Node> {
    //     for node in self.nodes.iter() {
    //         if node.id == *node_id {
    //             return Some(node.clone());
    //         }
    //     }

    //     println!("Unable to find {:?} in {:?}", node_id, self.nodes);

    //     return None;
    // }

    // Given a from index and to index, return the edge if it exists in the graph.
    fn get_edge(&self, from: &NodeId, to: &NodeId) -> Option<Edge> {
        for edge in self.edges.iter() {
            if edge.from == *from && edge.to == *to {
                return Some(edge.clone());
            }
        }
        None
    }

    fn find_power_node(&self, search_id: &str, power_nodes: Vec<PowerNode>) -> Option<PowerNode> {
        let candidates = power_nodes
            .into_iter()
            .filter(|power_node| power_node.id == search_id)
            .collect::<Vec<PowerNode>>();

        match candidates.first() {
            Some(power_node) => return Some(power_node.clone()),
            None => return None,
        }
    }

    fn expand_power_edge(&self, power_edge: &PowerEdge) -> Option<Vec<Edge>> {
        let source_power_node: PowerNode = self
            .find_power_node(&power_edge.from, self.power_nodes.clone())
            .unwrap();
        let target_power_node: PowerNode = self
            .find_power_node(&power_edge.to, self.power_nodes.clone())
            .unwrap();

        console_log!("{:?} ", power_edge);
        console_log!("{:?} -> {:?}", source_power_node, target_power_node);

        let edges: Vec<Edge> = source_power_node
            .cluster
            .items
            .iter()
            .cartesian_product(target_power_node.cluster.items.clone())
            .map(|item| {
                console_log!("{:?}", item);
                return Edge::new(&item.0, &item.1);
            })
            .collect();

        return Some(edges);
    }

    /// For a given set of nodes, return all edges between the nodes.
    fn subgraph(&self, nodes: &Set<NodeId>) -> Vec<Edge> {
        let mut output = Vec::new();

        for edge in self.edges.iter() {
            if nodes.contains(edge.from.clone()) && nodes.contains(edge.to.clone()) {
                output.push(edge.clone());
            }
        }

        return output;
    }

    // fn node_id_to_index(&self) {}

    // /// For a given Cluster, find all node names for sibling nodes.
    // fn find_siblings(&self, cluster: &Cluster) -> Set<String> {
    //     let mut output = Set::from_iter(Vec::new());

    //     // Find all children of parents. This will be the selected cluster's generation.

    //     for neighbor_id in cluster.get_neighbors() {
    //         let neighbor = self.get_node(&neighbor_id).unwrap();

    //         let children: Vec<String> = self
    //             .edges
    //             .iter()
    //             .filter_map(|edge| {
    //                 if edge.from == neighbor.id {
    //                     return Some(edge.to.clone());
    //                 }

    //                 None
    //             })
    //             .collect();

    //         for child in children {
    //             output.insert(child);
    //         }
    //     }

    //     let cluster_set = Set::from_iter(cluster.get_items());

    //     // Return the list.
    //     return (output.difference(&cluster_set)).clone();
    // }

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
        let mut c_prime: Vec<Cluster>;

        // Add all nodes to c and c_prime as singleton clusters.
        for node in (&self.nodes).into_iter() {
            // println!("Node: {:?}", node);
            console_log!("Node: {:?}", node);

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

        let comparison_sets: Set<UnorderedTuple<String>> =
            clusters::generate_comparison_set(&c_prime);

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
        let mut max_similarity_result = similarity_matrix.get_max_similarity();
        console_log!("{:?}", max_similarity_result);

        while c_prime.len() > 0 {
            match max_similarity_result {
                Some(_) => (),
                None => break,
            }

            let max_similarity = max_similarity_result.unwrap();

            if max_similarity.1 < 0.5_f32 {
                break;
            }

            console_log!("Max similarity: {:?}", max_similarity);

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

            console_log!("{:?} <-> {:?}", cluster, comparison_cluster);

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

            console_log!("{:?}", similarity_matrix);
            max_similarity_result = similarity_matrix.get_max_similarity();
        }

        // Add first and second order neighborhoods as clusters in `c`.

        for cluster in c.clone() {
            let items = cluster.get_neighbors();
            let neighbors = items
                .iter()
                .map(|node| self.neighbors(node))
                .fold(Set::new(), |acc: Set<String>, e| acc.union(&e));
            let neighborhood_cluster = Cluster::new(Set::from_iter(items), neighbors);

            let neighbor_similarity = cluster.similarity(&neighborhood_cluster);

            if neighbor_similarity >= 0.5 {
                console_log!(
                    "The similarity between {:?} and {:?} is {:?}. Adding to `c`.",
                    cluster,
                    neighborhood_cluster,
                    neighbor_similarity
                );

                c.push(neighborhood_cluster);
            }
        }

        // Do it again for second-order neighbors.
        for cluster in c.clone() {
            let items = cluster.get_neighbors();
            let neighbors = items
                .iter()
                .map(|node| self.neighbors(node))
                .fold(Set::new(), |acc: Set<String>, e| acc.union(&e));
            let neighborhood_cluster = Cluster::new(Set::from_iter(items), neighbors);

            let neighbor_similarity = cluster.similarity(&neighborhood_cluster);

            if neighbor_similarity >= 0.5 {
                c.push(neighborhood_cluster);
            }
        }

        // Deduplicate everything
        let mut processed_indices: Vec<String> = vec![];
        let mut deduped_c: Vec<Cluster> = vec![];
        for item in c {
            if processed_indices.contains(&item.get_id()) {
                continue;
            }

            deduped_c.push(item.clone());
            processed_indices.push(item.get_id());
        }

        self.clusters = deduped_c;

        console_log!("Complete set of clusters: {:?}.", self.clusters);

        // Add singletons to the powergraph
        for cluster in self.clusters.clone().into_iter() {
            if cluster.get_items().len() > 1 {
                continue;
            }

            console_log!(
                "{:?} is a singleton. Adding to PowerNodes.",
                cluster.get_id()
            );

            let singleton = PowerNode {
                id: cluster.get_id(),
                cluster,
            };

            self.power_nodes.push(singleton)
        }

        // Generate candidates for PowerEdges
        let mut edge_candidates: Vec<PowerEdgeCandidate> = Vec::new();

        // console_log!("Generating pairs of clusters to evaluate.",);

        // let cluster_pairs: Set<UnorderedTuple<&Cluster>> = Set::from_iter(

        //         .collect(),
        // );

        // console_log!("{:?} pairs of clusters identified.", cluster_pairs.len());

        for cluster_pair in self
            .clusters
            .iter()
            .combinations(2)
            .map(|cluster| UnorderedTuple {
                one: *(cluster.get(0).unwrap()),
                two: *(cluster.get(1).unwrap()),
            })
        {
            let cluster_one = cluster_pair.one;
            let cluster_two = cluster_pair.two;

            console_log!(
                "Checking {:?} and {:?} for poweredge candidates",
                cluster_one,
                cluster_two
            );

            let node_intersection = cluster_one.items.intersection(&cluster_two.items);
            let node_union = cluster_one.items.union(&cluster_two.items);
            console_log!(" - Intersection : {:?}", node_intersection);
            console_log!(
                " - Subgraph: {:?}",
                self.clusters_create_subgraph(&cluster_one, &cluster_two)
            );

            if node_intersection.len() == 0
                && self.clusters_create_subgraph(&cluster_one, &cluster_two)
            {
                edge_candidates.push({
                    PowerEdgeCandidate {
                        from: cluster_one.clone(),
                        to: cluster_two.clone(),
                        size: node_union.len() as f32,
                    }
                })
            }

            if cluster_one == cluster_two && self.clusters_are_clique(&cluster_one) {
                edge_candidates.push({
                    PowerEdgeCandidate {
                        from: cluster_one.clone(),
                        to: cluster_two.clone(),
                        size: node_union.len() as f32 / 2_f32,
                    }
                })
            }
        }

        println!("PowerEdge Candidates: {:?}", edge_candidates);

        while edge_candidates.len() > 0 {
            edge_candidates.sort_by_key(|item| (item.size * 10000_f32) as u32);

            let edge_candidate = edge_candidates.pop().unwrap();

            let candidate_processor_results = self.process_edge_candidate(&edge_candidate);

            for result in candidate_processor_results {
                match result {
                    PowerEdgeCandidateProcessorOutput::NewPowerEdgeCandidate(candidate) => {
                        println!("Power Edge Candidate found: {:?}", candidate);
                        edge_candidates.push(candidate);
                    }
                    PowerEdgeCandidateProcessorOutput::NewPowerNode(power_node) => {
                        println!("Power Node found: {:?}", power_node);
                        self.power_nodes.push(power_node);
                    }
                    PowerEdgeCandidateProcessorOutput::NewPowerEdge(power_edge) => {
                        println!("Power Edge found: {:?}", power_edge);
                        self.power_edges.push(power_edge)
                    }
                }
            }
        }

        // For all remaining edges not yet covered by power edges, create new power edges.
        console_log!("PowerEdges: {:?}", self.power_edges);
        let covered_edges: Vec<Edge> = self
            .power_edges
            .iter()
            .map(|power_edge| return self.expand_power_edge(power_edge).unwrap())
            .flatten()
            .collect();

        console_log!("Covered edges: {:?}", covered_edges);
        for edge in &self.edges {
            if !covered_edges.contains(&edge)
                && !covered_edges.contains(&Edge {
                    from: edge.to.clone(),
                    to: edge.from.clone(),
                })
            {
                self.power_edges.push(PowerEdge {
                    from: edge.from.clone(),
                    to: edge.to.clone(),
                })
            }
        }

        console_log!(
            "Complete!\nPower Nodes: {:?}\n\tPower Edges: {:?}",
            self.power_nodes,
            self.power_edges
        );
    }

    fn process_edge_candidate(
        &self,
        edge_candidate: &PowerEdgeCandidate,
    ) -> Vec<PowerEdgeCandidateProcessorOutput> {
        if edge_candidate.size == 2.0 && edge_candidate.from == edge_candidate.to {
            return vec![];
        }

        console_log!("Evaluating PowerEdge Candidate {:?}", edge_candidate);

        // Is there an existing powernode that overlaps with the source of the powernode
        // that is not a perfect superset?
        for power_node in self.power_nodes.iter() {
            let u_s_intersection = edge_candidate
                .from
                .items
                .intersection(&power_node.cluster.items);

            let s_subset_u = edge_candidate
                .from
                .items
                .is_subset_of(&power_node.cluster.items);
            let u_subset_s = power_node
                .cluster
                .items
                .is_subset_of(&edge_candidate.from.items);

            if u_s_intersection.len() > 0 && !s_subset_u && !u_subset_s {
                console_log!(
                    "Creating a new PowerEdgeCandidate.\n\tCluster U: {:?}\n\tCluster S: {:?}",
                    edge_candidate.from,
                    power_node.cluster
                );

                console_log!("Checking intersection: {:?}.", u_s_intersection);

                console_log!(
                    "U - S: {:?} ",
                    edge_candidate
                        .from
                        .items
                        .difference(&power_node.cluster.items)
                );

                console_log!(
                    "S - U: {:?}.",
                    power_node
                        .cluster
                        .items
                        .difference(&edge_candidate.from.items)
                );

                let difference_cluster =
                    edge_candidate.from.clone().difference(&power_node.cluster);
                let intersection_cluster = edge_candidate
                    .from
                    .clone()
                    .intersection(&power_node.cluster);

                return vec![
                    PowerEdgeCandidateProcessorOutput::NewPowerEdgeCandidate(PowerEdgeCandidate {
                        from: difference_cluster.clone(),
                        to: edge_candidate.to.clone(),
                        size: (difference_cluster.size() + edge_candidate.to.size()) as f32,
                    }),
                    PowerEdgeCandidateProcessorOutput::NewPowerEdgeCandidate(PowerEdgeCandidate {
                        from: intersection_cluster.clone(),
                        to: edge_candidate.to.clone(),
                        size: (intersection_cluster.size() + edge_candidate.to.size()) as f32,
                    }),
                ];
            }
        }
        // Is there an existing powernode that overlaps with the source of the powernode
        // that is not a perfect superset?
        for power_node in self.power_nodes.iter() {
            let w_s_intersection = edge_candidate
                .to
                .items
                .intersection(&power_node.cluster.items);
            console_log!(
                "w_s_intersection with {:?}: {:?}",
                power_node,
                w_s_intersection
            );

            let s_subset_w = edge_candidate
                .to
                .items
                .is_subset_of(&power_node.cluster.items);
            let w_subset_s = power_node
                .cluster
                .items
                .is_subset_of(&edge_candidate.to.items);

            console_log!("s_subset_w: {:?}, w_subset_s: {:?}", s_subset_w, w_subset_s);

            if w_s_intersection.len() > 0 && !s_subset_w && !w_subset_s {
                console_log!(
                    "Creating a new PowerEdgeCandidate.\n\tCluster W: {:?}\n\tCluster S: {:?}",
                    edge_candidate.to,
                    power_node.cluster
                );

                console_log!("Checking intersection: {:?}.", w_s_intersection);

                console_log!(
                    "U - S: {:?} ",
                    edge_candidate
                        .to
                        .items
                        .difference(&power_node.cluster.items)
                );

                console_log!(
                    "S - U: {:?}.",
                    power_node
                        .cluster
                        .items
                        .difference(&edge_candidate.to.items)
                );

                let difference_cluster = edge_candidate.to.clone().difference(&power_node.cluster);
                let intersection_cluster =
                    edge_candidate.to.clone().intersection(&power_node.cluster);

                return vec![
                    PowerEdgeCandidateProcessorOutput::NewPowerEdgeCandidate(PowerEdgeCandidate {
                        from: edge_candidate.from.clone(),
                        to: difference_cluster.clone(),
                        size: (difference_cluster.size() + edge_candidate.from.size()) as f32,
                    }),
                    PowerEdgeCandidateProcessorOutput::NewPowerEdgeCandidate(PowerEdgeCandidate {
                        from: edge_candidate.from.clone(),
                        to: intersection_cluster.clone(),
                        size: (intersection_cluster.size() + edge_candidate.from.size()) as f32,
                    }),
                ];
            }
        }

        let power_edge_nodes = edge_candidate.from.items.union(&edge_candidate.to.items);

        let cluster_subgraph = self.subgraph(&power_edge_nodes);
        let uncontained_edges: Vec<&Edge> = self
            .edges
            .iter()
            .filter(|edge| !cluster_subgraph.contains(edge))
            .collect();
        if uncontained_edges.len() > 0 {
            warn!("Edges not contained in powernode: {:?}", uncontained_edges);

            let new_candidates = uncontained_edges
                .into_iter()
                .filter_map(|edge| {
                    warn!(
                        "Evaluating edge {:?} not contained in powernode {:?} ",
                        edge, edge_candidate
                    );

                    let s = Set::from_iter(vec![edge.from.clone()]);
                    let t = Set::from_iter(vec![edge.to.clone()]);

                    if edge_candidate.from.items.is_proper_subset_of(&s) {
                        console_log!(
                            "Candidate edge source {:?} does not contain elements of {:?}",
                            edge_candidate.from,
                            s
                        );
                        let target_cluster = edge_candidate
                            .to
                            .clone()
                            .difference(&Cluster::new(t, Set::new()));

                        return Some(PowerEdgeCandidateProcessorOutput::NewPowerEdgeCandidate(
                            PowerEdgeCandidate {
                                from: edge_candidate.from.clone(),
                                to: target_cluster.clone(),
                                size: (target_cluster.size() + edge_candidate.from.size()) as f32,
                            },
                        ));
                    } else if edge_candidate.from.items.is_proper_subset_of(&t) {
                        console_log!(
                            "Candidate edge source {:?} does not contain elements of {:?}",
                            edge_candidate.from,
                            t
                        );

                        let target_cluster = edge_candidate
                            .to
                            .clone()
                            .difference(&Cluster::new(s, Set::new()));

                        return Some(PowerEdgeCandidateProcessorOutput::NewPowerEdgeCandidate(
                            PowerEdgeCandidate {
                                from: edge_candidate.from.clone(),
                                to: target_cluster.clone(),
                                size: (target_cluster.size() + edge_candidate.from.size()) as f32,
                            },
                        ));
                    } else if edge_candidate.to.items.is_proper_subset_of(&s) {
                        console_log!(
                            "Candidate edge target {:?} does not contain elements of {:?}",
                            edge_candidate.to,
                            s
                        );

                        let source_cluster = edge_candidate
                            .from
                            .clone()
                            .difference(&Cluster::new(t, Set::new()));

                        return Some(PowerEdgeCandidateProcessorOutput::NewPowerEdgeCandidate(
                            PowerEdgeCandidate {
                                from: source_cluster.clone(),
                                to: edge_candidate.to.clone(),
                                size: (source_cluster.size() + edge_candidate.to.size()) as f32,
                            },
                        ));
                    } else if edge_candidate.to.items.is_proper_subset_of(&t) {
                        console_log!(
                            "Candidate edge target {:?} does not contain elements of {:?}",
                            edge_candidate.to,
                            t
                        );

                        let source_cluster = edge_candidate
                            .from
                            .clone()
                            .difference(&Cluster::new(s, Set::new()));

                        return Some(PowerEdgeCandidateProcessorOutput::NewPowerEdgeCandidate(
                            PowerEdgeCandidate {
                                from: source_cluster.clone(),
                                to: edge_candidate.to.clone(),
                                size: (source_cluster.size() + edge_candidate.to.size()) as f32,
                            },
                        ));
                    }

                    console_log!("Welp. We somehow didn't find any subsets. Is this bad?");
                    None
                })
                .collect();

            warn!("Candidates!: {:?}", new_candidates);

            return new_candidates;
        }

        if edge_candidate.to == edge_candidate.from {
            return vec![
                PowerEdgeCandidateProcessorOutput::NewPowerNode(PowerNode {
                    id: edge_candidate.to.get_id(),
                    cluster: edge_candidate.to.clone(),
                }),
                PowerEdgeCandidateProcessorOutput::NewPowerEdge(PowerEdge {
                    from: edge_candidate.to.get_id(),
                    to: edge_candidate.to.get_id(),
                }),
            ];
        }

        // Otherwise, add power nodes for `from` and `to`, and a power edge between them.
        return vec![
            PowerEdgeCandidateProcessorOutput::NewPowerNode(PowerNode {
                id: edge_candidate.from.get_id(),
                cluster: edge_candidate.from.clone(),
            }),
            PowerEdgeCandidateProcessorOutput::NewPowerNode(PowerNode {
                id: edge_candidate.to.get_id(),
                cluster: edge_candidate.to.clone(),
            }),
            PowerEdgeCandidateProcessorOutput::NewPowerEdge(PowerEdge {
                from: edge_candidate.from.get_id(),
                to: edge_candidate.to.get_id(),
            }),
        ];
    }

    fn clusters_create_subgraph(&self, cluster_one: &Cluster, cluster_two: &Cluster) -> bool {
        for u in cluster_one.items.iter() {
            for w in cluster_two.items.iter() {
                match self.get_edge(&u, &w) {
                    Some(_edge) => continue,
                    _ => (),
                }

                match self.get_edge(&w, &u) {
                    Some(_edge) => continue,
                    _ => (),
                }

                return false;
            }
        }

        return true;
    }

    fn clusters_are_clique(&self, cluster_one: &Cluster) -> bool {
        for u in cluster_one.items.iter() {
            for w in cluster_one.items.iter() {
                if u == w {
                    continue;
                }

                match self.get_edge(&u, &w) {
                    Some(_edge) => continue,
                    _ => (),
                }

                match self.get_edge(&w, &u) {
                    Some(_edge) => continue,
                    _ => (),
                }

                return false;
            }
        }

        return true;
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

    // #[test]
    // fn find_siblings_trivial_case() {
    //     let nodes: Vec<Node> = vec![
    //         Node::new("parent".to_string(), "foo".to_string()),
    //         Node::new("child".to_string(), "bar".to_string()),
    //         Node::new("sibling".to_string(), "baz".to_string()),
    //         Node::new("sibling2".to_string(), "fizz".to_string()),
    //         Node::new("child2".to_string(), "Boo!".to_string()),
    //     ];

    //     let edges: Vec<Edge> = vec![
    //         Edge::new("parent", "child"),
    //         Edge::new("parent", "sibling"),
    //         Edge::new("parent", "sibling2"),
    //         Edge::new("child", "child2"),
    //     ];

    //     let powergraph = PowerGraph::new(nodes, edges);
    //     let cluster = Cluster::new(
    //         Set::from_iter(vec!["child".to_string()]),
    //         powergraph.neighbors(&"child".to_string()),
    //     );
    //     let siblings = powergraph.find_siblings(&cluster);

    //     assert_eq!(
    //         siblings,
    //         Set::from_iter(vec!["sibling".to_string(), "sibling2".to_string()])
    //     );
    // }

    // #[test]
    // fn find_siblings_non_trivial_case() {
    //     let nodes: Vec<Node> = vec![
    //         Node::new("parent1".to_string(), "foo".to_string()),
    //         Node::new("parent2".to_string(), "foo".to_string()),
    //         Node::new("parent3".to_string(), "foo".to_string()),
    //         Node::new("child".to_string(), "bar".to_string()),
    //         Node::new("sibling".to_string(), "baz".to_string()),
    //         Node::new("sibling2".to_string(), "fizz".to_string()),
    //         Node::new("child2".to_string(), "Boo!".to_string()),
    //     ];

    //     let edges: Vec<Edge> = vec![
    //         Edge::new("parent1", "child"),
    //         Edge::new("parent2", "child"),
    //         Edge::new("parent3", "child"),
    //         Edge::new("parent2", "sibling"),
    //         Edge::new("parent3", "sibling2"),
    //         Edge::new("child", "child2"),
    //     ];

    //     let powergraph = PowerGraph::new(nodes, edges);
    //     let cluster = Cluster::new(
    //         Set::from_iter(vec!["child".to_string()]),
    //         powergraph.neighbors(&"child".to_string()),
    //     );
    //     let siblings = powergraph.find_siblings(&cluster);

    //     assert_eq!(
    //         siblings,
    //         Set::from_iter(vec!["sibling".to_string(), "sibling2".to_string()])
    //     );
    // }

    #[test]
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

    #[test]
    fn clusters_create_subgraph() {
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
        let cluster_one = Cluster::new(
            Set::from_iter(vec!["parent".to_string()]),
            Set::from_iter(vec![
                "child".to_string(),
                "sibling".to_string(),
                "sibling2".to_string(),
            ]),
        );

        let cluster_two = Cluster::new(
            Set::from_iter(vec!["child".to_string()]),
            Set::from_iter(vec!["parent".to_string(), "child2".to_string()]),
        );

        let is_subgraph = powergraph.clusters_create_subgraph(&cluster_one, &cluster_two);
        assert!(is_subgraph);
    }

    #[test]
    fn clusters_create_subgraph_negative_case() {
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
        let cluster_one = Cluster::new(
            Set::from_iter(vec!["parent".to_string()]),
            Set::from_iter(vec![
                "child".to_string(),
                "sibling".to_string(),
                "sibling2".to_string(),
            ]),
        );

        let cluster_two = Cluster::new(
            Set::from_iter(vec!["child2".to_string()]),
            Set::from_iter(vec!["child".to_string()]),
        );

        let is_subgraph = powergraph.clusters_create_subgraph(&cluster_one, &cluster_two);
        assert!(!is_subgraph);
    }

    #[test]
    fn cluster_clique_detection() {
        let nodes: Vec<Node> = vec![
            Node::new("a".to_string(), "foo".to_string()),
            Node::new("b".to_string(), "bar".to_string()),
            Node::new("c".to_string(), "baz".to_string()),
            Node::new("d".to_string(), "baz".to_string()),
        ];

        let edges: Vec<Edge> = vec![
            Edge::new("a", "b"),
            Edge::new("a", "c"),
            Edge::new("b", "c"),
            Edge::new("b", "d"),
        ];

        let powergraph = PowerGraph::new(nodes, edges);
        let cluster_one = Cluster::new(
            Set::from_iter(vec!["a".to_string(), "c".to_string()]),
            Set::from_iter(vec!["b".to_string(), "a".to_string(), "c".to_string()]),
        );

        let is_subgraph = powergraph.clusters_are_clique(&cluster_one);
        assert!(is_subgraph);
    }

    #[test]
    fn cluster_clique_detection_negative() {
        let nodes: Vec<Node> = vec![
            Node::new("a".to_string(), "foo".to_string()),
            Node::new("b".to_string(), "bar".to_string()),
            Node::new("c".to_string(), "baz".to_string()),
            Node::new("d".to_string(), "baz".to_string()),
        ];

        let edges: Vec<Edge> = vec![
            Edge::new("a", "b"),
            Edge::new("a", "c"),
            Edge::new("b", "c"),
            Edge::new("b", "d"),
        ];

        let powergraph = PowerGraph::new(nodes, edges);
        let cluster_one = Cluster::new(
            Set::from_iter(vec!["a".to_string(), "c".to_string(), "d".to_string()]),
            Set::from_iter(vec!["b".to_string(), "a".to_string(), "c".to_string()]),
        );

        let is_subgraph = powergraph.clusters_are_clique(&cluster_one);
        assert!(!is_subgraph);
    }

    #[test]
    fn cluster_subgraph() {
        let nodes: Vec<Node> = vec![
            Node::new("a".to_string(), "foo".to_string()),
            Node::new("b".to_string(), "bar".to_string()),
            Node::new("c".to_string(), "baz".to_string()),
            Node::new("d".to_string(), "baz".to_string()),
        ];

        let edges: Vec<Edge> = vec![
            Edge::new("a", "b"),
            Edge::new("a", "c"),
            Edge::new("b", "c"),
            Edge::new("b", "d"),
        ];

        let powergraph = PowerGraph::new(nodes, edges.clone());
        let nodes = Set::from_iter(vec![
            "a".to_string(),
            "c".to_string(),
            "b".to_string(),
            "d".to_string(),
        ]);

        let subgraph_edges = powergraph.subgraph(&nodes);

        assert!(subgraph_edges == edges);
    }
}
