use std::collections::HashSet;

use crate::clusters::Cluster;

// #[wasm_bindgen]
pub struct SimilarityMatrix {
    matrix: Vec<Vec<f32>>,
    inactive_clusters: HashSet<u32>,
}

// #[wasm_bindgen]
impl SimilarityMatrix {
    pub fn new(clusters: Vec<Cluster>) -> SimilarityMatrix {
        let mut matrix: Vec<Vec<f32>> = Vec::new();
        let size = clusters.len();

        for _ in clusters {
            let new_vector = vec![0.0_f32; size];
            matrix.push(new_vector);
        }

        SimilarityMatrix {
            matrix: matrix,
            inactive_clusters: HashSet::new(),
        }
    }
}
