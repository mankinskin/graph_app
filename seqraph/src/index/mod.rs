//use std::collections::HashMap;

use crate::{
    vertex::*,
    search::*,
    HypergraphRef,
};
use std::{borrow::Borrow, ops::{RangeFrom, RangeInclusive}};

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
    pub(crate) fn index_prefix(
        &self,
        pattern: impl IntoPattern,
    ) -> Result<Child, NoMatch> {
        self.indexer().index_prefix(pattern)
    }
}
pub(crate) enum ContextHalf {
    Child(Child),
    Pattern(Pattern),
}
impl ContextHalf {
    pub fn try_new(p: impl IntoPattern) -> Option<Self> {
        let p = p.borrow();
        match p.len() {
            0 => None,
            1 => Some(Self::Child(*p.into_iter().next().unwrap())),
            _ => Some(Self::Pattern(p.into_pattern())),
        }
    }
    pub fn unwrap_child(self) -> Child {
        match self {
            Self::Child(c) => c,
            _ => panic!("Failed to unwrap ContextHalf::Child!"),
        }
    }
    pub fn expect_child(self, msg: &str) -> Child {
        match self {
            Self::Child(c) => c,
            _ => panic!("Expected ContextHalf::Child!: {}", msg),
        }
    }
}

impl Borrow<[Child]> for ContextHalf {
    fn borrow(&self) -> &[Child] {
        match self {
            Self::Child(c) => std::slice::from_ref(c),
            Self::Pattern(p) => p.borrow(),
        }
    }
}
impl AsRef<[Child]> for ContextHalf {
    fn as_ref(&self) -> &[Child] {
        self.borrow()
    }
}

enum PathRootHalf {
    Perfect(Pattern),
    Unperfect(ContextHalf, Child),
}

pub(crate) struct IndexSplitResult {
    inner: Child,
    location: ChildLocation,
    context: Vec<ChildLocation>,
}

#[macro_use]
#[cfg(test)]
#[allow(clippy::many_single_char_names)]
pub(crate) mod tests {

    use super::*;
    use crate::{
        Hypergraph,
        QueryRangePath,
    };
    use pretty_assertions::assert_eq;
    use itertools::*;

    #[test]
    fn index_prefix1() {
        let mut graph = Hypergraph::default();
        let (a, b, _w, x, y, z) = graph.index_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('w'),
            Token::Element('x'),
            Token::Element('y'),
            Token::Element('z'),
        ]).into_iter().next_tuple().unwrap();
        let ab = graph.index_pattern([a, b]);
        let by = graph.index_pattern([b, y]);
        let yz = graph.index_pattern([y, z]);
        let xa = graph.index_pattern([x, a]);
        let xab = graph.index_patterns([[x, ab], [xa, b]]);
        let xaby = graph.index_patterns([vec![xab, y], vec![xa, by]]);
        let _xabyz = graph.index_patterns([vec![xaby, z], vec![xab, yz]]);
        let graph = HypergraphRef::from(graph);
        let query = vec![by, z];
        let byz = graph.index_prefix(query.borrow()).expect("Indexing failed");
        let byz_found = graph.find_parent(&query);
        assert_eq!(
            byz_found,
            Ok(QueryFound::complete(query, byz)),
            "byz"
        );
        let query = vec![ab, y];
        let aby = graph.index_prefix(query.borrow()).expect("Indexing failed");
        let aby_found = graph.find_parent(&query);
        assert_eq!(
            aby_found,
            Ok(QueryFound::complete(query, aby)),
            "aby"
        );
    }
    #[test]
    fn index_prefix2() {
        let mut graph = Hypergraph::default();
        let (a, b, _w, x, y, z) = graph.index_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('w'),
            Token::Element('x'),
            Token::Element('y'),
            Token::Element('z'),
        ]).into_iter().next_tuple().unwrap();
        // index 6
        let yz = graph.index_pattern([y, z]);
        let xab = graph.index_pattern([x, a, b]);
        let _xyz = graph.index_pattern([x, yz]);
        let _xabz = graph.index_pattern([xab, z]);
        let _xabyz = graph.index_pattern([xab, yz]);

        let graph = HypergraphRef::from(graph);

        let query = vec![a, b, y, x];
        let aby = graph.index_prefix(query.borrow()).expect("Indexing failed");
        let query = vec![a, b, y];
        let aby_found = graph.find_ancestor(&query);
        assert_eq!(
            aby_found,
            Ok(QueryFound {
                found: FoundPath::Complete(aby),
                query: QueryRangePath {
                    entry: 0,
                    exit: 2,
                    start: vec![],
                    end: vec![],
                    query,
                }
            }),
            "aby"
        );
    }
}