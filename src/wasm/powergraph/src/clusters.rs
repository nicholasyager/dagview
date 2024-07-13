use serde::Serialize;

use crate::{sets::Set, unordered_tuple::UnorderedTuple};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Cluster {
    pub items: Set<String>,
    neighbors: Set<String>,
}

impl Cluster {
    pub fn new(items: Set<String>, neighbors: Set<String>) -> Cluster {
        Cluster {
            items: items.clone(),
            neighbors: neighbors.difference(&items),
        }
    }

    /// Compute the similarity score between two Clusters. Similarity is
    /// The Jaccard index of the parent nodes for each cluster.

    pub fn similarity(&self, other_cluster: &Cluster) -> f32 {
        let intersection = self
            .neighbors
            .intersection(&other_cluster.neighbors)
            .difference(&self.items)
            .difference(&other_cluster.items);

        let union = self
            .neighbors
            .union(&other_cluster.neighbors)
            .difference(&self.items)
            .difference(&other_cluster.items);

        let similarity = intersection.len() as f32 / union.len() as f32;

        println!(
            "{:?} <-> {:?}: {:?} {:?} => {:?}",
            self, other_cluster, intersection, union, similarity
        );
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

        Cluster::new(items, parents)
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

    pub fn get_id(&self) -> String {
        let mut values = self
            .items
            .to_vec()
            .into_iter()
            .map(|item| item.to_string())
            .collect::<Vec<String>>();

        values.sort();

        return values.join("-").clone();
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
                one: cluster.get_id().to_string(),
                two: comparison_cluster.get_id().to_string(),
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
