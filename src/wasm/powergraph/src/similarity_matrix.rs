use itertools::Itertools;

use crate::unordered_tuple::UnorderedTuple;
use std::collections::HashMap;

// #[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct SimilarityMatrix {
    matrix: Vec<(UnorderedTuple<String>, f32)>,
}

// #[wasm_bindgen]
impl SimilarityMatrix {
    pub fn new() -> SimilarityMatrix {
        let matrix = Vec::new();

        SimilarityMatrix {
            matrix, // active_clusters,
        }
    }

    pub fn remove_element(&mut self, element_id: String) {
        // println!("Removing {:?} from the matrix.", element_id);

        for index in self
            .matrix
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                if item.0.one == element_id || item.0.two == element_id {
                    return Some(index);
                }
                None
            })
            .sorted()
            .rev()
        {
            self.matrix.remove(index);
        }
    }

    /// Set the similarity between two clusters based on their index.
    pub fn set_similarity(&mut self, index: UnorderedTuple<String>, similarity: f32) {
        // self.matrix.insert(index, similarity);
        match self.matrix.binary_search_by(|value| value.0.cmp(&index)) {
            Ok(pos) => self.matrix[pos] = (index, similarity),
            Err(_) => {
                let pos = self.matrix.partition_point(|value| value.1 > similarity);
                self.matrix.insert(pos, (index, similarity));
            }
        }
    }

    // Get the column and row with the largest similarity score
    pub fn get_max_similarity(&self) -> Option<(UnorderedTuple<String>, f32)> {
        if self.matrix.len() == 0 {
            return None;
        }

        return Some(self.matrix[0].clone());
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
            matrix.get_max_similarity().unwrap(),
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
