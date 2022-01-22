use crate::{
    r#match::*,
    search::*,
    Hypergraph,
};
use itertools::*;

#[derive(Clone, Debug)]
enum BfsNode {
    Start(Child),
    Parent(ChildPath), // start path, parent, pattern index
    Child(ChildPath, Child, ChildPath, Child), // start path, root, end path, child
}

pub struct Searcher<'g, T: Tokenize, D: MatchDirection> {
    graph: &'g Hypergraph<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<'g, T: Tokenize, D: MatchDirection> std::ops::Deref for Searcher<'g, T, D> {
    type Target = Hypergraph<T>;
    fn deref(&self) -> &Self::Target {
        self.graph
    }
}
impl<'g, T: Tokenize + 'g, D: MatchDirection> Searcher<'g, T, D> {
    pub fn new(graph: &'g Hypergraph<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    pub(crate) fn matcher(&self) -> Matcher<'g, T, D> {
        Matcher::new(self.graph)
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
                self.bfs_match(
                    start,
                    context,
                    |_start_path, _root, _loc| {
                        vec![]
                    }
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
                if tail.is_empty() {
                    Ok(FoundPath::complete(head))
                } else {
                    self.bfs_match(
                        head,
                        tail,
                        |start_path, root, loc| {
                            // root at end of parent, recurse upwards to get all next children
                            let mut start_path = start_path.clone();
                            start_path.push(loc);
                            self.bfs_root_parents(root, start_path)
                        }
                    )
                }
            )
    }
    fn bfs_root_parents(
        &self,
        root: Child,
        start_path: ChildPath,
    ) -> Vec<BfsNode> {
        let vertex = root.vertex(self.graph);
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
                        BfsNode::Parent(start_path)
                    })
                    .collect_vec()
            })
            .flatten()
            .collect_vec()
    }
    fn bfs_parent_children_end_op(
        &self,
        mut start_path: ChildPath,
        context_next: Child,
        end_op: impl Fn(ChildPath, Child, ChildLocation) -> Vec<BfsNode>,
    ) -> Vec<BfsNode> {
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
                vec![BfsNode::Child(
                    start_path,
                    root,
                    vec![ChildLocation::new(root, pattern_id, D::index_next(sub_index).unwrap())],
                    next_child,
                )]
            } else {
                vec![]
            }
        } else {
            end_op(start_path, root, loc)
        }
    }
    fn bfs_match<'a>(
        &'g self,
        start: impl Vertexed<'a, 'g>,
        query: impl IntoPattern<Item = impl AsChild>,
        end_op: impl Fn(ChildPath, Child, ChildLocation) -> Vec<BfsNode> + Copy,
    ) -> SearchResult {
        let start_index = start.as_child();
        // try any parent, i.e. first
        let query = query.as_pattern_view();
        D::pattern_head(query)
            .and_then(|query_next| {
                let query_next = query_next.to_child();
                // if context not empty
                // breadth first traversal
                Bft::new(BfsNode::Start(start_index), |node| {
                    match node.clone() {
                        BfsNode::Start(root) => {
                            self.bfs_root_parents(root, vec![])
                                .into_iter()
                        },
                        BfsNode::Parent(start_path) => {
                            self.bfs_parent_children_end_op(
                                start_path,
                                query_next,
                                end_op,
                            )
                            .into_iter()
                        },
                        BfsNode::Child(start_path, root, path, child) => {
                            let vertex = self.expect_vertex_data(child);
                            let child_patterns = vertex.get_children();
                            child_patterns
                                .into_iter()
                                .map(|(&pid, pattern)| {
                                    let &head = D::pattern_head(pattern).unwrap();
                                    let mut path = path.clone();
                                    path.push(ChildLocation::new(child, pid, D::head_index(pattern)));
                                    BfsNode::Child(start_path.clone(), root, path, head)
                                })
                                .collect_vec()
                                .into_iter()
                        },
                    }
                })
                .find_map(|(_, node)| {
                    match node {
                        BfsNode::Parent(
                            mut start_path,
                        ) => {
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
                                        let next_index = D::index_next(sub_index).unwrap();
                                        let query_tail = D::pattern_tail(query).into_pattern();
                                        if sub_index != D::head_index(pattern) {
                                            start_path.push(loc);
                                        }
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
                        },
                        BfsNode::Child(
                            start_path,
                            root,
                            mut path,
                            child,
                        ) => {
                            // find child starting with next_child
                            let vertex = self.expect_vertex_data(child);
                            let child_patterns = vertex.get_children();
                            child_patterns
                                .into_iter()
                                .find(|(_pid, pattern)| {
                                    let &head = D::pattern_head(pattern).unwrap();
                                    head == query_next
                                })
                                .map(|(&pid, pattern)| {
                                    path.push(ChildLocation::new(child, pid, D::head_index(pattern)));
                                    let query_tail = D::pattern_tail(query).into_pattern();
                                    FoundPath::new(root, start_path, path, query_tail)
                                })
                        },
                        _ => None,
                    }
                })
                .map(|found_path|
                    if let Some(end_path) = found_path.end_path.clone() {
                        match self.matcher()
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
                                        => self.bfs_match(found_path.root, remainder, end_op),
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
                )
            })
            .unwrap_or_else(|| Ok(FoundPath::remainder(start_index, query)))
    }
    ///// comparison on child pattern and context
    ///// returns ParentMatch and end index of match
    ///// matches only when context matches at least one index
    //pub(crate) fn compare_child_context_at(
    //    &'g self,
    //    child_patterns: &'g ChildPatterns,
    //    location: PatternLocation,
    //    sub_index: usize,
    //    context: impl IntoPattern<Item = impl AsChild>,
    //) -> Option<SearchFound> {
    //    let child_pattern = child_patterns
    //        .get(&location.pattern_id)
    //        .expect("non existent pattern found as best match!");
    //    let (back_context, child_tail) = Self::split_child_context(child_pattern, sub_index);
    //    // match search context with child tail
    //    // back context is from child pattern
    //    self.child_context_at_offset_comparison(context, back_context, child_tail, child_pattern.len())
    //        .map(|(parent_match, end_index)|
    //            SearchFound {
    //                parent_match,
    //                location: location.with_range(sub_index..end_index),
    //                path: vec![location.parent],
    //            }
    //        )
    //        .ok()
    //    // returns result of matching sub with parent's children
    //}
    //pub(crate) fn split_child_context(
    //    child_pattern: &Pattern,
    //    sub_index: usize,
    //) -> (Pattern, Pattern) {
    //    let (back_context, rem) = D::directed_pattern_split(child_pattern, sub_index);
    //    let child_tail = D::pattern_tail(&rem[..]);
    //    (back_context, child_tail.to_vec())
    //}
    //pub(crate) fn child_context_at_offset_comparison(
    //    &self,
    //    context: impl IntoPattern<Item = impl AsChild>,
    //    back_context: impl IntoPattern<Item = impl AsChild>,
    //    child_tail: impl IntoPattern<Item = impl AsChild>,
    //    child_pattern_len: usize,
    //) -> Result<(ParentMatch, usize), NoMatch> {
    //    // match search context with child tail
    //    // back context is from child pattern
    //    self.compare(context.as_pattern_view(), child_tail).map(|pm| {
    //        let post_len = pm.1.as_ref().map(|p| p.len()).unwrap_or_default();
    //        let end = child_pattern_len-post_len;
    //        (ParentMatch {
    //            parent_range: D::to_found_range(pm.1, back_context.into_pattern()),
    //            remainder: pm.0,
    //        }, end)
    //    })
    //}
    // find largest matching ancestor with width < width_ceiling
    //pub(crate) fn find_largest_matching_ancestor<'a>(
    //    &'g self,
    //    start: impl Vertexed<'a, 'g>,
    //    context: impl IntoPattern<Item = impl AsChild>,
    //    width_ceiling: Option<TokenPosition>,
    //) -> SearchResult {
    //    let vertex_index = start.index();
    //    let vertex = start.vertex(self);

    //    // search direct matching parent
    //    // search parents (and their children) for matching next index
    //    //let mut first = None;
    //    self.vertex_find_matching_or_any_parent(
    //        //&mut first,
    //        vertex,
    //        context,
    //        width_ceiling
    //    )
    //    //.or_else(||
    //    //    // use first
    //    //    // Todo: breadth first search
    //    //    first.map(|(index, parent)| {
    //    //        let vert = self.expect_vertex_data(index);
    //    //        let child_patterns = vert.get_children();
    //    //        let (pid, sub_index) = parent.any_pattern_index();
    //    //        let loc = PatternLocation::new(Child::new(index, parent.width), pid);
    //    //        (child_patterns, loc, sub_index)
    //    //    })
    //    //)
    //    .and_then(|(
    //        child_patterns,
    //        location,
    //        sub_index,
    //    )|
    //        self.compare_child_context_at(
    //                child_patterns,
    //                location,
    //                sub_index,
    //                context,
    //            )
    //    )
    //    .ok_or(NoMatch::NoMatchingParent(vertex_index))
    //    .and_then(|search_found| {
    //        // if found parent
    //        if let Some(rem) = &search_found.parent_match.remainder {
    //            if D::found_till_end(&search_found.parent_match.parent_range) {
    //                // continue searching in parents of found
    //                self.find_largest_matching_ancestor(search_found.location.parent, rem, None)
    //                    .map(|super_found| SearchFound {
    //                        parent_match: search_found.clone()
    //                            .parent_match
    //                            .embed_in_super(super_found.parent_match),
    //                        location: super_found.location,
    //                        path: [search_found.path.clone(), super_found.path].concat()
    //                    })
    //                    .or(Ok(search_found))
    //            } else {
    //                Ok(search_found)
    //            }
    //        } else {
    //            Ok(search_found)
    //        }
    //    })
    //}
    //#[allow(unused)]
    ///// find by pattern by iterator of possibly new tokens
    //pub(crate) fn find_ancestor_try_iter(
    //    &self,
    //    pattern: impl IntoIterator<Item = Result<impl ToChild + Tokenize, NoMatch>>,
    //) -> SearchResult {
    //    let pattern: Pattern = pattern
    //        .into_iter()
    //        .map(|r| r.map(ToChild::to_child))
    //        .collect::<Result<Pattern, NoMatch>>()?;
    //    self.find_ancestor(pattern)
    //}
}
