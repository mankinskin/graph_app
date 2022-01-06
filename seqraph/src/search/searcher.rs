use crate::{
    r#match::*,
    search::*,
    Hypergraph,
};

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
    fn vertex_find_common_parent<'a>(
        &'g self,
        start: impl Vertexed<'a, 'g>,
        context: impl IntoPattern<Item = impl AsChild>,
        width_ceiling: Option<TokenPosition>,
    ) -> SearchResult {
        let vertex = start.vertex(self);
        let parents = vertex.get_parents_below_width(width_ceiling);
        parents.clone().find_map(|(index, parent)| {
            let vert = self.expect_vertex_data(*index);
            let child_patterns = vert.get_children();
            //print!("matching parent \"{}\" ", self.index_string(parent.index));
            // get child pattern indices able to match at all
            //let candidates = D::filter_parent_pattern_indices(parent, child_patterns);
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
                .map(|&(pattern_id, sub_index)|
                    (child_patterns, PatternLocation {
                        parent: Child::new(index, parent.width),
                        range: sub_index..child_patterns.get(&pattern_id).unwrap().len(),
                        pattern_id,
                    })
                )
        })
        .and_then(|(child_patterns, location)|
            self.matcher()
                .compare_child_pattern_at_offset(
                    child_patterns,
                    context,
                    location.pattern_id,
                    location.range.start,
                )
                .map(|(parent_match, _)| SearchFound {
                    location,
                    parent_match,
                })
                .ok()
        )
        .ok_or(NoMatch::NoParents)
    }
    /// find largest matching ancestor with width < width_ceiling
    pub(crate) fn find_largest_matching_ancestor<'a>(
        &'g self,
        start: impl Vertexed<'a, 'g>,
        context: impl IntoPattern<Item = impl AsChild>,
        width_ceiling: Option<TokenPosition>,
    ) -> SearchResult {
        let vertex_index = *start.index();
        let vertex = start.vertex(self);

        // search parents for matching parents
        self.vertex_find_common_parent(vertex, context.as_pattern_view(), width_ceiling)
            .or_else(|_| {
                // no direct matching parent
                // search possible parents for matching
                let mut parents = vertex.get_parents_below_width(width_ceiling);
                let context = context.as_pattern_view();
                // Todo: breadth first search?
                let mut first = None;
                parents
                    .find_map(|(index, parent)| {
                        first.get_or_insert((index, parent));
                        self.matcher()
                            .match_context_with_parent_children(
                                context.as_pattern_view(),
                                index,
                                parent,
                            )
                            .map(|(parent_match, location)| SearchFound {
                                parent_match,
                                location,
                            })
                            .ok()
                    })
                    .or_else(||
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
                                        .compare_child_pattern_at_offset(
                                            child_patterns,
                                            context,
                                            pattern_index,
                                            sub_index,
                                        )
                                        .map(|parent_match| (parent_match, pattern_index, sub_index))
                                })
                                .map(|((parent_match, end_index), pattern_id, sub_index)|
                                    SearchFound {
                                        location: PatternLocation {
                                            parent: Child::new(index, parent.width),
                                            pattern_id,
                                            range: sub_index..end_index,
                                        },
                                        parent_match,
                                    }
                                )
                                .ok()
                        )
                    )
                    .ok_or(NoMatch::NoMatchingParent(vertex_index))
            })
        .and_then(|search_found| {
            if let Some(rem) = &search_found.parent_match.remainder {
                if D::found_at_end(&search_found.parent_match.parent_range) {
                    self.find_largest_matching_ancestor(search_found.location.parent, rem, None)
                        .map(|super_found| SearchFound {
                            parent_match: search_found
                                .parent_match
                                .embed_in_super(super_found.parent_match),
                            location: super_found.location,
                        })
                } else {
                    Ok(search_found)
                }
            } else {
                Ok(search_found)
            }
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
