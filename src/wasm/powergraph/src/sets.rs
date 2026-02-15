use itertools::Itertools;
use serde::Serialize;
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Set<T: std::cmp::PartialEq + std::hash::Hash + std::cmp::Eq> {
    pub items: HashSet<T>,
}

impl<T: Hash + Eq> IntoIterator for Set<T> {
    type Item = T;
    type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<T: Eq + Hash + Ord> std::hash::Hash for Set<T> {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.items.iter().sorted().for_each(|item| {
            item.hash(state);
        })
    }
}


impl<'a, T: std::cmp::PartialEq + Clone + Hash + Eq> Set<&T> {
    pub fn to_owned(&'a self) -> Set<T> {
        Set::from_set(
            self.items
                .clone()
                .into_iter()
                .map(|item| item.clone())
                .collect(),
        )
    }
}

impl<'a, T: std::cmp::PartialEq + Clone + Hash + Eq> Set<T> {
    pub fn new() -> Set<T> {
        Set {
            items: HashSet::new(),
        }
    }

    pub fn from_iter(items: Vec<T>) -> Set<T> {
        let mut new_set = Set::new();
        for item in items {
            new_set.insert(item);
        }

        return new_set;
    }

    pub fn from_set(items: HashSet<T>) -> Set<T> {
        return Set { items };
    }

    pub fn insert(&mut self, item: T) {
        self.items.insert(item);
    }

    pub fn contains(&self, item: &T) -> bool {
        return self.items.contains(item);
    }

    pub fn intersection(&self, other_cluster: &Set<T>) -> Set<T> {
        return Set {
            items: self
                .items
                .intersection(&other_cluster.items)
                .map(|item| item.clone())
                .collect(),
        };
    }

    pub fn union(&self, other_cluster: &Set<T>) -> Set<T> {
        Set {
            items: self
                .items
                .union(&other_cluster.items)
                .map(|item| item.clone())
                .collect(),
        }
    }

    pub fn difference(&'a self, other_cluster: &'a Set<T>) -> Set<&'a T> {
        Set {
            items: self.items.difference(&other_cluster.items).collect(), // .map(|item| item.clone())
                                                                          // .collect(),
        }
    }

    pub fn symmetric_difference(&self, other_cluster: &Set<T>) -> Set<T> {
        Set {
            items: self
                .items
                .difference(&other_cluster.items)
                .map(|item| item.clone())
                .collect(),
        }
    }

    pub fn is_subset_of(&self, other_set: &Set<T>) -> bool {
        return self.difference(other_set).len() == 0;
    }

    pub fn is_proper_subset_of(&self, other_set: &Set<T>) -> bool {
        return self.difference(other_set).len() == 0 && other_set.difference(self).len() > 0;
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn to_vec(&self) -> Vec<T> {
        return self.items.clone().into_iter().collect::<Vec<T>>();
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<'_, T> {
        self.items.iter()
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
        let mut answer_set = HashSet::new();
        answer_set.insert(2);
        assert_eq!(intersection.items, answer_set);
    }

    #[test]
    fn clusters_union() {
        let cluster_a = Set::from_iter(vec![1, 2]);
        let cluster_b = Set::from_iter(vec![2, 3, 4]);

        let intersection = cluster_a.union(&cluster_b);
        let mut answer_set = HashSet::new();
        answer_set.insert(1);
        answer_set.insert(2);
        answer_set.insert(3);
        answer_set.insert(4);
        assert_eq!(intersection.items, answer_set);
    }

    #[test]
    fn clusters_difference() {
        let cluster_a = Set::from_iter(vec![1, 2]);
        let cluster_b = Set::from_iter(vec![2, 3, 4]);

        let intersection = cluster_a.difference(&cluster_b);

        let mut answer_set = HashSet::new();
        answer_set.insert(&1);
        assert_eq!(intersection.items, answer_set);
    }

    #[test]
    fn set_subset_detection() {
        let cluster_a = Set::from_iter(vec![1, 2]);
        let cluster_b = Set::from_iter(vec![1, 2, 3, 4]);

        let is_subset = cluster_a.is_subset_of(&cluster_b);
        assert!(is_subset);
    }

    #[test]
    fn set_subset_detection_negative() {
        let cluster_a = Set::from_iter(vec![1, 2]);
        let cluster_b = Set::from_iter(vec![1, 3, 4]);

        let is_subset = cluster_a.is_subset_of(&cluster_b);
        assert!(!is_subset);
    }
}
