use crate::sets::Set;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Cluster {
    items: Set<u32>,
    parents: Set<u32>,
}

#[wasm_bindgen]
impl Cluster {
    #[wasm_bindgen(constructor)]
    pub fn new(items: Vec<u32>, parents: Vec<u32>) -> Cluster {
        Cluster {
            items: Set::from_iter(items),
            parents: Set::from_iter(parents),
        }
    }

    /// Compute the similarity score between two Clusters. Similarity is
    /// The Jaccard index of the parent nodes for each cluster.
    #[wasm_bindgen]
    pub fn similarity(&self, other_cluster: Cluster) -> f32 {
        let intersection = self.parents.intersection(&other_cluster.parents);
        let union = self.parents.union(&other_cluster.parents);

        return intersection.len() as f32 / union.len() as f32;
    }
}

#[cfg(test)]
mod tests {

    use crate::clusters::Cluster;

    #[test]
    fn trivial_positive_case() {
        let set1 = Cluster::new(vec![2], vec![1_u32]);
        let set2 = Cluster::new(vec![2], vec![1_u32]);

        assert_eq!(set1.similarity(set2), 1.0_f32)
    }

    #[test]
    fn trivial_negative_case() {
        let set1 = Cluster::new(vec![2], vec![1_u32]);
        let set2 = Cluster::new(vec![3], vec![2_u32]);

        assert_eq!(set1.similarity(set2), 0.0_f32)
    }

    #[test]
    fn nontrivial_case() {
        let set1 = Cluster::new(vec![2, 3], vec![0, 1]);
        let set2 = Cluster::new(vec![4, 5], vec![0, 6, 5, 7]);

        assert_eq!(set1.similarity(set2), 0.2_f32)
    }
}
