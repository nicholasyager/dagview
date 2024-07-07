#[derive(Debug, Clone, PartialEq)]
pub struct Set<T> {
    items: Vec<T>,
}

impl<T: std::cmp::PartialEq + Clone> Set<T> {
    pub fn new() -> Set<T> {
        Set { items: Vec::new() }
    }

    pub fn from_iter(items: Vec<T>) -> Set<T> {
        Set { items }
    }

    pub fn insert(&mut self, item: T) {
        if self.contains(item.clone()) {
            return;
        }
        self.items.push(item);
    }

    pub fn contains(&self, item: T) -> bool {
        return self.items.contains(&item);
    }

    pub fn intersection(&self, other_cluster: &Set<T>) -> Set<T> {
        let mut common_items = Vec::new();

        for item in &self.items {
            if other_cluster.items.iter().any(|i| i == item) {
                common_items.push(item.clone())
            }
        }

        Set::from_iter(common_items)
    }

    pub fn union(&self, other_cluster: &Set<T>) -> Set<T> {
        let mut all_items = self.items.clone();

        for item in &other_cluster.items {
            if !all_items.iter().any(|i| i == item) {
                all_items.push(item.clone())
            }
        }

        Set::from_iter(all_items)
    }

    pub fn difference(&self, other_cluster: &Set<T>) -> Set<T> {
        let mut different_items = Vec::new();

        for item in &self.items {
            if !other_cluster.items.iter().any(|i| i == item) {
                different_items.push(item.clone())
            }
        }

        Set::from_iter(different_items)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn to_vec(&self) -> Vec<T> {
        return self.items.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clusters_intersect() {
        let cluster_a = Set::from_iter(vec![1, 2]);
        let cluster_b = Set::from_iter(vec![2, 3, 4]);

        let intersection = cluster_a.intersection(&cluster_b);
        assert_eq!(intersection.items, vec![2]);
    }

    #[test]
    fn clusters_union() {
        let cluster_a = Set::from_iter(vec![1, 2]);
        let cluster_b = Set::from_iter(vec![2, 3, 4]);

        let intersection = cluster_a.union(&cluster_b);
        assert_eq!(intersection.items, vec![1, 2, 3, 4]);
    }

    #[test]
    fn clusters_difference() {
        let cluster_a = Set::from_iter(vec![1, 2]);
        let cluster_b = Set::from_iter(vec![2, 3, 4]);

        let intersection = cluster_a.difference(&cluster_b);
        assert_eq!(intersection.items, vec![1]);
    }
}
