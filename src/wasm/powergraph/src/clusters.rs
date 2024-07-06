use crate::sets::Set;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct Cluster {
    items: Set<usize>,
    parents: Set<usize>,
}

#[wasm_bindgen]
impl Cluster {
    #[wasm_bindgen(constructor)]
    pub fn new(items: Vec<usize>, parents: Vec<usize>) -> Cluster {
        Cluster {
            items: Set::from_iter(items),
            parents: Set::from_iter(parents),
        }
    }

    /// Compute the similarity score between two Clusters. Similarity is
    /// The Jaccard index of the parent nodes for each cluster.
    #[wasm_bindgen]
    pub fn similarity(&self, other_cluster: &Cluster) -> f32 {
        let intersection = self.parents.intersection(&other_cluster.parents);
        let union = self.parents.union(&other_cluster.parents);

        let similarity = intersection.len() as f32 / union.len() as f32;

        println!(
            "{:?} <-> {:?}: {:?} {:?} => {:?}",
            self, other_cluster, intersection, union, similarity
        );
        similarity
    }

    pub fn union(self, other_cluster: &Cluster) -> Cluster {
        let unioned_items = self.items.union(&other_cluster.items);
        let unioned_parents = self.parents.union(&other_cluster.parents);
        Cluster::new(unioned_items.to_vec(), unioned_parents.to_vec())
    }

    pub fn add_item(&mut self, item: usize) {
        self.items.insert(item);
    }

    pub fn add_parent(&mut self, parent: usize) {
        self.items.insert(parent);
    }

    pub fn get_parents(&self) -> Vec<usize> {
        return self.parents.to_vec();
    }

    pub fn get_items(&self) -> Vec<usize> {
        return self.items.to_vec();
    }
}

pub fn generate_comparison_set(clusters: &Vec<Cluster>) -> Set<(usize, usize)> {
    let mut comparison_set: Set<(usize, usize)> = Set::new();

    for (index, cluster) in clusters.iter().enumerate() {
        for (comparison_index, comparison_cluster) in clusters.iter().enumerate() {
            if cluster == comparison_cluster {
                continue;
            }

            let cluster_parents = Set::from_iter(cluster.get_parents());
            let comparison_cluster_parents = Set::from_iter(comparison_cluster.get_parents());

            if cluster_parents
                .intersection(&comparison_cluster_parents)
                .len()
                == 0
            {
                continue;
            }

            if comparison_index > index {
                comparison_set.insert((index, comparison_index));
            } else {
                comparison_set.insert((comparison_index, index));
            }
        }
    }

    return comparison_set;
}

#[cfg(test)]
mod tests {

    use crate::{
        clusters::{generate_comparison_set, Cluster},
        sets::Set,
    };

    #[test]
    fn trivial_positive_case() {
        let set1 = Cluster::new(vec![2], vec![1_usize]);
        let set2 = Cluster::new(vec![2], vec![1_usize]);

        assert_eq!(set1.similarity(&set2), 1.0_f32)
    }

    #[test]
    fn trivial_negative_case() {
        let set1 = Cluster::new(vec![2], vec![1_usize]);
        let set2 = Cluster::new(vec![3], vec![2_usize]);

        assert_eq!(set1.similarity(&set2), 0.0_f32)
    }

    #[test]
    fn nontrivial_case() {
        let set1 = Cluster::new(vec![2, 3], vec![0, 1]);
        let set2 = Cluster::new(vec![4, 5], vec![0, 6, 5, 7]);

        assert_eq!(set1.similarity(&set2), 0.2_f32)
    }

    #[test]
    fn comparison_sets() {
        let set1 = Cluster::new(vec![1], vec![]);
        let set2 = Cluster::new(vec![2], vec![1]);
        let set3 = Cluster::new(vec![3], vec![1]);
        let set4 = Cluster::new(vec![4], vec![2]);
        let clusters = vec![set1, set2, set3, set4];

        let comparison_set = generate_comparison_set(&clusters);
        assert_eq!(comparison_set, Set::from_iter(vec![(1, 2)]));
    }
}
