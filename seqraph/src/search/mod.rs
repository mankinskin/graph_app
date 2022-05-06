use std::cmp::Ordering;

use crate::{
    vertex::*,
    *,
};
mod searcher;
pub use searcher::*;
mod match_direction;
pub use match_direction::*;
//mod async_searcher;
//pub use async_searcher::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NoMatch {
    EmptyPatterns,
    NoParents,
    NoChildPatterns,
    NotFound(Pattern),
    NoMatchingParent(VertexIndex),
    InvalidPattern(PatternId),
    InvalidPatternRange(PatternId, Pattern, String),
    SingleIndex,
    ParentMatchingPartially,
    UnknownKey,
    UnknownIndex,
}

pub trait ResultOrd: Wide {
    fn is_complete(&self) -> bool;
    fn cmp(&self, other: impl ResultOrd) -> Ordering {
        let l = self.is_complete();
        let r = other.is_complete();
        if l == r {
            self.width().cmp(&other.width())
        } else {
            l.cmp(&r)
        }
    }
    fn eq(&self, other: impl ResultOrd) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl<T: ResultOrd> ResultOrd for &T {
    fn is_complete(&self) -> bool {
        ResultOrd::is_complete(*self)
    }
}
impl ResultOrd for GraphRangePath {
    fn is_complete(&self) -> bool {
        false
    }
}
impl ResultOrd for FoundPath {
    fn is_complete(&self) -> bool {
        matches!(self, FoundPath::Complete(_))
    }
}
impl Wide for FoundPath {
    fn width(&self) -> usize {
        match self {
            Self::Complete(c) => c.width,
            Self::Range(r) => r.width(),
        }
    }
}
impl<Rhs: ResultOrd> PartialOrd<Rhs> for FoundPath {
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
        Some(ResultOrd::cmp(self, other))
    }
}
impl<Rhs: ResultOrd> PartialEq<Rhs> for FoundPath {
    fn eq(&self, other: &Rhs) -> bool {
        ResultOrd::eq(self, other)
    }
}
impl<Rhs: ResultOrd> PartialOrd<Rhs> for GraphRangePath {
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
        Some(ResultOrd::cmp(self, other))
    }
}
impl<Rhs: ResultOrd> PartialEq<Rhs> for GraphRangePath {
    fn eq(&self, other: &Rhs) -> bool {
        ResultOrd::eq(self, other)
    }
}
#[derive(Debug, Clone, Eq)]
pub(crate) enum FoundPath {
    Complete(Child),
    Range(GraphRangePath),
}
impl<
    'a: 'g,
    'g,
