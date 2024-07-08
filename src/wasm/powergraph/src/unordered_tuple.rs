#[derive(Debug, Clone, Hash)]
pub struct UnorderedTuple<T> {
    pub one: T,
    pub two: T,
}

impl<T: PartialEq> PartialEq for UnorderedTuple<T> {
    fn eq(&self, other: &Self) -> bool {
        (self.one == other.one && self.two == other.two)
            || (self.one == other.two && self.two == other.one)
    }
}

impl<T: PartialEq> Eq for UnorderedTuple<T> {}
