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

#[derive(Clone, Debug)]
pub(crate) enum SearchNode {
    Query(QueryRangePath),
    Root(QueryRangePath, StartPath),
    Match(RangePath, QueryRangePath),
    End(RangePath, QueryRangePath),
}
impl BftNode for SearchNode {
    fn query_node(query: QueryRangePath) -> Self {
        Self::Query(query)
    }
    fn root_node(query: QueryRangePath, start_path: StartPath) -> Self {
        Self::Root(query, start_path)
    }
    fn match_node(path: RangePath, query: QueryRangePath) -> Self {
        Self::Match(path, query)
    }
    fn end_node(path: RangePath, query: QueryRangePath) -> Self {
        Self::End(path, query)
    }
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
    pub fn new_directed<D: MatchDirection, C: AsChild, P: IntoPattern<Item = C>>(query: P) -> Result<Self, NoMatch> {
        let entry = D::head_index(query.as_pattern_view());
        let exit = D::index_next(query.as_pattern_view(), entry).ok_or_else(|| NoMatch::SingleIndex)?;
        Ok(Self {
            query: query.into_pattern(),
            entry,
            start: vec![],
            exit,
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
    pub(crate) fn get_next<T: Tokenize>(&self, trav: impl Traversable<T>) -> Child {
        if let Some(next) = self.end.last() {
            trav.graph().expect_child_at(next)
        } else {
            self.query.get(self.exit).cloned().expect("Invalid exit")
        }
    }
    pub(crate) fn advance_end<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(&mut self, trav: Trav) -> bool {
        let graph = trav.graph();
        while let Some(mut location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(location);
            if let Some(next) = D::index_next(pattern, location.sub_index) {
                location.sub_index = next;
                self.end.push(location);
                return true;
            }
        }
        if let Some(next) = D::index_next(&self.query, self.exit) {
            self.exit = next;
            true
        } else {
            false
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangePath {
    pub(crate) root_pattern: PatternLocation,
    pub(crate) entry: usize,
    pub(crate) start: ChildPath,
    pub(crate) exit: usize,
    pub(crate) end: ChildPath,
}
impl RangePath {
    pub fn get_exit_location(&self) -> ChildLocation {
        if self.end.is_empty() {
            self.root_pattern.to_child_location(self.exit)
        } else {
            self.end.last().unwrap().clone()
        }
    }
    pub fn into_start_path(self) -> StartPath{
        StartPath {
            path: self.start,
            entry: self.root_pattern.to_child_location(self.entry),
        }
    }
    pub fn is_complete(&self) -> bool {
        // todo: respect entry and exit
        self.start.is_empty() && self.end.is_empty()
    }
    pub(crate) fn get_next<T: Tokenize>(&self, trav: impl Traversable<T>) -> Child {
        if let Some(next) = self.end.last() {
            trav.graph().expect_child_at(next)
        } else {
            trav.graph().expect_child_at(self.get_exit_location())
        }
    }
    pub(crate) fn push_next(&mut self, next: ChildLocation) {
        self.end.push(next);
    }
    pub(crate) fn advance_end<T: Tokenize, Trav: Traversable<T>, D: MatchDirection>(&mut self, trav: Trav) -> bool {
        let graph = trav.graph();
        while let Some(mut location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(location);
            if let Some(next) = D::index_next(pattern, location.sub_index) {
                location.sub_index = next;
                self.end.push(location);
                return true;
            }
        }
        let mut location = self.get_exit_location();
        let pattern = graph.expect_pattern_at(location);
        if let Some(next) = D::index_next(pattern, location.sub_index) {
            location.sub_index = next;
            self.end.push(location);
            true
        } else {
            false
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundPath {
    pub(crate) path: RangePath,
    pub(crate) query: QueryRangePath,
}
impl FoundPath {
    pub fn unwrap_complete(self) -> Child {
        self.is_complete().then(||
            self.path.root_pattern.parent
        ).unwrap()
    }
    pub fn is_complete(&self) -> bool {
        self.path.is_complete()
    }
}
pub type SearchResult = Result<FoundPath, NoMatch>;

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

    //#[test]
    //fn find_ancestor1() {
    //    let Context {
    //        graph,
    //        a,
    //        b,
    //        c,
    //        d,
    //        e,
    //        f,
    //        g,
    //        h,
    //        i,
    //        ab,
    //        bc,
    //        abc,
    //        abcd,
    //        ababababcdefghi,
    //        ..
    //     } = &*context();
    //    let a_bc_pattern = vec![Child::new(a, 1), Child::new(bc, 2)];
    //    let ab_c_pattern = vec![Child::new(ab, 2), Child::new(c, 1)];
    //    let a_bc_d_pattern = vec![Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
    //    let b_c_pattern = vec![Child::new(b, 1), Child::new(c, 1)];
    //    let bc_pattern = vec![Child::new(bc, 2)];
    //    let a_b_c_pattern = vec![Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];
    //    assert_eq!(
    //        graph.find_ancestor(&bc_pattern),
    //        Err(NoMatch::SingleIndex),
    //        "bc"
    //    );
    //    assert_eq!(
    //        graph.find_ancestor(&b_c_pattern),
    //        Ok(FoundPath::complete(bc)),
    //        "b_c"
    //    );
    //    assert_eq!(
    //        graph.find_ancestor(&a_bc_pattern),
    //        Ok(FoundPath::complete(abc)),
    //        "a_bc"
    //    );
    //    assert_eq!(
    //        graph.find_ancestor(&ab_c_pattern),
    //        Ok(FoundPath::complete(abc)),
    //        "ab_c"
    //    );
    //    assert_eq!(
    //        graph.find_ancestor(&a_bc_d_pattern),
    //        Ok(FoundPath::complete(abcd)),
    //        "a_bc_d"
    //    );
    //    assert_eq!(
    //        graph.find_ancestor(&a_b_c_pattern),
    //        Ok(FoundPath::complete(abc)),
    //        "a_b_c"
    //    );
    //    let a_b_a_b_a_b_a_b_c_d_e_f_g_h_i_pattern =
    //        vec![*a, *b, *a, *b, *a, *b, *a, *b, *c, *d, *e, *f, *g, *h, *i];
    //    assert_eq!(
    //        graph.find_ancestor(&a_b_a_b_a_b_a_b_c_d_e_f_g_h_i_pattern),
    //        Ok(FoundPath::complete(ababababcdefghi)),
    //        "a_b_a_b_a_b_a_b_c_d_e_f_g_h_i"
    //    );
    //    let a_b_c_c_pattern = [&a_b_c_pattern[..], &[Child::new(c, 1)]].concat();
    //    assert_eq!(
    //        graph.find_ancestor(&a_b_c_c_pattern),
    //        Ok(FoundPath::remainder(abc, [c].as_slice())),
    //        "a_b_c_c"
    //    );
    //}
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
            Ok(FoundPath {
                path: RangePath {
                    root_pattern: xabyz.to_pattern_location(xaby_z_id),
                    entry: 0,
                    start: vec![
                        ChildLocation {
                            parent: xaby,
                            pattern_id: xa_by_id,
                            sub_index: 1,
                        },
                    ],
                    end: vec![],
                    exit: 1,
                },
                query: QueryRangePath {
                    exit: query.len() - 1,
                    query,
                    entry: 0,
                    start: vec![],
                    end: vec![],
                },
            })
        );
    }
    //#[test]
    //fn find_sequence() {
    //    let Context {
    //        graph,
    //        abc,
    //        ababababcdefghi,
    //        ..
    //     } = &*context();
    //    assert_eq!(
    //        graph.find_sequence("a".chars()),
    //        //Ok(FoundPath::complete(a))
    //        Err(NoMatch::SingleIndex),
    //    );

    //    let abc_found = graph.find_sequence("abc".chars());
    //    assert_eq!(
    //        abc_found,
    //        Ok(FoundPath::complete(abc))
    //    );
    //    let ababababcdefghi_found = graph.find_sequence("ababababcdefghi".chars());
    //    assert_eq!(
    //        ababababcdefghi_found,
    //        Ok(FoundPath::complete(ababababcdefghi))
    //    );
    //}
}
