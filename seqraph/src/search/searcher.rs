use crate::{
    r#match::*,
    search::*,
    Hypergraph,
};
use itertools::*;
use std::ops::Deref;

pub struct Searcher<'g, T: Tokenize, D: MatchDirection> {
    graph: &'g Hypergraph<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<'g, T: Tokenize, D: MatchDirection> Deref for Searcher<'g, T, D> {
    type Target = Hypergraph<T>;
    fn deref(&self) -> &Self::Target {
        self.graph
    }
}

struct AncestorSearch<'g, T: Tokenize + 'g, D: MatchDirection> {
    _ty: std::marker::PhantomData<(&'g T, D)>,
}
impl<'g, T: Tokenize + 'g, D: MatchDirection + 'g> BreadthFirstTraversal<'g> for AncestorSearch<'g, T, D> {
    type Trav = Searcher<'g, T, D>;
    fn end_op((searcher, start_path, root, loc): <Self::Trav as Traversable>::State) -> Vec<<Self::Trav as Traversable>::Node> {
        let mut start_path = start_path.clone();
        start_path.push(loc);
        searcher.bft_parents(root, start_path)
    }
}
struct ParentSearch<'g, T: Tokenize + 'g, D: MatchDirection> {
    _ty: std::marker::PhantomData<(&'g T, D)>,
}
impl<'g, T: Tokenize + 'g, D: MatchDirection + 'g> BreadthFirstTraversal<'g> for ParentSearch<'g, T, D> {
    type Trav = Searcher<'g, T, D>;
    fn end_op(_state: <Self::Trav as Traversable>::State) -> Vec<<Self::Trav as Traversable>::Node> {
        vec![]
    }
}
impl<'g, T: Tokenize + 'g, D: MatchDirection + 'g> Traversable for Searcher<'g, T, D> {
    type Node = SearchNode;
    type State = (
        &'g Self,
        ChildPath,
        Child,
        ChildLocation,
    );
}
impl<'g, T: Tokenize + 'g, D: MatchDirection> Searcher<'g, T, D> {
    pub fn new(graph: &'g Hypergraph<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    // find largest matching direct parent
    pub(crate) fn find_pattern_parent<'a>(
        &'g self,
        pattern: impl IntoPattern<Item = impl AsChild, Token=impl AsChild + Clone + Vertexed<'a, 'g>>,
    ) -> SearchResult {
        Right::split_head_tail(pattern.as_pattern_view())
            .ok_or(NoMatch::EmptyPatterns)
            .and_then(|(head, context)| {
                let start = head.vertex(self);
                self.bft_search::<ParentSearch<'g, T, D>, _, _, _>(
                    start,
                    context,
                )
            })
    }
    /// find largest matching ancestor for pattern
    pub(crate) fn find_pattern_ancestor<'a>(
        &'g self,
        pattern: impl IntoPattern<Item = impl AsChild, Token=impl AsChild + Clone + Vertexed<'a, 'g>>,
    ) -> SearchResult {
        Right::split_head_tail(pattern.as_pattern_view())
            .ok_or(NoMatch::EmptyPatterns)
            .and_then(|(head, tail)|
                self.find_ancestor_in_context(head, tail)
            )
    }
    pub(crate) fn find_ancestor_in_context(
        &'g self,
        head: impl AsChild,
        tail: impl IntoPattern<Item = impl AsChild>,
    ) -> SearchResult {
        let head = head.as_child();
        if tail.is_empty() {
            Err(NoMatch::SingleIndex)
        } else {
            self.bft_search::<AncestorSearch<'g, T, D>, _, _, _>(
                head,
                tail,
            )
        }
    }
    fn bft_parents(
        &self,
        root: Child,
        start_path: ChildPath,
    ) -> Vec<SearchNode> {
        let vertex = root.vertex(self);
        let mut parents = vertex.get_parents().into_iter().collect_vec();
        // try parents in ascending width
        parents.sort_unstable_by_key(|a| a.1.width);
        parents.into_iter()
            .map(|(i, parent)| {
                let p = Child::new(i, parent.width);
                parent.pattern_indices
                    .iter()
                    .map(|&i| {
                        let mut start_path = start_path.clone();
                        start_path.push(ChildLocation::new(p, i.pattern_id, i.sub_index));
                        SearchNode::Parent(start_path)
                    })
                    .collect_vec()
            })
            .flatten()
            .collect_vec()
    }
    fn bft_children<
        S: BreadthFirstTraversal<'g, Trav=Self>,
    >(
        &'g self,
        mut start_path: ChildPath,
        context_next: Child,
    ) -> Vec<SearchNode> {
        // find parent partition with matching context
        let loc@ChildLocation {
            parent: root,
            pattern_id,
            sub_index,
        } = start_path.pop().unwrap();
        let parent_vertex = self.expect_vertex_data(root.index());
        let child_patterns = parent_vertex.get_children();
        let pattern = child_patterns.get(&pattern_id).unwrap();
        if let Some(next_child) = D::next_child(pattern, sub_index) {
            // equal indices would have been found in find
            if next_child.width > context_next.width {
                Some(SearchNode::Child(
                    start_path,
                    root,
                    vec![ChildLocation::new(root, pattern_id, D::index_next(sub_index).unwrap())],
                    next_child,
                ))
            } else {
                None
            }
            .into_iter()
            .collect_vec()
        } else {
            S::end_op((self, start_path, root, loc))
        }
    }
    pub(crate) fn match_parent(
        &self,
        mut start_path: ChildPath,
        query_next: Child,
        query: impl IntoPattern<Item=impl AsChild>
    ) -> Option<FoundPath> {
        // find next child equal to next context index
        let loc@ChildLocation {
            parent,
            pattern_id,
            sub_index,
        } = start_path.pop().unwrap();
        let parent_vertex = self.expect_vertex_data(parent.index());
        let child_patterns = parent_vertex.get_children();
        let pattern = child_patterns.get(&pattern_id).unwrap();
        D::next_child(pattern, sub_index)
            .and_then(|child_next|
                (child_next == query_next).then(|| {
                    // todo: explain this
                    if sub_index != D::head_index(pattern) {
                        start_path.push(loc);
                    }
                    let next_index = D::index_next(sub_index).unwrap();
                    let query_tail = D::pattern_tail(query.as_pattern_view()).into_pattern();
                    let end_path = vec![
                        ChildLocation::new(
                            parent_vertex.as_child(),
                            pattern_id,
                            next_index,
                        )
                    ];
                    FoundPath::new(parent, start_path, end_path, query_tail)
                })
            )
    }
    pub(crate) fn match_child(
        &self,
        start_path: ChildPath,
        root: Child,
        mut path: ChildPath,
        current: Child,
        query_next: Child,
        query: impl IntoPattern<Item=impl AsChild>
    ) -> Option<FoundPath> {
        // find child starting with next_child
        let vertex = self.expect_vertex_data(current);
        let child_patterns = vertex.get_children();
        child_patterns
            .into_iter()
            .find(|(_pid, pattern)| {
                let &head = D::pattern_head(pattern).unwrap();
                head == query_next
            })
            .map(|(&pid, pattern)| {
                path.push(ChildLocation::new(current, pid, D::head_index(pattern)));
                let query_tail = D::pattern_tail(query.as_pattern_view()).into_pattern();
                FoundPath::new(root, start_path, path, query_tail)
            })
    }
    pub(crate) fn expand_found<
        S: BreadthFirstTraversal<'g, Trav=Self>,
    >(
        &'g self,
        found_path: FoundPath,
    ) -> SearchResult {
        if let Some(end_path) = found_path.end_path.clone() {
            match self.matcher::<D>()
                .match_path_in_context(
                    end_path,
                    found_path.remainder.clone().unwrap_or_default(),
                ) {
                Err(mismatch_path) =>
                    Ok(FoundPath {
                        root: found_path.root,
                        start_path: found_path.start_path,
                        end_path: if mismatch_path.path.is_empty() {
                            None
                        } else {
                            Some(mismatch_path.path)
                        },
                        remainder: Some(mismatch_path.query),
                    }),
                Ok(match_path) =>
                    match match_path.remainder {
                        GrowthRemainder::Query(remainder)
                            => self.bft_search::<S, _, _, _>(found_path.root, remainder.clone())
                                    .or_else(|_| Ok(FoundPath::remainder(found_path.root, remainder))),
                        GrowthRemainder::Child(_) => Ok(FoundPath {
                            root: found_path.root,
                            start_path: found_path.start_path,
                            end_path: if match_path.path.is_empty() {
                                None
                            } else {
                                Some(match_path.path)
                            },
                            remainder: None,
                        }),
                        GrowthRemainder::None => Ok(FoundPath {
                            root: found_path.root,
                            start_path: found_path.start_path,
                            end_path: if match_path.path.len() < 2 {
                                None
                            } else {
                                Some(match_path.path)
                            },
                            remainder: None,
                        })
                    },
                }
        } else {
            Ok(found_path)
        }
    }
    fn bft_search<
        'a,
        S: BreadthFirstTraversal<'g, Trav=Self>,
        V: Vertexed<'a, 'g>,
        C: AsChild,
        Q: IntoPattern<Item = C>,
    >(
        &'g self,
        start: V,
        query: Q,
    ) -> SearchResult {
        let start_index = start.as_child();
        // try any parent, i.e. first
        let query = query.as_pattern_view();
        D::pattern_head(query)
            .ok_or_else(|| NoMatch::SingleIndex)
            .and_then(|query_next| {
                let query_next = query_next.to_child();
                // if context not empty
                // breadth first traversal
                Bft::new(SearchNode::Start(start_index), move |node| {
                    match node.clone() {
                        SearchNode::Start(root) => {
                            self.bft_parents(root, vec![])
                                .into_iter()
                        },
                        SearchNode::Parent(start_path) => {
                            self.bft_children::<S>(
                                start_path,
                                query_next,
                            )
                            .into_iter()
                        },
                        SearchNode::Child(start_path, root, path, child) => {
                            let vertex = self.expect_vertex_data(child);
                            let child_patterns = vertex.get_children();
                            child_patterns
                                .into_iter()
                                .map(|(&pid, pattern)| {
                                    let &head = D::pattern_head(pattern).unwrap();
                                    let mut path = path.clone();
                                    path.push(ChildLocation::new(child, pid, D::head_index(pattern)));
                                    SearchNode::Child(start_path.clone(), root, path, head)
                                })
                                .collect_vec()
                                .into_iter()
                        },
                    }
                })
                .find_map(|(_, node)|
                    match node {
                        SearchNode::Parent(start_path) =>
                            self.match_parent(start_path, query_next, query),
                        SearchNode::Child(start_path, root, path, child) =>
                            self.match_child(start_path, root, path, child, query_next, query),
                        _ => None,
                    }
                )
                .map(|found_path| self.expand_found::<S>(found_path))
                .unwrap_or_else(||
                    Err(NoMatch::NotFound(query.into_pattern()))
                )
            })
    }
}
