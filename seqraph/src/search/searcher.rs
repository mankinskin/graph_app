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
    /// find largest matching ancestor with width < width_ceiling
    pub(crate) fn find_largest_matching_ancestor(
        &self,
        start: impl Vertexed,
        context: impl IntoPattern<Item = impl AsChild>,
        width_ceiling: Option<TokenPosition>,
    ) -> SearchResult {
        let vertex_index = *start.index();
        let vertex = start.vertex(self);
        let parents = vertex.get_parents_below_width(width_ceiling);

        // search parents for matching parents
        if let Some((&index, child_patterns, parent, pattern_id, sub_index)) =
            parents.clone().find_map(|(index, parent)| {
                let vert = self.expect_vertex_data(*index);
                let child_patterns = vert.get_children();
                //print!("matching parent \"{}\" ", self.index_string(parent.index));
                // get child pattern indices able to match at all
                let candidates = D::filter_parent_pattern_indices(parent, child_patterns);
                candidates
                    .into_iter()
                    .find(|(pattern_index, sub_index)|
                        // find pattern with same next index
                        D::compare_next_index_in_child_pattern(
                            child_patterns,
                            context.as_pattern_view(),
                            pattern_index,
                            *sub_index,
                        )
                    )
                    .map(|(pattern_index, sub_index)|
                        (index, child_patterns, parent, pattern_index, sub_index)
                    )
            })
        {
            // direct matching parent
            self.matcher()
                .compare_child_pattern_at_offset(
                    child_patterns,
                    context,
                    pattern_id,
                    sub_index,
                )
                .map(|parent_match| SearchFound {
                    index: Child::new(index, parent.width),
                    pattern_id,
                    sub_index,
                    parent_match,
                })
        } else {
            // no direct matching parent
            // search possible parents for matching
            let mut parents = parents;
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
                        .map(|(parent_match, pattern_id, sub_index)| SearchFound {
                            index: Child::new(index, parent.width),
                            pattern_id,
                            sub_index,
                            parent_match,
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
                            .map(|(parent_match, pattern_id, sub_index)| SearchFound {
                                index: Child::new(index, parent.width),
                                pattern_id,
                                sub_index,
                                parent_match,
                            })
                            .ok()
                    )
                )
                .ok_or(NoMatch::NoMatchingParent(vertex_index))
        }
        .and_then(|search_found| {
            if let Some(rem) = &search_found.parent_match.remainder {
                self.find_largest_matching_ancestor(search_found.index, rem, None)
                    .map(|super_found| SearchFound {
                        parent_match: search_found
                            .parent_match
                            .embed_in_super(super_found.parent_match),
                        index: super_found.index,
                        sub_index: super_found.sub_index,
                        pattern_id: super_found.pattern_id,
                    })
            } else {
                Ok(search_found)
            }
        })
    }
    //#[allow(unused)]
    ///// find by pattern by iterator of possibly new tokens
    //pub(crate) fn find_pattern_try_iter(
    //    &self,
    //    pattern: impl IntoIterator<Item = Result<impl ToChild + Tokenize, NoMatch>>,
    //) -> SearchResult {
    //    let pattern: Pattern = pattern
    //        .into_iter()
    //        .map(|r| r.map(ToChild::to_child))
    //        .collect::<Result<Pattern, NoMatch>>()?;
    //    self.find_pattern(pattern)
    //}
}
