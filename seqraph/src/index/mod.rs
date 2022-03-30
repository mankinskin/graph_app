//use std::collections::HashMap;

use crate::{
    vertex::*,
    search::*,
    HypergraphRef,
};
use std::borrow::Borrow;

mod indexer;
pub use indexer::*;
mod index_direction;
pub use index_direction::*;

//#[derive(Debug, Clone, PartialEq, Eq)]
//pub struct IndexedPath {
//    pub(crate) indexed: IndexedChild,
//    pub(crate) end_path: Option<ChildPath>,
//    pub(crate) remainder: Option<Pattern>
//}
//#[derive(Debug, Clone, PartialEq, Eq)]
//pub struct IndexedChild {
//    pub(crate) location: ChildLocation,
//    pub(crate) context: Option<Child>,
//    pub(crate) inner: Child,
//}
//impl IndexedPath {
//    pub fn new(indexed: IndexedChild, end_path: ChildPath, remainder: impl IntoPattern) -> Self {
//        Self {
//            indexed,
//            end_path: if end_path.is_empty() {
//                None
//            } else {
//                Some(end_path)
//            },
//            remainder: if remainder.is_empty() {
//                None
//            } else {
//                Some(remainder.into_pattern())
//            },
//        }
//    }
//}
//type IndexingResult = Result<IndexedPath, NoMatch>;

impl<'t, 'g, T> HypergraphRef<T>
where
    T: Tokenize + 't,
{
    pub fn indexer(&self) -> Indexer<T, Right> {
        Indexer::new(self.clone())
    }
    #[allow(unused)]
    pub(crate) fn index_pattern(
        &self,
        pattern: impl IntoPattern,
    ) -> Result<Child, NoMatch> {
        self.indexer().index_pattern(pattern)
    }
}

pub(crate) enum ContextHalf {
    Child(Child),
    Pattern(Pattern),
}
impl Borrow<[Child]> for ContextHalf {
    fn borrow(&self) -> &[Child] {
        match self {
            Self::Child(c) => std::slice::from_ref(c),
            Self::Pattern(p) => p.borrow(),
        }
    }
}

#[macro_use]
#[cfg(test)]
#[allow(clippy::many_single_char_names)]
pub(crate) mod tests {

    use super::*;
    use crate::Hypergraph;
    use pretty_assertions::{
        assert_eq,
    };
    use itertools::*;

    #[test]
    fn index_pattern1() {
        let mut graph = Hypergraph::default();
        let (a, b, _w, x, y, z) = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('w'),
            Token::Element('x'),
            Token::Element('y'),
            Token::Element('z'),
        ]).into_iter().next_tuple().unwrap();
        let ab = graph.insert_pattern([a, b]);
        let by = graph.insert_pattern([b, y]);
        let yz = graph.insert_pattern([y, z]);
        let xa = graph.insert_pattern([x, a]);
        let xab = graph.insert_patterns([[x, ab], [xa, b]]);
        let xaby = graph.insert_patterns([vec![xab, y], vec![xa, by]]);
        let _xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);
        let graph = HypergraphRef::from(graph);
        let query = vec![by, z];
        let byz = graph.index_pattern(query.borrow()).expect("Indexing failed");
        let byz_found = graph.find_parent(&query);
        assert_eq!(
            byz_found,
            Ok(QueryFound::complete(query, byz)),
            "byz"
        );
    }
}