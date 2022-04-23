//use std::collections::HashMap;

use crate::{
    vertex::*,
    search::*,
    HypergraphRef, QueryRangePath,
};
use std::ops::RangeFrom;

mod indexer;
pub use indexer::*;
mod index_direction;
pub use index_direction::*;

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
    ) -> Result<(Child, QueryRangePath), NoMatch> {
        self.indexer().index_prefix(pattern)
    }
    pub(crate) fn index_path_prefix(
        &self,
        query: QueryRangePath,
    ) -> Result<(Child, QueryRangePath), NoMatch> {
        self.indexer().index_path_prefix(query)
    }
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
        QueryRangePath, Traversable,
    };
    use pretty_assertions::assert_eq;
    use itertools::*;
    use std::{borrow::Borrow, collections::{HashMap, HashSet}};

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
        // index 6
        let ab = graph.index_pattern([a, b]);
        let by = graph.index_pattern([b, y]);
        let yz = graph.index_pattern([y, z]);
        let xa = graph.index_pattern([x, a]);
        let xab = graph.index_patterns([[x, ab], [xa, b]]);
        let xaby = graph.index_patterns([vec![xab, y], vec![xa, by]]);
        let _xabyz = graph.index_patterns([vec![xaby, z], vec![xab, yz]]);
        let graph = HypergraphRef::from(graph);
        let query = vec![by, z];
        let (byz, _) = graph.index_prefix(query.borrow()).expect("Indexing failed");
        assert_eq!(byz, Child {
            index: 13,
            width: 3,
        }, "byz");
        let byz_found = graph.find_ancestor(&query);
        assert_eq!(
            byz_found,
            Ok(QueryFound::complete(query, byz)),
            "byz"
        );
        let query = vec![ab, y];
        let (aby, _) = graph.index_prefix(query.borrow()).expect("Indexing failed");
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

        let graph_ref = HypergraphRef::from(graph);

        let query = vec![a, b, y, x];
        let (aby, _) = graph_ref.index_prefix(query.borrow()).expect("Indexing failed");
        assert_eq!(aby, Child {
            index: 12,
            width: 3,
        }, "aby");
        let graph = graph_ref.read().unwrap();
        let aby_vertex = graph.expect_vertex_data(aby);
        assert_eq!(aby_vertex.parents.len(), 1, "aby");
        assert_eq!(aby_vertex.children.len(), 1, "aby");
        drop(graph);
        let query = vec![a, b, y];
        let aby_found = graph_ref.find_ancestor(&query);
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