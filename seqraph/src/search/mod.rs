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
mod bft;
pub use bft::*;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryRangePath {
    pub(crate) query: Pattern,
    pub(crate) entry: usize,
    pub(crate) start: ChildPath,
    pub(crate) exit: usize,
    pub(crate) end: ChildPath,
}
impl QueryRangePath {
    pub fn complete(query: impl IntoPattern<Item = impl AsChild>, index: impl AsChild) -> Self {
        let query = query.into_pattern();
        Self {
            entry: 0,
            exit: query.len() - 1,
            query,
            start: vec![],
            end: vec![],
        }
    }
    pub fn new_directed<D: MatchDirection, C: AsChild, P: IntoPattern<Item = C>>(query: P) -> Result<Self, NoMatch> {
        let entry = D::head_index(query.as_pattern_view());
        Ok(Self {
            query: query.into_pattern(),
            entry,
            start: vec![],
            exit: entry,
            end: vec![],
        })
    }
    pub fn get_entry(&self) -> Child {
        // todo: use path
        self.query.get(self.entry).cloned().expect("Invalid entry")
    }
    pub fn get_exit(&self) -> Child {
        // todo: use path
        self.query.get(self.exit).cloned().expect("Invalid exit")
    }
    fn get_end<T: Tokenize>(&self, trav: impl Traversable<T>) -> Child {
        if let Some(next) = self.end.last() {
            trav.graph().expect_child_at(next)
        } else {
            self.query.get(self.exit).cloned().expect("Invalid exit")
        }
    }
    fn advance_next<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(&mut self, trav: Trav) -> bool {
        let graph = trav.graph();
        // skip path segments with no successors
        while let Some(mut location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(location);
            if let Some(next) = D::index_next(&pattern, location.sub_index) {
                location.sub_index = next;
                self.end.push(location);
                return true;
            }
        }
        // end is empty (exit is prev)
        if let Some(next) = D::index_next(&self.query, self.exit) {
            self.exit = next;
            true
        } else {
            false
        }
    }
    fn push_next(&mut self, next: ChildLocation) {
        self.end.push(next);
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphRangePath {
    pub(crate) start: StartPath,
    //pub(crate) root_pattern: PatternLocation,
    //pub(crate) entry: usize,
    //pub(crate) start: ChildPath,
    pub(crate) exit: usize,
    pub(crate) end: ChildPath,
    //pub(crate) width: usize,
}
impl GraphRangePath {
    pub fn get_exit_location(&self) -> ChildLocation {
        self.start.entry()
            .into_pattern_location()
            .to_child_location(self.exit)
    }
    pub fn get_entry_location(&self) -> ChildLocation {
        self.start.entry()
    }
    pub fn into_start_path(self) -> StartPath {
        self.start
    }
    pub fn new(start: StartPath) -> Self {
        Self {
            exit: start.entry().sub_index,
            //entry: entry.sub_index,
            //root_pattern: entry.into_pattern_location(),
            start,
            end: vec![],
            //width,
        }
    }
    pub fn is_complete(&self) -> bool {
        // todo: respect exit (need graph access)
        self.start.is_complete() && self.end.is_empty()
    }
    pub fn get_end_location(&self) -> ChildLocation {
        if self.end.is_empty() {
            self.get_exit_location()
        } else {
            self.end.last().unwrap().clone()
        }
    }
    fn get_end<T: Tokenize>(&self, trav: impl Traversable<T>) -> Child {
        trav.graph().expect_child_at(self.get_end_location())
    }
    fn advance_next<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(&mut self, trav: Trav) -> bool {
        let graph = trav.graph();
        // skip path segments with no successors
        while let Some(mut location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(location);
            if let Some(next) = D::index_next(&pattern, location.sub_index) {
                *self.start.width_mut() += pattern.get(next).unwrap().width;
                location.sub_index = next;
                self.end.push(location);
                return true;
            }
        }
        // end is empty (exit is prev)
        let location = self.get_exit_location();
        let pattern = graph.expect_pattern_at(location);
        if let Some(next) = D::index_next(&pattern, location.sub_index) {
            *self.start.width_mut() += pattern.get(next).unwrap().width;
            self.exit = next;
            true
        } else {
            false
        }
    }
    fn push_next(&mut self, next: ChildLocation) {
        self.end.push(next);
    }
    //pub(crate) fn reduce_mismatch<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(mut self, trav: Trav) -> FoundPath {
    //    let graph = trav.graph();
    //    if self.reduce_end_path::<T, D>(&*graph) {
    //        return FoundPath::new(self)
    //    }
    //    // end is empty (exit is mismatch)
    //    let pattern = graph.expect_pattern_at(self.root_pattern);
    //    if let Some(prev) = D::index_prev(&pattern, self.exit) {
    //        if self.entry == prev {
    //            if let Some(last) = self.start.pop() {
    //                self.entry = last.sub_index;
    //                self.root_pattern = last.into_pattern_location();
    //                let pattern = graph.expect_pattern_at(self.root_pattern);
    //                self.exit = pattern.len() - 1;
    //                FoundPath::new(self)
    //            } else {
    //                FoundPath::Complete(*pattern.get(self.entry).unwrap())
    //            }
    //        } else {
    //            self.exit = prev;
    //            FoundPath::new(self)
    //        }
    //    } else {
    //        FoundPath::new(self)
    //    }
    //}
    //pub(crate) fn reduce_end_path<T: Tokenize, D: MatchDirection>(&mut self, graph: &Hypergraph<T>) {
    //}
    pub(crate) fn reduce_end<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(mut self, trav: Trav) -> FoundPath {
        let graph = trav.graph();
        //self.reduce_end_path::<T, D>(&*graph);
        // remove segments pointing to mismatch at pattern head
        while let Some(location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(location);
            // skip segments at end of pattern
            if D::index_next(pattern.as_pattern_view(), location.sub_index).is_some() {
                self.end.push(location);
                break;
            }
        }
        FoundPath::new(self)
    }
}
#[derive(Clone)]
pub enum PathPair {
    GraphMajor(GraphRangePath, QueryRangePath),
    QueryMajor(QueryRangePath, GraphRangePath),
}
impl PathPair {
    pub fn from_mode(path: GraphRangePath, query: QueryRangePath, mode: bool) -> Self {
        if mode {
            Self::GraphMajor(path, query)
        } else {
            Self::QueryMajor(query, path)
        }
    }
    pub fn mode(&self) -> bool {
        matches!(self, Self::GraphMajor(_, _))
    }
    pub fn push_major(&mut self, location: ChildLocation) {
        match self {
            Self::GraphMajor(path, query) =>
                path.push_next(location),
            Self::QueryMajor(query, path) =>
                query.push_next(location),
        }
    }
    pub fn unpack(self) -> (GraphRangePath, QueryRangePath) {
        match self {
            Self::GraphMajor(path, query) =>
                (path, query),
            Self::QueryMajor(query, path) =>
                (path, query),
        }
    }
}
//pub trait RangePath: Clone {
//    fn get_end<T: Tokenize>(&self, trav: impl Traversable<T>) -> Child;
//    fn advance_next<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(&mut self, trav: Trav) -> bool;
//    fn push_next(&mut self, next: ChildLocation);
//}
//impl<P: RangePath> RangePath for &P {
//    fn get_end<T: Tokenize>(&self, trav: impl Traversable<T>) -> Child {
//        self.get_end(trav)
//    }
//    fn advance_next<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(&mut self, trav: Trav) -> bool {
//        self.advance_next::<_, _, D>(trav)
//    }
//    fn push_next(&mut self, next: ChildLocation) {
//        self.push_next(next)
//    }
//}
//impl RangePath for QueryRangePath {
//}
//impl RangePath for GraphRangePath {
//}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoundPath {
    Complete(Child),
    Range(GraphRangePath),
}
impl FoundPath {
    pub fn new(path: GraphRangePath) -> Self {
        if path.is_complete() {
            FoundPath::Complete(path.start.entry().parent)
        } else {
            FoundPath::Range(path)
        }
    }
    fn width(&self) -> usize {
        match self {
            Self::Complete(c) => c.width,
            Self::Range(r) => r.start.width(),
        }
    }
    pub fn unwrap_complete(self) -> Child {
        match self {
            Self::Complete(index) => index,
            _ => panic!("Unable to unwrap {:?} as complete.", self),
        }
    }
}
impl Into<FoundPath> for GraphRangePath {
    fn into(self) -> FoundPath {
        FoundPath::new(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryFound {
    pub(crate) found: FoundPath,
    pub(crate) query: QueryRangePath,
}

impl QueryFound {
    pub fn new(found: impl Into<FoundPath>, query: QueryRangePath) -> Self {
        QueryFound {
            found: found.into(),
            query,
        }
    }
    pub fn complete(query: impl IntoPattern<Item = impl AsChild>, index: impl AsChild) -> Self {
        Self {
            found: FoundPath::Complete(index.as_child()),
            query: QueryRangePath::complete(query, index),
        }
    }
    pub fn unwrap_complete(self) -> Child {
        self.found.unwrap_complete()
    }
}
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
    };
    use pretty_assertions::{
        assert_eq,
    };
    use itertools::*;

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
            //ababababcdefghi,
            ..
         } = &*context();
        let a_bc_pattern = vec![Child::new(a, 1), Child::new(bc, 2)];
        let ab_c_pattern = vec![Child::new(ab, 2), Child::new(c, 1)];
        let a_bc_d_pattern = vec![Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
        let b_c_pattern = vec![Child::new(b, 1), Child::new(c, 1)];
        let bc_pattern = vec![Child::new(bc, 2)];
        let a_b_c_pattern = vec![Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];

        let query = bc_pattern;
        //assert_eq!(
        //    graph.find_ancestor(&query),
        //    Err(NoMatch::SingleIndex),
        //    "bc"
        //);
        //let query = b_c_pattern;
        //assert_eq!(
        //    graph.find_ancestor(&query),
        //    Ok(QueryFound::complete(query, bc)),
        //    "b_c"
        //);
        //let query = a_bc_pattern;
        //assert_eq!(
        //    graph.find_ancestor(&query),
        //    Ok(QueryFound::complete(query, abc)),
        //    "a_bc"
        //);
        //let query = ab_c_pattern;
        //assert_eq!(
        //    graph.find_ancestor(&query),
        //    Ok(QueryFound::complete(query, abc)),
        //    "ab_c"
        //);
        //let query = a_bc_d_pattern;
        //assert_eq!(
        //    graph.find_ancestor(&query),
        //    Ok(QueryFound::complete(query, abcd)),
        //    "a_bc_d"
        //);
        //let query = a_b_c_pattern.clone();
        //assert_eq!(
        //    graph.find_ancestor(&query),
        //    Ok(QueryFound::complete(query, abc)),
        //    "a_b_c"
        //);
        //let query =
        //    vec![*a, *b, *a, *b, *a, *b, *a, *b, *c, *d, *e, *f, *g, *h, *i];
        //assert_eq!(
        //    graph.find_ancestor(&query),
        //    Ok(QueryFound::complete(query, ababababcdefghi)),
        //    "a_b_a_b_a_b_a_b_c_d_e_f_g_h_i"
        //);
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
                },
            }),
            "a_b_c_c"
        );
    }
    #[test]
    fn find_ancestor2() {
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
        let (xaby, xaby_ids) = graph.insert_patterns_with_ids([vec![xab, y], vec![xa, by]]);
        let xa_by_id = xaby_ids[1];
        let (xabyz, xabyz_ids) = graph.insert_patterns_with_ids([vec![xaby, z], vec![xab, yz]]);
        let xaby_z_id = xabyz_ids[0];
        let graph = HypergraphRef::from(graph);
        let query = vec![by, z];
        let byz_found = graph.find_ancestor(&query);
        assert_eq!(
            byz_found,
            Ok(QueryFound {
                found: FoundPath::Range(GraphRangePath {
                    start: StartPath::Path(
                        xabyz.to_pattern_location(xaby_z_id)
                            .to_child_location(0),
                        vec![
                            ChildLocation {
                                parent: xaby,
                                pattern_id: xa_by_id,
                                sub_index: 1,
                            },
                        ],
                        3,
                    ),
                    end: vec![],
                    exit: 1,
                }),
                query: QueryRangePath {
                    exit: query.len() - 1,
                    query,
                    entry: 0,
                    start: vec![],
                    end: vec![],
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