> FoundPath {
    pub(crate) fn new<
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(trav: &'a Trav, range_path: GraphRangePath) -> Self {
        if range_path.is_complete::<_, D, _>(trav) {
            Self::Complete(Into::<StartPath>::into(range_path).entry().parent)
        } else {
            Self::Range(range_path)
        }
    }
    #[track_caller]
    pub fn unwrap_complete(self) -> Child {
        match self {
            Self::Complete(index) => index,
            _ => panic!("Unable to unwrap {:?} as complete.", self),
        }
    }
    #[track_caller]
    pub fn expect_complete(self, msg: &str) -> Child {
        match self {
            Self::Complete(index) => index,
            _ => panic!("Unable to unwrap {:?} as complete: {}", self, msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryResult<Q: TraversalQuery> {
    pub(crate) found: FoundPath,
    pub(crate) query: Q,
}

impl<Q: TraversalQuery> QueryResult<Q> {
    pub(crate) fn new(found: impl Into<FoundPath>, query: Q) -> Self {
        Self {
            found: found.into(),
            query,
        }
    }
    #[track_caller]
    pub fn unwrap_complete(self) -> Child {
        self.found.unwrap_complete()
    }
    #[track_caller]
    pub fn expect_complete(self, msg: &str) -> Child {
        self.found.expect_complete(msg)
    }
}
impl<Q: QueryPath> QueryResult<Q> {
    pub fn complete(query: impl IntoPattern, index: impl AsChild) -> Self {
        Self {
            found: FoundPath::Complete(index.as_child()),
            query: Q::complete(query),
        }
    }
}
pub type QueryFound = QueryResult<QueryRangePath>;
pub type SearchResult = Result<QueryFound, NoMatch>;

impl<'t, 'g, T> HypergraphRef<T>
where
    T: Tokenize + 't,
{
    pub(crate) fn searcher<D: MatchDirection>(&'g self) -> Searcher<T, D> {
        Searcher::new(self.clone())
    }
    pub(crate) fn right_searcher(&'g self) -> Searcher<T, Right> {
        self.searcher()
    }
    #[allow(unused)]
    pub(crate) fn left_searcher(&'g self) -> Searcher<T, Left> {
        self.searcher()
    }
    pub fn expect_pattern(
        &self,
        pattern: impl IntoIterator<Item = impl AsToken<T>>,
    ) -> Child {
        self.find_sequence(pattern).unwrap().unwrap_complete()
    }
    pub(crate) fn find_ancestor(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
    ) -> SearchResult {
        let pattern = self.read().unwrap().to_children(pattern);
        self.right_searcher().find_pattern_ancestor(pattern)
    }
    #[allow(unused)]
    pub(crate) fn find_parent(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
    ) -> SearchResult {
        let pattern = self.read().unwrap().to_children(pattern);
        self.right_searcher().find_pattern_parent(pattern)
    }
    pub fn find_sequence(
        &self,
        pattern: impl IntoIterator<Item = impl AsToken<T>>,
    ) -> SearchResult {
        let iter = tokenizing_iter(pattern.into_iter());
        let pattern = self.read().unwrap().to_token_children(iter)?;
        self.find_ancestor(pattern)
    }
}
#[macro_use]
#[cfg(test)]
#[allow(clippy::many_single_char_names)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        graph::tests::context,
        Child,
        traversal::path::GraphRangePath,
    };
    use pretty_assertions::{
        assert_eq,
    };
    use itertools::*;

    #[test]
    fn find_parent1() {
        let Context {
            graph,
            a,
            b,
            c,
            d,
            ab,
            bc,
            abc,
            abcd,
            ..
         } = &*context();
        let a_bc_pattern = vec![Child::new(a, 1), Child::new(bc, 2)];
        let ab_c_pattern = vec![Child::new(ab, 2), Child::new(c, 1)];
        let a_bc_d_pattern = vec![Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
        let b_c_pattern = vec![Child::new(b, 1), Child::new(c, 1)];
        let bc_pattern = vec![Child::new(bc, 2)];
        let a_b_c_pattern = vec![Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];

        let query = bc_pattern;
        assert_eq!(
            graph.find_parent(&query),
            Err(NoMatch::SingleIndex),
            "bc"
        );
        let query = b_c_pattern;
        assert_eq!(
            graph.find_parent(&query),
            Ok(QueryFound::complete(query, bc)),
            "b_c"
        );
        let query = a_bc_pattern;
        assert_eq!(
            graph.find_parent(&query),
            Ok(QueryFound::complete(query, abc)),
            "a_bc"
        );
        let query = ab_c_pattern;
        assert_eq!(
            graph.find_parent(&query),
            Ok(QueryFound::complete(query, abc)),
            "ab_c"
        );
        let query = a_bc_d_pattern;
        assert_eq!(
            graph.find_parent(&query),
            Ok(QueryFound::complete(query, abcd)),
            "a_bc_d"
        );
        let query = a_b_c_pattern.clone();
        assert_eq!(
            graph.find_parent(&query),
            Ok(QueryFound::complete(query, abc)),
            "a_b_c"
        );
        let query = [&a_b_c_pattern[..], &[Child::new(c, 1)]].concat();
        assert_eq!(
            graph.find_parent(&query),
            Ok(QueryFound {
                found: FoundPath::Complete(*abc),
                query: QueryRangePath {
                    exit: query.len() - 2,
                    query,
                    entry: 0,
                    start: vec![],
                    end: vec![],
                    finished: false,
                },
            }),
            "a_b_c_c"
        );
    }
    #[test]
    fn find_ancestor1() {
        let Context {
            graph,
            a,
            b,
            c,
            d,
            e,
            f,
            g,
            h,
            i,
            ab,
            bc,
            abc,
            abcd,
            ababababcdefghi,
            ..
         } = &*context();
        let a_bc_pattern = vec![Child::new(a, 1), Child::new(bc, 2)];
        let ab_c_pattern = vec![Child::new(ab, 2), Child::new(c, 1)];
        let a_bc_d_pattern = vec![Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
        let b_c_pattern = vec![Child::new(b, 1), Child::new(c, 1)];
        let bc_pattern = vec![Child::new(bc, 2)];
        let a_b_c_pattern = vec![Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];

        let query = bc_pattern;
        assert_eq!(
            graph.find_ancestor(&query),
            Err(NoMatch::SingleIndex),
            "bc"
        );
        let query = b_c_pattern;
        assert_eq!(
            graph.find_ancestor(&query),
            Ok(QueryFound::complete(query, bc)),
            "b_c"
        );
        let query = a_bc_pattern;
        assert_eq!(
            graph.find_ancestor(&query),
            Ok(QueryFound::complete(query, abc)),
            "a_bc"
        );
        let query = ab_c_pattern;
        assert_eq!(
            graph.find_ancestor(&query),
            Ok(QueryFound::complete(query, abc)),
            "ab_c"
        );
        let query = a_bc_d_pattern;
        assert_eq!(
            graph.find_ancestor(&query),
            Ok(QueryFound::complete(query, abcd)),
            "a_bc_d"
        );
        let query = a_b_c_pattern.clone();
        assert_eq!(
            graph.find_ancestor(&query),
            Ok(QueryFound::complete(query, abc)),
            "a_b_c"
        );
        let query =
            vec![*a, *b, *a, *b, *a, *b, *a, *b, *c, *d, *e, *f, *g, *h, *i];
        assert_eq!(
            graph.find_ancestor(&query),
            Ok(QueryFound::complete(query, ababababcdefghi)),
            "a_b_a_b_a_b_a_b_c_d_e_f_g_h_i"
        );
        let query = [&a_b_c_pattern[..], &[Child::new(c, 1)]].concat();
        assert_eq!(
            graph.find_ancestor(&query),
            Ok(QueryFound {
                found: FoundPath::Complete(*abc),
                query: QueryRangePath {
                    exit: query.len() - 2,
                    query,
                    entry: 0,
                    start: vec![],
                    end: vec![],
                    finished: false,
                },
            }),
            "a_b_c_c"
        );
    }
    #[test]
    fn find_ancestor2() {
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
        let (xaby, xaby_ids) = graph.index_patterns_with_ids([vec![xab, y], vec![xa, by]]);
        let xa_by_id = xaby_ids[1];
        let (xabyz, xabyz_ids) = graph.index_patterns_with_ids([vec![xaby, z], vec![xab, yz]]);
        let xaby_z_id = xabyz_ids[0];
        let graph = HypergraphRef::from(graph);
        let query = vec![by, z];
        let byz_found = graph.find_ancestor(&query);
        assert_eq!(
            byz_found,
            Ok(QueryFound {
                found: FoundPath::Range(GraphRangePath {
                    start: StartPath::Path {
                        entry: xabyz.to_pattern_location(xaby_z_id)
                            .to_child_location(0),
                        path: vec![
                            ChildLocation {
                                parent: xaby,
                                pattern_id: xa_by_id,
                                sub_index: 1,
                            },
                        ],
                        width: 2,
                    },
                    end: vec![],
                    exit: 1,
                    inner_width: 0,
                    end_width: 1,
                }),
                query: QueryRangePath {
                    exit: query.len() - 1,
                    query,
                    entry: 0,
                    start: vec![],
                    end: vec![],
                    finished: true,
                },
            }),
            "by_z"
        );
    }
    #[test]
    fn find_sequence() {
        let Context {
            graph,
            abc,
            ababababcdefghi,
            ..
         } = &*context();
        assert_eq!(
            graph.find_sequence("a".chars()),
            Err(NoMatch::SingleIndex),
        );
        let query = graph.read().unwrap().expect_token_pattern("abc".chars());
        let abc_found = graph.find_ancestor(&query);
        assert_eq!(
            abc_found,
            Ok(QueryFound::complete(query, abc)),
            "abc"
        );
        let query = graph.read().unwrap().expect_token_pattern("ababababcdefghi".chars());
        let ababababcdefghi_found = graph.find_ancestor(&query);
        assert_eq!(
            ababababcdefghi_found,
            Ok(QueryFound::complete(query, ababababcdefghi)),
            "ababababcdefghi"
        );
    }
}
