#[derive(Debug, Clone, Hash)]
pub struct UnorderedTuple {
    pub one: String,
    pub two: String,
}

impl PartialEq for UnorderedTuple {
    fn eq(&self, other: &Self) -> bool {
        (self.one == other.one && self.two == other.two)
            || (self.one == other.two && self.two == other.one)
    }
}

impl Eq for UnorderedTuple {}
