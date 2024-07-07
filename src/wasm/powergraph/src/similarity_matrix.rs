use crate::unordered_tuple::UnorderedTuple;
use std::collections::HashMap;

// #[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct SimilarityMatrix {
    matrix: HashMap<UnorderedTuple, f32>,
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
    // pub fn add_element(&mut self, element_id: &String) {
    //     self.matrix.insert(element_id.clone(), HashMap::new());
    // }

    pub fn remove_element(&mut self, element_id: String) {
        println!("Removing {:?} from the matrix.", element_id);

        let remove_list: Vec<UnorderedTuple> = self
            .matrix
            .clone()
            .into_iter()
            .filter_map(|(key, _)| {
                if key.one == element_id || key.two == element_id {
                    return Some(key);
                }
                None
            })
            .collect();

        for key in remove_list {
            self.matrix.remove(&key);
        }
    }

    /// Set the similarity between two clusters based on their index.
    pub fn set_similarity(&mut self, index: UnorderedTuple, similarity: f32) {
        self.matrix.insert(index, similarity);
    }

    // Get the column and row with the largest similarity score
    pub fn get_max_similarity(&self) -> (UnorderedTuple, f32) {
        let mut values: Vec<(UnorderedTuple, f32)> = self
            .matrix
            .iter()
            .map(|(index, similarity)| (index.clone(), similarity.clone()))
            .collect();

        values.sort_by_key(|value| (value.1 * 10000_f32) as u32);
        values.reverse();

        return values[0].clone();
    }

    pub fn len(&self) -> usize {
        return self.matrix.len();
    }
}

#[cfg(test)]
mod test {

    use crate::similarity_matrix::UnorderedTuple;

    use super::SimilarityMatrix;

    #[test]
    fn remove_element() {
        let mut matrix = SimilarityMatrix::new();

        matrix.set_similarity(
            UnorderedTuple {
                one: "foo".to_string(),
                two: "bar".to_string(),
            },
            0.1,
        );
        matrix.set_similarity(
            UnorderedTuple {
                one: "foo".to_string(),
                two: "baz".to_string(),
            },
            0.75,
        );
        matrix.set_similarity(
            UnorderedTuple {
                one: "foo".to_string(),
                two: "buzz".to_string(),
            },
            0.7,
        );

        assert_eq!(matrix.len(), 3);

        matrix.remove_element("buzz".to_string());
        assert_eq!(matrix.len(), 2);
        assert_eq!(
            matrix.get_max_similarity(),
            (
                UnorderedTuple {
                    one: "foo".to_string(),
                    two: "baz".to_string(),
                },
                0.75
            )
        );
    }
}
