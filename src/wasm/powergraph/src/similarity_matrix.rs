use std::collections::HashSet;

// #[wasm_bindgen]
pub struct SimilarityMatrix {
    matrix: Vec<Vec<f32>>,
}

// #[wasm_bindgen]
impl SimilarityMatrix {
    pub fn new(size: usize) -> SimilarityMatrix {
        let mut matrix: Vec<Vec<f32>> = Vec::new();
        let mut active_clusters: HashSet<usize> = HashSet::new();

        for index in 0..size {
            let new_vector = vec![0.0_f32; size];
            matrix.push(new_vector);
            active_clusters.insert(index);
        }

        SimilarityMatrix {
            matrix,
            // active_clusters,
        }
    }

    /// Start tracking another element. Add A new row and column to
    /// the matrix, and add the index to the cluster_index map.
    pub fn add_element(&mut self) {
        for row in &mut self.matrix {
            row.push(0.0_f32);
        }
        let new_vector = vec![0.0_f32; self.matrix.len() + 1];
        self.matrix.push(new_vector);
    }

    pub fn remove_element(&mut self, index: usize) {
        self.matrix.remove(index);
        for row in &mut self.matrix {
            row.remove(index);
        }
    }

    pub fn len(&self) -> usize {
        return self.matrix.len();
    }
}

#[cfg(test)]
mod test {
    use super::SimilarityMatrix;

    #[test]
    fn add_element() {
        let mut matrix = SimilarityMatrix::new(0_usize);
        assert_eq!(matrix.len(), 0);

        matrix.add_element();
        assert_eq!(matrix.len(), 1);
        assert_eq!(matrix.matrix[0].len(), 1);
    }

    // #[test]
    // fn add_element() {
    //     let mut matrix = SimilarityMatrix::new(0_usize);
    //     assert_eq!(matrix.len(), 0);

    //     matrix.add_element();
    //     assert_eq!(matrix.len(), 1);
    //     assert_eq!(matrix.matrix[0].len(), 1);
    // }
}
