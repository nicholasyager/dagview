use std::collections::HashMap;

// #[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct SimilarityMatrix {
    matrix: HashMap<usize, HashMap<usize, f32>>,
}

// #[wasm_bindgen]
impl SimilarityMatrix {
    pub fn new() -> SimilarityMatrix {
        let matrix = HashMap::new();

        SimilarityMatrix {
            matrix, // active_clusters,
        }
    }

    /// Start tracking another element. Add A new row and column to
    /// the matrix, and add the index to the cluster_index map.
    pub fn add_element(&mut self, element_id: usize) {
        self.matrix.insert(element_id, HashMap::new());
    }

    pub fn remove_element(&mut self, element_id: usize) {
        println!("Removing {:?} from the matrix.", element_id);
        self.matrix.remove(&element_id);
        for (_, row) in &mut self.matrix {
            row.remove(&element_id);
        }
    }

    /// Set the similarity between two clusters based on their index.
    pub fn set_similarity(
        &mut self,
        cluster_index: usize,
        comparison_cluster_index: usize,
        similarity: f32,
    ) {
        if !self.matrix.contains_key(&cluster_index) {
            self.add_element(cluster_index);
        }

        let row = self.matrix.get_mut(&cluster_index).unwrap();
        row.insert(comparison_cluster_index, similarity);
    }

    // Get the column and row with the largest similarity score
    pub fn get_max_similarity(&self) -> (usize, usize, f32) {
        let mut values: Vec<(usize, usize, f32)> = self
            .matrix
            .iter()
            .flat_map(|(index, row)| {
                row.iter().map(|(col_index, similarity)| {
                    (index.clone(), col_index.clone(), similarity.clone())
                })
            })
            .collect();

        values.sort_by_key(|value| (value.2 * 10000_f32) as u32);
        values.reverse();

        return values[0];
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
        let mut matrix = SimilarityMatrix::new();
        assert_eq!(matrix.len(), 0);

        matrix.add_element(1);
        assert_eq!(matrix.len(), 1);
    }

    #[test]
    fn remove_element() {
        let mut matrix = SimilarityMatrix::new();
        for item in 0..10 {
            matrix.add_element(item);
            let row = matrix.matrix.get_mut(&item).unwrap();
            for value in 0..10_usize {
                row.insert(value, 0.0_f32);
            }
        }
        assert_eq!(matrix.len(), 10);

        matrix.remove_element(9_usize);
        assert_eq!(matrix.len(), 9);

        for (_, row) in matrix.matrix {
            assert_eq!(row.len(), 9);
            assert_eq!(row.contains_key(&9_usize), false)
        }
    }
}
