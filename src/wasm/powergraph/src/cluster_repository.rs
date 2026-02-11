use std::collections::{HashMap, HashSet};

use log::trace;

use crate::clusters::Cluster;

#[derive(Debug, Clone)]
pub struct ClusterRepository {
    pub clusters: HashMap<String, Cluster>,
    overlaps: HashMap<(String, String), OverlapType>,

    // node_cluster_neighbor_map is a mapping between nodes and the clusters their neighbors are in
    node_cluster_neighbor_map: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Clone)]
enum OverlapType {
    Partial,
    Subset,
    Equal,
}

fn is_equal_clusters(cluster: &Cluster, comparison_cluster: &Cluster) -> bool {
    cluster.items == comparison_cluster.items
}

fn is_partial_subset(cluster: &Cluster, comparison_cluster: &Cluster) -> bool {
    cluster.items.intersection(&comparison_cluster.items).len() > 0
        && (cluster.items.difference(&comparison_cluster.items).len() == 0
            || comparison_cluster.items.difference(&cluster.items).len() == 0)
}

fn is_subset(cluster: &Cluster, comparison_cluster: &Cluster) -> bool {
    cluster.items.difference(&comparison_cluster.items).len() == 0
        && cluster.items.difference(&comparison_cluster.items).len() > 0
}

impl ClusterRepository {
    pub fn new() -> ClusterRepository {
        ClusterRepository {
            clusters: HashMap::new(),
            overlaps: HashMap::new(),
            node_cluster_neighbor_map: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.clusters.len()
    }

    fn calculate_overlaps(&self, cluster: &Cluster) -> HashMap<(String, String), OverlapType> {
        let cluster_id = cluster.get_id().to_string();
        let overlaps = self
            .clusters
            .iter()
            .filter_map(|(comparison_cluster_id, comparison_cluster)| {
                if is_equal_clusters(cluster, comparison_cluster) {
                    return Some((
                        (cluster_id.clone(), comparison_cluster_id.clone()),
                        OverlapType::Equal,
                    ));
                }

                if is_partial_subset(cluster, comparison_cluster) {
                    return Some((
                        (cluster_id.clone(), comparison_cluster_id.clone()),
                        OverlapType::Partial,
                    ));
                }

                if is_subset(cluster, comparison_cluster) {
                    return Some((
                        (cluster_id.clone(), comparison_cluster_id.clone()),
                        OverlapType::Subset,
                    ));
                }

                None
            })
            .collect::<HashMap<(String, String), OverlapType>>();
        // trace!(
        //     "cluster: {:?}. overlaps: {:?} clusters: {:?}",
        //     cluster,
        //     overlaps,
        //     self.clusters
        // );
        return overlaps;
    }

    // Add a new Cluster into the ClusterRepository and identify overlapping clusters.
    pub fn add_cluster(&mut self, cluster: &Cluster) {
        let cluster_id = cluster.get_id().to_string();
        self.clusters.insert(cluster_id.clone(), cluster.clone());

        let overlaps = self.calculate_overlaps(cluster);
        self.overlaps.extend(overlaps.into_iter());

        cluster.get_neighbors().into_iter().for_each(|node| {
            self.node_cluster_neighbor_map
                .entry(node)
                .and_modify(|mapping| {
                    mapping.insert(cluster_id.clone());
                })
                .or_insert_with(|| {
                    let mut mapping = HashSet::new();
                    mapping.insert(cluster_id.clone());
                    return mapping;
                });
        });
    }

    pub fn get(&self, cluster_id: &String) -> Option<&Cluster> {
        self.clusters.get(cluster_id)
    }

    pub fn remove(&mut self, cluster_id: &String) {
        self.clusters.remove(cluster_id);

        for key in self.overlaps.clone().into_keys().filter_map(|(from, to)| {
            if from == *cluster_id || to == *cluster_id {
                return Some((from, to));
            }
            None
        }) {
            self.overlaps.remove(&key);
        }

        for (_, neighbors) in self.node_cluster_neighbor_map.iter_mut() {
            neighbors.remove(cluster_id);
        }
    }

    pub fn get_sibling_clusters(&self, cluster: &Cluster) -> HashSet<Cluster> {
        cluster
            .get_neighbors()
            .iter()
            .filter_map(|neighbor| self.node_cluster_neighbor_map.get(neighbor))
            .flatten()
            .filter_map(|cluster_id| self.clusters.get(cluster_id))
            .map(|value| value.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlap_calculation() {}
}
