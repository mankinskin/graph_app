use crate::{
    r#match::*,
    search::*,
    Hypergraph,
};
use itertools::*;

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
    /// find largest matching direct parent
    pub(crate) fn find_ancestor<'a>(
        &'g self,
        pattern: impl IntoPattern<Item = impl AsChild, Token=impl AsChild + Clone + Vertexed<'a, 'g>>,
    ) -> SearchResult {
        Right::split_head_tail(pattern.as_pattern_view())
            .ok_or(NoMatch::EmptyPatterns)
            .and_then(|(head, tail)|
                if tail.is_empty() {
                    // Todo: positive type to return when single index is passed
                    Err(NoMatch::SingleIndex)
                } else {
                    self.find_largest_matching_ancestor(head, tail.to_vec(), None)
                }
            )
    }
    /// find largest matching direct parent
    pub(crate) fn find_parent<'a>(
        &'g self,
        pattern: impl IntoPattern<Item = impl AsChild, Token=impl AsChild + Clone + Vertexed<'a, 'g>>,
    ) -> SearchResult {
        Right::split_head_tail(pattern.as_pattern_view())
            .ok_or(NoMatch::EmptyPatterns)
            .and_then(|(head, tail)|
                self.vertex_find_common_parent(
                    head,
                    tail,
                    None,
                )
            )
    }
    /// find largest matching ancestor with width < width_ceiling
    pub(crate) fn find_largest_matching_ancestor<'a>(
        &'g self,
        start: impl Vertexed<'a, 'g>,
        context: impl IntoPattern<Item = impl AsChild>,
        width_ceiling: Option<TokenPosition>,
    ) -> SearchResult {
        let vertex_index = start.index();
        let vertex = start.vertex(self);

        // search parents for matching parents
        self.vertex_find_common_parent(vertex, context.as_pattern_view(), width_ceiling)
            .or_else(|_| {
                // no direct matching parent
                // search possible parents for matching
                let mut parents = vertex.get_parents_below_width(width_ceiling).collect_vec();
                // try parents in ascending width
                parents.sort_unstable_by_key(|a| a.1.width);
                let context = context.as_pattern_view();
                let mut first = None;
                parents.into_iter()
                    .find_map(|(index, parent)| {
                        // find pattern with matching context
                        first.get_or_insert((index, parent));
                        self .find_matching_context_in_parent(
                                context.as_pattern_view(),
                                index,
                                parent,
                            )
                            .map(|(parent_match, location)| SearchFound {
                                parent_match,
                                location,
                                path: vec![Child::new(index, parent.width)],
                            })
                            .ok()
                    })
                    .or_else(||
                        // try any parent, i.e. first
                        // Todo: breadth first search?
                        first.and_then(|(index, parent)|
                            parent
                                .pattern_indices
                                .iter()
                                .next()
                                .cloned()
                                .ok_or(NoMatch::NoParents)
                                .and_then(|(pattern_index, sub_index)| {
                                    let vert = self.expect_vertex_data(index);
                                    let child_patterns = vert.get_children();
                                    self.matcher()
                                        .compare_child_context_at_offset(
                                            child_patterns,
                                            context,
                                            pattern_index,
                                            sub_index,
                                        )
                                        .map(|parent_match| (parent_match, pattern_index, sub_index))
                                })
                                .map(|((parent_match, end_index), pattern_id, sub_index)|
                                    SearchFound {
                                        location: PatternRangeLocation {
                                            parent: Child::new(index, parent.width),
                                            pattern_id,
                                            range: sub_index..end_index,
                                        },
                                        parent_match,
                                        path: vec![Child::new(index, parent.width)],
                                    }
                                )
                                .ok()
                        )
                    )
                    .ok_or(NoMatch::NoMatchingParent(vertex_index))
            })
        .and_then(|search_found| {
            // if found parent
            if let Some(rem) = &search_found.parent_match.remainder {
                if D::found_at_end(&search_found.parent_match.parent_range) {
                    // continue searching
                    self.find_largest_matching_ancestor(search_found.location.parent, rem, None)
                        .map(|super_found| SearchFound {
                            parent_match: search_found.clone()
                                .parent_match
                                .embed_in_super(super_found.parent_match),
                            location: super_found.location,
                            path: [search_found.path.clone(), super_found.path].concat()
                        })
                        .or(Ok(search_found))
                } else {
                    Ok(search_found)
                }
            } else {
                Ok(search_found)
            }
        })
    }
    /// find parent common to vertex and a context
    fn vertex_find_common_parent<'a>(
        &'g self,
        start: impl Vertexed<'a, 'g>,
        context: impl IntoPattern<Item = impl AsChild>,
        width_ceiling: Option<TokenPosition>,
    ) -> SearchResult {
        let vertex = start.vertex(self);
        vertex.get_parents_below_width(width_ceiling)
            .find_map(|(index, parent)| {
                let vert = self.expect_vertex_data(*index);
                let child_patterns = vert.get_children();
                // find parent indices with same context
                parent.pattern_indices
                    .iter()
                    .find(|(pattern_index, sub_index)|
                        // find pattern with same next index
                        D::compare_next_index_in_child_pattern(
                            child_patterns,
                            context.as_pattern_view(),
                            pattern_index,
                            *sub_index,
                        )
                    )
                    .map(|&(pattern_id, sub_index)| (
                        child_patterns,
                        PatternLocation {
                            parent: Child::new(index, parent.width),
                            pattern_id,
                        },
                        // continue matching from next index
                        sub_index,
                    ))
            })
            .and_then(|(
                child_patterns, // from parent
                location, // location in parent
                sub_index, // sub index in found parent pattern
            )|
                self.matcher()
                    .compare_child_context_at_offset(
                        child_patterns,
                        context,
                        location.pattern_id,
                        sub_index,
                    )
                    .map(|(parent_match, end)| SearchFound {
                        path: vec![location.parent.clone()],
                        location: location.with_range(sub_index..end),
                        parent_match,
                    })
                    .ok()
            )
            .ok_or(NoMatch::NoParents)
    }
    /// match context against child contexts in parent.
    pub fn find_matching_context_in_parent(
        &'g self,
        context: impl IntoPattern<Item = impl AsChild>,
        parent_index: impl Indexed,
        parent: &Parent,
    ) -> Result<(ParentMatch, PatternRangeLocation), NoMatch> {
        //println!("compare_parent_context");
        let vert = self.expect_vertex_data(parent_index.index());
        let child_patterns = vert.get_children();
        //print!("matching parent \"{}\" ", self.index_string(parent.index));
        // optionally filter by sub index
        //println!("with successors \"{}\"", self.pattern_string(post_pattern));
        // try to find child pattern with same next index
        D::filter_parent_pattern_indices(parent, child_patterns)
            .into_iter()
            .find_or_first(|(pattern_index, sub_index)|
                D::compare_next_index_in_child_pattern(
                    child_patterns,
                    context.as_pattern_view(),
                    pattern_index,
                    *sub_index,
                )
            )
            .ok_or(NoMatch::NoChildPatterns)
            .and_then(|(pattern_index, sub_index)| {
                self.matcher().compare_child_context_at_offset(
                    child_patterns,
                    context,
                    pattern_index,
                    sub_index,
                )
                .map(|(parent_match, end_index)|
                    (parent_match, PatternRangeLocation {
                        parent: Child::new(parent_index, parent.width),
                        pattern_id: pattern_index,
                        range: sub_index..end_index
                    })
                )
                // if context doesn't match, at least current child matches
            })
    }
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
