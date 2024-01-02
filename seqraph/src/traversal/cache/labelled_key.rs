use crate::shared::*;

#[cfg(test)]
mod vkey {
    use super::*;
    pub type VertexCacheKey = LabelledKey;
    pub fn labelled_key<Trav: Traversable>(trav: &Trav, child: Child) -> VertexCacheKey
        where TravToken<Trav>: Display
    {
        LabelledKey::build(trav, child)
    }
    macro_rules! lab {
        ($x:ident) => {
            LabelledKey::new($x, stringify!($x))
        }
    }
    pub(crate) use lab;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct LabelledKey {
        index: VertexIndex,
        label: String,
    }
    impl LabelledKey {
        pub fn new(child: impl Borrow<Child>, label: impl ToString) -> Self {
            Self {
                label: label.to_string(),            
                index: child.borrow().vertex_index(),
            }
        }
        pub fn build<Trav: Traversable>(trav: &Trav, child: Child) -> Self
            where TravToken<Trav>: Display
        {
            let index = child.vertex_index();
            Self {
                label: trav.graph().index_string(index),            
                index,
            }
        }
    }
    impl Borrow<VertexIndex> for LabelledKey {
        fn borrow(&self) -> &VertexIndex {
            &self.index
        }
    }
    impl Hash for LabelledKey {
        fn hash<H: Hasher>(&self, h: &mut H) {
            self.index.hash(h)
        }
    }
    impl Display for LabelledKey {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.label)
        }
    }
}

#[cfg(not(test))]
mod vkey {
    use super::*;
    pub type VertexCacheKey = VertexIndex;
    pub fn labelled_key<Trav: Traversable>(_trav: &Trav, child: Child) -> VertexCacheKey
        where TravToken<Trav>: Display
    {
        child.vertex_index()
    }
}
pub use vkey::*;