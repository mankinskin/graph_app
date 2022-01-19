use crate::{
    r#match::*,
    search::*,
    Hypergraph,
};
use itertools::*;

#[derive(Clone, Debug)]
enum BfsNode {
    Start(Child),
    Parent(ChildPath, Child, PatternId, usize), // start path, parent, pattern index
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
                    |_start_path, _root, _pattern_id, _sub_index| {
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
                        |mut start_path, root, pattern_id, sub_index| {
                            // root at end of parent, recurse upwards to get all next children
                            start_path.push(ChildLocation::new(root, pattern_id, sub_index));
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
                    .map(|&(pid, sub_index)| {
                        BfsNode::Parent(start_path.clone(), p.clone(), pid, sub_index)
                    })
                    .collect_vec()
            })
            .flatten()
            .collect_vec()
    }
    fn bfs_parent_children_end_op(
        &self,
        start_path: ChildPath,
        root: Child,
        pattern_id: PatternId,
        sub_index: usize,
        context_next: Child,
        end_op: impl Fn(ChildPath, Child, PatternId, usize) -> Vec<BfsNode>,
    ) -> Vec<BfsNode> {
        // find parent partition with matching context
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
            end_op(start_path, root, pattern_id, sub_index)
        }
    }
    fn bfs_match<'a>(
        &'g self,
        start: impl Vertexed<'a, 'g>,
        context: impl IntoPattern<Item = impl AsChild>,
        end_op: impl Fn(ChildPath, Child, PatternId, usize) -> Vec<BfsNode> + Copy,
    ) -> SearchResult {
        let start_index = start.as_child();
        // try any parent, i.e. first
        let context = context.as_pattern_view();
        D::pattern_head(context)
            .and_then(|context_next| {
                let context_next: Child = context_next.to_child();
                // if context not empty
                // breadth first traversal
                Bft::new(BfsNode::Start(start_index), |node| {
                    match node.clone() {
                        BfsNode::Start(root) => {
                            self.bfs_root_parents(root, vec![])
                                .into_iter()
                        },
                        BfsNode::Parent(start_path, parent, pattern_id, sub_index) => {
                            self.bfs_parent_children_end_op(
                                start_path,
                                parent,
                                pattern_id,
                                sub_index,
                                context_next,
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
                            start_path,
                            parent,
                            pid,
                            sub_index,
                        ) => {
                            // find next child equal to next context index
                            let parent_vertex = self.expect_vertex_data(parent.index());
                            let child_patterns = parent_vertex.get_children();
                            let pattern = child_patterns.get(&pid).unwrap();
                            D::next_child(pattern, sub_index)
                                .and_then(|next_child|
                                    (next_child == context_next).then(|| {
                                        let next_index = D::index_next(sub_index).unwrap();
                                        let remainder = D::pattern_tail(context).into_pattern();
                                        let remainder = if remainder.is_empty() {
                                            None
                                        } else {
                                            Some(remainder)
                                        };
                                        let end_path = if D::index_next(next_index)
                                            .map(|next| next == pattern.len())
                                            .unwrap_or(true) {
                                            // matches completely
                                            vec![]
                                        } else {
                                            vec![
                                                ChildLocation::new(
                                                    parent_vertex.as_child(),
                                                    pid,
                                                    next_index,
                                                )
                                            ]
                                        };
                                        FoundPath {
                                            root: parent,
                                            start_path,
                                            end_path,
                                            remainder,
                                        }
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
                                    head == context_next
                                })
                                .map(|(&pid, pattern)| {
                                    path.push(ChildLocation::new(child, pid, D::head_index(pattern)));
                                    let remainder = D::pattern_tail(context).into_pattern();
                                    FoundPath {
                                        root,
                                        start_path,
                                        end_path: path,
                                        remainder: if remainder.is_empty() {
                                            None
                                        } else {
                                            Some(remainder)
                                        },
                                    }
                                })
                        },
                        _ => None,
                    }
                })
                .map(|found_path|
                    if let Some(remainder) = found_path.remainder.clone() {
                        self.matcher().grow_path_into_context(found_path.end_path.clone(), remainder)
                            .map_err(NoMatch::Mismatch)
                            .and_then(|match_path|
                                match match_path.remainder {
                                    GrowRemainder::Context(remainder)
                                        => self.bfs_match(found_path.root, remainder, end_op),
                                    _ => Ok(FoundPath {
                                            start_path: found_path.start_path,
                                            root: found_path.root,
                                            end_path: vec![],
                                            remainder: None,
                                        }),
                                }
                            )
                    } else {
                        Ok(found_path)
                    }
                )
            })
            .unwrap_or_else(||
                Ok(FoundPath {
                    root: start_index,
                    start_path: vec![],
                    end_path: vec![],
                    remainder: if context.is_empty() {
                        None
                    } else {
                        Some(context.into_pattern())
                    },
                })
            )
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
