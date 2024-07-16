mod cluster_repository;
mod clusters;
mod edge_repository;
mod sets;
mod similarity_matrix;
mod unordered_tuple;
mod utils;

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::hash::Hasher;

use std::process::exit;
use std::time::Instant;

use cluster_repository::ClusterRepository;
use clusters::Cluster;
use edge_repository::EdgeRepository;
use itertools::Itertools;
use serde::Serialize;
use sets::Set;
use similarity_matrix::SimilarityMatrix;
use unordered_tuple::UnorderedTuple;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

use log::{info, trace, warn};

#[wasm_bindgen(start)]
fn start() {
    // executed automatically ...
    set_panic_hook();
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn debug(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => {
        let value  = format_args!($($t)*).to_string();

        #[cfg(target_arch = "wasm32")]
        log(&value);

        #[cfg(not(target_arch = "wasm32"))]
        info!("{}", &value);
    }
}

macro_rules! console_debug {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => {
        let value  = format_args!($($t)*).to_string();
        // #[cfg(target_arch = "wasm32")]
        // debug(&value);
        #[cfg(not(target_arch = "wasm32"))]
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
#[derive(Debug, Clone, Serialize)]
pub struct Edge {
    from: NodeId,
    to: NodeId,
}

impl std::hash::Hash for Edge {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.from.hash(state);
        self.to.hash(state);
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to
    }
}

impl Eq for Edge {
    // fn eq(&self, other: &Self) -> bool {
    //     self.from == other.from && self.to == other.to && self.size == other.size
    // }
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

    pub fn get_from(&self) -> String {
        self.from.clone()
    }

    pub fn get_to(&self) -> String {
        self.to.clone()
    }

    pub fn get_id(&self) -> String {
        format!("{}-{}", self.from, self.to)
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

#[derive(Debug, Clone)]
pub struct PowerEdgeCandidate {
    from: Cluster,
    to: Cluster,
    size: f32,
}

impl Hash for PowerEdgeCandidate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
    }
}

impl PartialEq for PowerEdgeCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.size == other.size
    }
}

impl Eq for PowerEdgeCandidate {
    // fn eq(&self, other: &Self) -> bool {
    //     self.from == other.from && self.to == other.to && self.size == other.size
    // }
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
    edges: EdgeRepository,
    power_nodes: Vec<PowerNode>,
    power_edges: Vec<PowerEdge>,
    clusters: Vec<Cluster>,
}

#[wasm_bindgen]
impl PowerGraph {
    #[wasm_bindgen(constructor)]
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> PowerGraph {
        // console::log_1(&"Hello using web-sys".into());

        let edge_repository = EdgeRepository::from_edge_list(edges);

        PowerGraph {
            nodes,
            edges: edge_repository,
            power_edges: Vec::new(),
            power_nodes: Vec::new(),
            clusters: Vec::new(),
        }
    }

