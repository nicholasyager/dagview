use std::collections::HashSet;

use serde::Serialize;

use crate::{sets::Set, unordered_tuple::UnorderedTuple};

#[derive(Debug, Clone, Serialize)]
pub struct Cluster {
    pub items: Set<String>,
    neighbors: Set<String>,
    id: String,
}

impl std::hash::Hash for Cluster {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.items.hash(state);
        self.neighbors.hash(state);
    }
}

impl PartialEq for Cluster {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items && self.neighbors == other.neighbors
    }
}

impl Eq for Cluster {
    // fn eq(&self, other: &Self) -> bool {
    //     self.from == other.from && self.to == other.to && self.size == other.size
    // }
}

impl Cluster {
    pub fn new(items: Set<String>, neighbors: Set<String>) -> Cluster {
        let neighbor_items = neighbors.difference(&items).to_owned();

        let id = {
            let mut values: Vec<&String> = items.items.iter().collect();
            values.sort();
            values.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join("-")
        };

        Cluster {
            items: items.clone(),
            neighbors: neighbor_items,
            id,
        }
    }

    /// Compute the similarity score between two Clusters. Similarity is
    /// The Jaccard index of the parent nodes for each cluster.

    pub fn similarity(&self, other_cluster: &Cluster) -> f32 {
        let source_nodes: HashSet<&String> =
            self.items.items.union(&other_cluster.items.items).collect();

        let intersection: HashSet<&String> = self
            .neighbors
            .items
            .intersection(&other_cluster.neighbors.items)
            .collect::<HashSet<&String>>();

        let intersection_less_source: HashSet<&&String> =
            intersection.difference(&source_nodes).collect();

        let union: HashSet<&String> = self
            .neighbors
            .items
            .union(&other_cluster.neighbors.items)
            .collect::<HashSet<&String>>();

        let union_less_source: HashSet<&&String> = union.difference(&source_nodes).collect();

        let similarity = intersection_less_source.len() as f32 / union_less_source.len() as f32;

        similarity
    }

    pub fn union(self, other_cluster: &Cluster) -> Cluster {
        let unioned_items = self.items.union(&other_cluster.items);
        let unioned_parents = self.neighbors.union(&other_cluster.neighbors);

        Cluster::new(unioned_items, unioned_parents)
    }

    pub fn difference(self, other_cluster: &Cluster) -> Cluster {
        let items = self.items.difference(&other_cluster.items);
        let parents = self.neighbors.difference(&other_cluster.neighbors);

        Cluster::new(
            Set::from_set(items.into_iter().cloned().collect()),
            Set::from_set(parents.into_iter().cloned().collect()),
        )
    }

    pub fn intersection(self, other_cluster: &Cluster) -> Cluster {
        let items = self.items.intersection(&other_cluster.items);
        let parents = self.neighbors.intersection(&other_cluster.neighbors);

        Cluster::new(items, parents)
    }

    pub fn add_item(&mut self, item: String) {
        self.items.insert(item);
    }

    pub fn add_neighbor(&mut self, neighbor: String) {
        self.neighbors.insert(neighbor);
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_neighbors(&self) -> Vec<String> {
        return self.neighbors.to_vec();
    }

    pub fn get_items(&self) -> Vec<String> {
        return self.items.to_vec();
    }

    pub fn size(&self) -> usize {
        return self.items.len();
    }
}

pub fn generate_comparison_set(clusters: &Vec<Cluster>) -> Set<UnorderedTuple<String>> {
    let mut comparison_set: Set<UnorderedTuple<String>> = Set::new();

    for cluster in clusters {
        for comparison_cluster in clusters {
            if cluster == comparison_cluster {
                continue;
            }

            let cluster_parents = Set::from_iter(cluster.get_neighbors());
            let comparison_cluster_parents = Set::from_iter(comparison_cluster.get_neighbors());

            if cluster_parents
                .intersection(&comparison_cluster_parents)
                .len()
                == 0
            {
                continue;
            }

            comparison_set.insert(UnorderedTuple {
                one: cluster.get_id().to_owned(),
                two: comparison_cluster.get_id().to_owned(),
            });
        }
    }

    return comparison_set;
}

#[cfg(test)]
mod tests {

    use crate::{
        clusters::{generate_comparison_set, Cluster, UnorderedTuple},
        sets::Set,
    };

    #[test]
    fn trivial_positive_case() {
        let set1 = Cluster::new(
            Set::from_iter(vec!["2".to_string()]),
            Set::from_iter(vec!["1".to_string()]),
        );
        let set2 = Cluster::new(
            Set::from_iter(vec!["2".to_string()]),
            Set::from_iter(vec!["1".to_string()]),
        );

        assert_eq!(set1.similarity(&set2), 1.0_f32)
    }

    #[test]
    fn trivial_negative_case() {
        let set1 = Cluster::new(
            Set::from_iter(vec!["2".to_string()]),
            Set::from_iter(vec!["1".to_string()]),
        );
        let set2 = Cluster::new(
            Set::from_iter(vec!["3".to_string()]),
            Set::from_iter(vec!["2".to_string()]),
        );

        assert_eq!(set1.similarity(&set2), 0.0_f32)
    }

    #[test]
    fn nontrivial_case() {
        let set1 = Cluster::new(
            Set::from_iter(vec!["2".to_string(), "3".to_string()]),
            Set::from_iter(vec!["0".to_string(), "1".to_string()]),
        );
        let set2 = Cluster::new(
            Set::from_iter(vec!["4".to_string(), "5".to_string()]),
            Set::from_iter(vec![
                "0".to_string(),
                "6".to_string(),
                "5".to_string(),
                "7".to_string(),
            ]),
        );

        assert_eq!(set1.similarity(&set2), 0.25_f32)
    }

    #[test]
    fn comparison_sets() {
        let set1 = Cluster::new(
            Set::from_iter(vec!["1".to_string()]),
            Set::from_iter(vec![]),
        );
        let set2 = Cluster::new(
            Set::from_iter(vec!["2".to_string()]),
            Set::from_iter(vec!["1".to_string()]),
        );
        let set3 = Cluster::new(
            Set::from_iter(vec!["3".to_string()]),
            Set::from_iter(vec!["1".to_string()]),
        );
        let set4 = Cluster::new(
            Set::from_iter(vec!["4".to_string()]),
            Set::from_iter(vec!["2".to_string()]),
        );
        let clusters = vec![set1, set2, set3, set4];

        let comparison_set = generate_comparison_set(&clusters);
        assert_eq!(
            comparison_set,
            Set::from_iter(vec![UnorderedTuple {
                one: "2".to_string(),
                two: "3".to_string()
            }])
        );

        assert_eq!(
            comparison_set,
            Set::from_iter(vec![UnorderedTuple {
                one: "3".to_string(),
                two: "2".to_string()
            }])
        );
    }
}
