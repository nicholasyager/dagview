use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct UnorderedTuple<T> {
    pub one: T,
    pub two: T,
}

impl<T: std::hash::Hash + Ord + Clone> std::hash::Hash for UnorderedTuple<T> {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        let items: Vec<&T> = vec![&self.one, &self.two];

        for item in items.iter().sorted() {
            item.hash(state);
        }
    }
}

impl<T: PartialEq> PartialEq for UnorderedTuple<T> {
    fn eq(&self, other: &Self) -> bool {
        (self.one == other.one && self.two == other.two)
            || (self.one == other.two && self.two == other.one)
    }
}

impl<T: PartialEq> Eq for UnorderedTuple<T> {}