    #[wasm_bindgen]
    pub fn to_object(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self).unwrap()
    }

    // Given a from index and to index, return the edge if it exists in the graph.
    fn get_edge(&self, from: &NodeId, to: &NodeId) -> Option<Edge> {
        self.edges.get_edge(from, to)
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

        console_debug!("{:?} ", power_edge);
        console_debug!("{:?} -> {:?}", source_power_node, target_power_node);
        let target_items: Vec<String> = target_power_node.cluster.items.into_iter().collect();
        let edges: Vec<Edge> = source_power_node
            .cluster
            .items
            .iter()
            .cartesian_product(target_items.into_iter())
            .map(|item| {
                console_debug!("{:?}", item);
                return Edge::new(&item.0, &item.1);
            })
            .collect();

        return Some(edges);
    }

    /// For a given set of nodes, return all edges between the nodes.
    fn subgraph(&self, nodes: &Set<NodeId>) -> Vec<Edge> {
        self.edges.subgraph(nodes)
    }

    fn neighbors(&self, node_id: &NodeId) -> Set<String> {
        let parents = self.edges.parents(node_id);
        let children = self.edges.children(node_id);

        parents.union(&children)
    }

    fn predecessors(&self, node_id: &NodeId) -> Set<String> {
        self.edges.parents(node_id)
    }

    /// Use graph topology to identify cluster pairs for comparison.
    fn generate_graph_comparison_set(
        &self,
        clusters: &Vec<Cluster>,
    ) -> Vec<UnorderedTuple<Cluster>> {
        let mut neighborhood_cluster_map: HashMap<String, Vec<Cluster>> = HashMap::new();

        console_debug!("Constructing neighborhood cluster map.");
        for cluster in clusters.iter() {
            for neighbor in cluster.get_neighbors() {
                if neighborhood_cluster_map.contains_key(&neighbor) {
                    if let Some(clusters) = neighborhood_cluster_map.get_mut(&neighbor) {
                        clusters.push(cluster.clone());
                    }
                } else {
                    neighborhood_cluster_map.insert(neighbor, vec![cluster.clone()]);
                }
            }
        }
        console_debug!(
            "Neighborhood cluster map created with {:?} entries.",
            neighborhood_cluster_map.len()
        );

        console_debug!("Creating comparison set.");
        return neighborhood_cluster_map
            .into_values()
            .flat_map(|cluster_ids| {
                cluster_ids
                    .iter()
                    .combinations(2)
                    .map(|combination| UnorderedTuple {
                        one: (*(combination.get(0).unwrap())).clone(),
                        two: (*(combination.get(1).unwrap())).clone(),
                    })
                    .collect::<Vec<UnorderedTuple<Cluster>>>()
            })
            .collect();
    }

    #[wasm_bindgen]
    pub fn decompose(&mut self) {
        let mut cluster_repository = ClusterRepository::new();

        // let mut c: Vec<Cluster> = Vec::new();

        // Add all nodes to c and c_prime as singleton clusters.
        console_log!("Identify singleton clusters.");
        for node in (&self.nodes).into_iter() {
            // println!("Node: {:?}", node);
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

            cluster_repository.add_cluster(&cluster_nodes);
        }
        let mut c_prime = cluster_repository.clone();

        console_log!(
            "Singleton cluster identification complete. Found {:?} clusters.",
            cluster_repository.len()
        );

        let mut similarity_matrix = SimilarityMatrix::new();

        let comparison_sets =
            // clusters::generate_comparison_set(&c_prime);
            self.generate_graph_comparison_set(&c_prime.clone().clusters.into_values().collect());

        console_log!(
            "Identified {:?} sets of clusters for comparison.",
            comparison_sets.len()
        );

        let mut cluster_map: HashMap<String, Cluster> = cluster_repository.clusters.clone();

        for comparison_set in comparison_sets.to_vec() {
            let cluster = comparison_set.one;

            let comparison_cluster = comparison_set.two;

            let similarity = cluster.similarity(&comparison_cluster);
            similarity_matrix.set_similarity(
                UnorderedTuple {
                    one: cluster.get_id(),
                    two: comparison_cluster.get_id(),
                },
                similarity,
            );
        }

        // Find the two clusters with maximum similarity
        let mut max_similarity_result = similarity_matrix.get_max_similarity();
        console_debug!("{:?}", max_similarity_result);

        const MINIMUM_SIMILARITY: f32 = 0.25_f32;

        while c_prime.len() > 0 {
            match max_similarity_result {
                Some(_) => (),
                None => break,
            }

            let max_similarity = max_similarity_result.unwrap();

            if max_similarity.1 < MINIMUM_SIMILARITY {
                break;
            }

            console_log!("Max similarity: {:?}", max_similarity);
            console_log!("Clusters remaining to process: {:?}", c_prime.len());

            let cluster = cluster_map.get(&max_similarity.0.one).unwrap();

            let comparison_cluster = cluster_map.get(&max_similarity.0.two).unwrap();

            // console_debug!("{:?} <-> {:?}", cluster, comparison_cluster);

            c_prime.remove(&cluster.get_id());
            c_prime.remove(&comparison_cluster.get_id());

            similarity_matrix.remove_element(cluster.get_id());
            similarity_matrix.remove_element(comparison_cluster.get_id());

            let unioned_cluster = cluster.clone().union(&comparison_cluster);

            // Add new cluster to everything!
            cluster_repository.add_cluster(&unioned_cluster);
            c_prime.add_cluster(&unioned_cluster);
            cluster_map.insert(unioned_cluster.get_id(), unioned_cluster.clone());

            // Calculate new similarities for the added element.

            // let cluster_parents = Set::from_iter(unioned_cluster.get_neighbors());
            for comparison_cluster in c_prime.get_sibling_clusters(&unioned_cluster).iter() {
                if unioned_cluster == *comparison_cluster {
                    continue;
                }

                // let comparison_cluster_parents = Set::from_iter(comparison_cluster.get_neighbors());

                let similarity = unioned_cluster.similarity(&comparison_cluster);
                similarity_matrix.set_similarity(
                    UnorderedTuple {
                        one: unioned_cluster.get_id(),
                        two: comparison_cluster.get_id(),
                    },
                    similarity,
                );
            }

            // console_debug!("{:?}", similarity_matrix);
            max_similarity_result = similarity_matrix.get_max_similarity();
        }

        // Add first and second order neighborhoods as clusters in `c`.

        for cluster in cluster_repository.clusters.clone().into_values() {
            let items = cluster.get_neighbors();
            let neighbors = items
                .iter()
                .map(|node| self.neighbors(node))
                .fold(Set::new(), |acc: Set<String>, e| acc.union(&e));
            let neighborhood_cluster = Cluster::new(Set::from_iter(items), neighbors);

            let neighbor_similarity = cluster.similarity(&neighborhood_cluster);

            if neighbor_similarity >= MINIMUM_SIMILARITY {
                console_log!(
                    "The similarity between {:?} and {:?} is {:?}. Adding to `c`.",
                    cluster,
                    neighborhood_cluster,
                    neighbor_similarity
                );

                cluster_repository.add_cluster(&neighborhood_cluster);
            }
        }

        // Do it again for second-order neighbors.
        for cluster in cluster_repository.clusters.clone().into_values() {
            let items = cluster.get_neighbors();
            let neighbors = items
                .iter()
                .map(|node| self.neighbors(node))
                .fold(Set::new(), |acc: Set<String>, e| acc.union(&e));
            let neighborhood_cluster = Cluster::new(Set::from_iter(items), neighbors);

            let neighbor_similarity = cluster.similarity(&neighborhood_cluster);

            if neighbor_similarity >= MINIMUM_SIMILARITY {
                cluster_repository.add_cluster(&neighborhood_cluster);
            }
        }

        for cluster in cluster_repository.clusters.clone().into_values() {
            if cluster.get_items().len() > 1 {
                continue;
            }

            console_debug!(
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

        // TODO: This is probably an area for a performance improvement.
        // for cluster_pair in self.generate_graph_comparison_set(&self.clusters) {
        console_log!("Checking cluster pairs for poweredge candidates");
        let combinations: Vec<UnorderedTuple<&Cluster>> = cluster_repository
            .clusters
            .values()
            .combinations_with_replacement(2)
            .map(|cluster| UnorderedTuple {
                one: *(cluster.get(0).unwrap()),
                two: *(cluster.get(1).unwrap()),
            })
            .collect();

        let combination_count = combinations.len();
        console_log!(
            "{:?} combinations for power edge candidates found. Evaluating.",
            combination_count
        );

        // #[cfg(not(target_arch = "wasm32"))]
        // {
        //     let mut time_chunk_start_time = Instant::now();
        //     let mut count = 0;
        // }
        for cluster_pair in combinations.iter() {
            // #[cfg(not(target_arch = "wasm32"))]
            // {
            //     count += 1;

            //     if time_chunk_start_time.elapsed().as_secs() >= 10 {
            //         let rate = count as f32 / time_chunk_start_time.elapsed().as_secs() as f32;
            //         console_log!(
            //         "Checking combination {:?} of {:?}. Speed: {:?} combinations / second. Time remaining: {:?}",
            //         index,
            //         combination_count,
            //         rate,
            //         (combination_count - index) as f32 / rate
            //     );
            //         time_chunk_start_time = Instant::now();
            //         count = 0;
            //     }
            // }

            let cluster_one = cluster_pair.one;
            let cluster_two = cluster_pair.two;

            let node_intersection = cluster_one.items.intersection(&cluster_two.items);
            let node_union = cluster_one.items.union(&cluster_two.items);

            if node_intersection.len() == 0
                && self.clusters_create_subgraph(&cluster_one, &cluster_two)
            {
                // console_debug!(
                //     "  a non-intersecting candidate between {:?} and {:?}.",
                //     cluster_one.get_id(),
                //     cluster_two.get_id()
                // );

                let edges = self.subgraph(&node_union);

                edge_candidates.push({
                    PowerEdgeCandidate {
                        from: cluster_one.clone(),
                        to: cluster_two.clone(),
                        size: edges.len() as f32,
                    }
                })
            }

            if cluster_one == cluster_two && self.clusters_are_clique(&cluster_one, &cluster_two) {
                // console_debug!(
                //     "There is a clique candidate between {:?} and {:?}.",
                //     cluster_one.get_id(),
                //     cluster_two.get_id()
                // );

                let edges = self.subgraph(&node_union);
                edge_candidates.push({
                    PowerEdgeCandidate {
                        from: cluster_one.clone(),
                        to: cluster_two.clone(),
                        size: edges.len() as f32 / 2_f32,
                    }
                })
            }
        }

        // console_debug!("PowerEdge Candidates: {:?}", edge_candidates);

        let mut completed_candidates: HashSet<PowerEdgeCandidate> = HashSet::new();

        while edge_candidates.len() > 0 {
            edge_candidates.sort_by_key(|item| (item.size * 10000_f32) as u32);

            let edge_candidate = edge_candidates.pop().unwrap();

            let candidate_processor_results =
                self.process_edge_candidate(&edge_candidate, &cluster_repository);

            for result in candidate_processor_results {
                match result {
                    PowerEdgeCandidateProcessorOutput::NewPowerEdgeCandidate(candidate) => {
                        // console_log!("Power Edge Candidate found: {:?}", candidate);

                        // Don't process the same edge candidate twice.
                        if completed_candidates.contains(&candidate) {
                            continue;
                        }

                        // Don't process invalid edges
                        if candidate.from.get_id() == "" || candidate.to.get_id() == "" {
                            continue;
                        }

                        cluster_repository.add_cluster(&candidate.from);
                        cluster_repository.add_cluster(&candidate.to);

                        if !edge_candidates.contains(&candidate) {
                            edge_candidates.push(candidate.clone());
                        }
                        completed_candidates.insert(candidate.clone());
                    }
                    PowerEdgeCandidateProcessorOutput::NewPowerNode(power_node) => {
                        // console_log!("Power Node found: {:?}", power_node);
                        cluster_repository.add_cluster(&power_node.cluster);

                        self.power_nodes.push(power_node);
                    }
                    PowerEdgeCandidateProcessorOutput::NewPowerEdge(power_edge) => {
                        // console_log!("Power Edge found: {:?}", power_edge);

                        self.power_edges.push(power_edge)
                    }
                }
            }
            console_log!("Candidate Count: {:?}", edge_candidates.len());
        }

        // For all remaining edges not yet covered by power edges, create new power edges.
        console_debug!("PowerEdges: {:?}", self.power_edges);
        let covered_edges: Vec<Edge> = self
            .power_edges
            .iter()
            .map(|power_edge| return self.expand_power_edge(power_edge).unwrap())
            .flatten()
            .collect();

        console_debug!("Covered edges: {:?}", covered_edges);
        for edge in self.edges.clone().into_iter() {
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
        cluster_repository: &ClusterRepository,
    ) -> Vec<PowerEdgeCandidateProcessorOutput> {
        if edge_candidate.size <= 2.0 && edge_candidate.from == edge_candidate.to {
            return vec![];
        }

        console_debug!("Evaluating PowerEdge Candidate {:?}", edge_candidate);

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
                console_debug!(
                    "Creating a new PowerEdgeCandidate.\n\tCluster U: {:?}\n\tCluster S: {:?}",
                    edge_candidate.from,
                    power_node.cluster
                );

                // console_debug!("Checking intersection: {:?}.", u_s_intersection);

                // console_debug!(
                //     "U - S: {:?} ",
                //     edge_candidate
                //         .from
                //         .items
                //         .difference(&power_node.cluster.items)
                // );

                // console_debug!(
                //     "S - U: {:?}.",
                //     power_node
                //         .cluster
                //         .items
                //         .difference(&edge_candidate.from.items)
                // );

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
            // console_debug!(
            //     "w_s_intersection with {:?}: {:?}",
            //     power_node,
            //     w_s_intersection
            // );

            let s_subset_w = edge_candidate
                .to
                .items
                .is_proper_subset_of(&power_node.cluster.items);
            let w_subset_s = power_node
                .cluster
                .items
                .is_proper_subset_of(&edge_candidate.to.items);

            // console_debug!("s_subset_w: {:?}, w_subset_s: {:?}", s_subset_w, w_subset_s);

            if w_s_intersection.len() > 0 && !s_subset_w && !w_subset_s {
                console_debug!(
                    "Creating a new PowerEdgeCandidate.\n\tCluster W: {:?}\n\tCluster S: {:?}",
                    edge_candidate.to,
                    power_node.cluster
                );

                console_debug!("Checking intersection: {:?}.", w_s_intersection);

                // console_debug!(
                //     "U - S: {:?} ",
                //     edge_candidate
                //         .to
                //         .items
                //         .difference(&power_node.cluster.items)
                // );

                // console_debug!(
                //     "S - U: {:?}.",
                //     power_node
                //         .cluster
                //         .items
                //         .difference(&edge_candidate.to.items)
                // );

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

        // let cluster_subgraph = self.subgraph(&power_edge_nodes);

        let overlapping_power_edges: Vec<&PowerEdge> = self
            .power_edges
            .iter()
            .filter(|power_edge| {
                let s = match cluster_repository.get(&power_edge.from){
                    Some(cluster) => cluster,
                    None => {
                        panic!("Attempted to load the cluster {:?}, from the cluster map, but it was not found. Skipping check.", power_edge.from);
                    }
                };
                let t = match cluster_repository.get(&power_edge.to){
                    Some(cluster) => cluster,
                    None => {
                        panic!("Attempted to load the cluster {:?}, from the cluster map, but it was not found. Skipping check.", power_edge.to);
                    }
                };

                // Check if (UxW) intersects with (SxT).
                let candidate_union = edge_candidate.from.clone().union(&edge_candidate.to);
                let comparison_union = s.clone().union(&t);

                let candidate_subgraph = Set::from_iter(self.subgraph(&candidate_union.items));
                let comparison_subgraph = Set::from_iter(self.subgraph(&comparison_union.items));

                let edge_intersection = candidate_subgraph.intersection(&comparison_subgraph);

                return edge_intersection.len() > 0;
            })
            .collect();

        console_debug!("Overlapping power edges: {:?}", overlapping_power_edges);

        if overlapping_power_edges.len() > 0 {
            for power_edge in overlapping_power_edges {
                // If (S, T) covers not all edges of (U, W): ((U × W) ⊄ (S × T)):

                let s = cluster_repository.get(&power_edge.from).unwrap();
                let t = cluster_repository.get(&power_edge.to).unwrap();

                let candidate_union = edge_candidate.from.clone().union(&edge_candidate.to);
                let comparison_union = s.clone().union(&t);

                let candidate_subgraph = Set::from_iter(self.subgraph(&candidate_union.items));
                let comparison_subgraph = Set::from_iter(self.subgraph(&comparison_union.items));

                let covers_all_edges = candidate_subgraph.is_proper_subset_of(&comparison_subgraph);

                console_debug!(
                    "Is {:?} a subset of {:?}? {:?}",
                    candidate_subgraph,
                    comparison_subgraph,
                    covers_all_edges
                );

                if !covers_all_edges {
                    if edge_candidate.from.items.is_proper_subset_of(&s.items) {
                        console_debug!(
                            "edge containment: Candidate edge source {:?} is a proper subset of source {:?} targeting {:?}",
                            edge_candidate.from,
                            s, t
                        );
                        let target_cluster = edge_candidate.to.clone().difference(&t);

                        return vec![PowerEdgeCandidateProcessorOutput::NewPowerEdgeCandidate(
                            PowerEdgeCandidate {
                                from: edge_candidate.from.clone(),
                                to: target_cluster.clone(),
                                size: (target_cluster.size() + edge_candidate.from.size()) as f32,
                            },
                        )];
                    } else if edge_candidate.from.items.is_proper_subset_of(&t.items) {
                        console_debug!(
                            "edge containment: Candidate edge source {:?} is a proper subset of target {:?} sourced from {:?}",
                            edge_candidate.from,
                            t, s
                        );

                        let target_cluster = edge_candidate.to.clone().difference(&s);

                        return vec![PowerEdgeCandidateProcessorOutput::NewPowerEdgeCandidate(
                            PowerEdgeCandidate {
                                from: edge_candidate.from.clone(),
                                to: target_cluster.clone(),
                                size: (target_cluster.size() + edge_candidate.from.size()) as f32,
                            },
                        )];
                    } else if edge_candidate.to.items.is_proper_subset_of(&s.items) {
                        console_debug!(
                            "edge containment: Candidate edge target {:?} is a proper subset of {:?}",
                            edge_candidate.to,
                            s
                        );

                        let source_cluster = edge_candidate.from.clone().difference(&t);

                        return vec![PowerEdgeCandidateProcessorOutput::NewPowerEdgeCandidate(
                            PowerEdgeCandidate {
                                from: source_cluster.clone(),
                                to: edge_candidate.to.clone(),
                                size: (source_cluster.size() + edge_candidate.to.size()) as f32,
                            },
                        )];
                    } else if edge_candidate.to.items.is_proper_subset_of(&t.items) {
                        console_debug!(
                            "edge containment: Candidate edge target {:?} is a proper subset of {:?}",
                            edge_candidate.to,
                            t
                        );

                        let source_cluster = edge_candidate.from.clone().difference(&s);

                        return vec![PowerEdgeCandidateProcessorOutput::NewPowerEdgeCandidate(
                            PowerEdgeCandidate {
                                from: source_cluster.clone(),
                                to: edge_candidate.to.clone(),
                                size: (source_cluster.size() + edge_candidate.to.size()) as f32,
                            },
                        )];
                    }
                }
            }
            return vec![];
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

    fn clusters_are_clique(&self, cluster_one: &Cluster, cluster_two: &Cluster) -> bool {
        for u in cluster_one.items.iter() {
            for w in cluster_two.items.iter() {
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

        let is_subgraph = powergraph.clusters_are_clique(&cluster_one, &cluster_one);
        assert!(is_subgraph);
    }

    #[test]
    fn singular_clusters_can_be_cliques() {
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
            Set::from_iter(vec!["a".to_string()]),
            Set::from_iter(vec!["b".to_string(), "c".to_string()]),
        );

        let is_subgraph = powergraph.clusters_are_clique(&cluster_one, &cluster_one);
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

        let is_subgraph = powergraph.clusters_are_clique(&cluster_one, &cluster_one);
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

        assert!(Set::from_iter(subgraph_edges) == Set::from_iter(edges));
    }

    // Does the decomposition algorithm appropriately detect bicliques?
    #[test]
    fn biclique_detection() {
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

        println!("{:?}", powergraph.power_nodes);
        println!("{:?}", powergraph.power_edges);

        assert_eq!(powergraph.power_edges.len(), 1);
        assert!(
            powergraph.power_edges[0].from == "c-d-e" || powergraph.power_edges[0].to == "c-d-e"
        );
    }
}
