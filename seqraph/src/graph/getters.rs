use crate::*;
use crate::vertex::indexed::Indexed;

impl<'t, G: GraphKind> Hypergraph<G> {
    pub fn get_vertex(
        &self,
        index: impl Indexed,
    ) -> Result<(&VertexKey<G::Token>, &VertexData), NoMatch> {
        self.graph
            .get_index(index.vertex_index())
            .ok_or(NoMatch::UnknownIndex)
    }
    pub fn get_vertex_mut(
        &mut self,
        index: impl Indexed,
    ) -> Result<(&mut VertexKey<G::Token>, &mut VertexData), NoMatch> {
        self.graph
            .get_index_mut(index.vertex_index())
            .ok_or(NoMatch::UnknownIndex)
    }
    #[track_caller]
    pub fn expect_vertex(
        &self,
        index: impl Indexed,
    ) -> (&VertexKey<G::Token>, &VertexData) {
        let index = index.vertex_index();
        self.get_vertex(index)
            .unwrap_or_else(|_| panic!("Index {} does not exist!", index))
    }
    pub fn get_pattern_at(
        &self,
        location: impl IntoPatternLocation,
    ) -> Result<&Pattern, NoMatch> {
        let location = location.into_pattern_location();
        let vertex = self.get_vertex_data(location.parent)?;
        let child_patterns = vertex.get_child_patterns();
        child_patterns.get(&location.id)
            .ok_or(NoMatch::NoChildPatterns) // todo: better error
    }
    #[track_caller]
    pub fn expect_pattern_at(
        &self,
        location: impl IntoPatternLocation,
    ) -> &Pattern {
        let location = location.into_pattern_location();
        self.get_pattern_at(location)
            .unwrap_or_else(|_|
                panic!("Pattern not found at location {:#?}", location)
            )
    }
    pub fn get_child_at(
        &self,
        location: impl IntoChildLocation,
    ) -> Result<Child, NoMatch> {
        let location = location.into_child_location();
        let pattern = self.get_pattern_at(&location)?;
        pattern.get(location.sub_index)
            .cloned()
            .ok_or(NoMatch::NoChildPatterns) // todo: better error
    }
    #[track_caller]
    pub fn expect_child_at(
        &self,
        location: impl IntoChildLocation,
    ) -> Child {
        let location = location.into_child_location();
        self.get_child_at(location)
            .unwrap_or_else(|_| panic!("Child not found at location {:#?}", location))
    }
    pub fn get_child_patterns_of(
        &self,
        index: impl Indexed,
    ) -> Result<&ChildPatterns, NoMatch> {
        self.get_vertex_data(index)
            .map(|vertex| vertex.get_child_patterns())
    }
    pub fn get_pattern_of(
        &self,
        index: impl Indexed,
        pid: PatternId
    ) -> Result<&Pattern, NoMatch> {
        self.get_vertex_data(index)
            .and_then(|vertex| vertex.get_child_pattern(&pid))
    }
    #[track_caller]
    pub fn expect_child_pattern(
        &self,
        index: impl Indexed,
        pid: PatternId
    ) -> &Pattern {
        self.expect_vertex_data(index)
            .expect_child_pattern(&pid)
    }
    #[track_caller]
    pub fn expect_child_patterns(
        &self,
        index: impl Indexed,
    ) -> &ChildPatterns {
        self.expect_vertex_data(index).get_child_patterns()
    }

    #[track_caller]
    pub fn expect_any_child_pattern(&self, index: impl Indexed) -> (&PatternId, &Pattern) {
        self.expect_vertex_data(index).expect_any_child_pattern()
    }
    pub fn expect_child_offset(
        &self,
        loc: &ChildLocation,
    ) -> usize {
        self.expect_vertex_data(&loc.vertex_index()).expect_child_offset(&loc.to_sub_location())
    }
    #[track_caller]
    pub fn expect_vertex_mut(
        &mut self,
        index: impl Indexed,
    ) -> (&mut VertexKey<G::Token>, &mut VertexData) {
        let index = index.vertex_index();
        self.get_vertex_mut(index)
            .unwrap_or_else(|_| panic!("Index {} does not exist!", index))
    }
    pub fn get_vertex_key(
        &self,
        index: impl Indexed,
    ) -> Result<&VertexKey<G::Token>, NoMatch> {
        self.get_vertex(index).map(|entry| entry.0)
    }
    #[track_caller]
    pub fn expect_vertex_key(
        &self,
        index: impl Indexed,
    ) -> &VertexKey<G::Token> {
        self.expect_vertex(index).0
    }
    pub fn get_vertex_data(
        &self,
        index: impl Indexed,
    ) -> Result<&VertexData, NoMatch> {
        self.get_vertex(index).map(|(_, v)| v)
    }
    pub fn get_vertex_data_mut(
        &mut self,
        index: impl Indexed,
    ) -> Result<&mut VertexData, NoMatch> {
        self.get_vertex_mut(index).map(|(_, v)| v)
    }
    #[track_caller]
    pub fn expect_vertex_data(
        &self,
        index: impl Indexed,
    ) -> &VertexData {
        self.expect_vertex(index).1
    }
    #[track_caller]
    pub fn expect_vertex_data_mut(
        &mut self,
        index: impl Indexed,
    ) -> &mut VertexData {
        self.expect_vertex_mut(index).1
    }
    pub fn get_vertex_data_by_key(
        &self,
        key: &VertexKey<G::Token>,
    ) -> Result<&VertexData, NoMatch> {
        self.graph.get(key).ok_or(NoMatch::UnknownKey)
    }
    pub fn get_vertex_data_by_key_mut(
        &mut self,
        key: &VertexKey<G::Token>,
    ) -> Result<&mut VertexData, NoMatch> {
        self.graph.get_mut(key).ok_or(NoMatch::UnknownKey)
    }
    #[track_caller]
    pub fn expect_vertex_data_by_key(
        &self,
        key: &VertexKey<G::Token>,
    ) -> &VertexData {
        self.graph.get(key).expect("Key does not exist")
    }
    #[track_caller]
    pub fn expect_vertex_data_by_key_mut(
        &mut self,
        key: &VertexKey<G::Token>,
    ) -> &mut VertexData {
        self.graph.get_mut(key).expect("Key does not exist")
    }
    pub fn vertex_iter(&self) -> impl Iterator<Item = (&VertexKey<G::Token>, &VertexData)> {
        self.graph.iter()
    }
    pub fn vertex_iter_mut(&mut self) -> impl Iterator<Item = (&VertexKey<G::Token>, &mut VertexData)> {
        self.graph.iter_mut()
    }
    pub fn vertex_key_iter(&self) -> impl Iterator<Item = &VertexKey<G::Token>> {
        self.graph.keys()
    }
    pub fn vertex_data_iter(&self) -> impl Iterator<Item = &VertexData> {
        self.graph.values()
    }
    pub fn vertex_data_iter_mut(&mut self) -> impl Iterator<Item = &mut VertexData> {
        self.graph.values_mut()
    }
    #[track_caller]
    pub fn expect_vertices(
        &self,
        indices: impl Iterator<Item = impl Indexed>,
    ) -> VertexPatternView<'_> {
        indices
            .map(move |index| self.expect_vertex_data(index))
            .collect()
    }
    pub fn get_vertices(
        &self,
        indices: impl Iterator<Item = impl Indexed>,
    ) -> Result<VertexPatternView<'_>, NoMatch> {
        indices
            .map(move |index| self.get_vertex_data(index))
            .collect()
    }
    pub fn get_token_data(
        &self,
        token: &Token<G::Token>,
    ) -> Result<&VertexData, NoMatch> {
        self.get_vertex_data_by_key(&VertexKey::Token(*token))
    }
    pub fn get_token_data_mut(
        &mut self,
        token: &Token<G::Token>,
    ) -> Result<&mut VertexData, NoMatch> {
        self.get_vertex_data_by_key_mut(&VertexKey::Token(*token))
    }
    pub fn get_index_by_key(
        &self,
        key: &VertexKey<G::Token>,
    ) -> Result<VertexIndex, NoMatch> {
        self.graph.get_index_of(key).ok_or(NoMatch::UnknownKey)
    }
    #[track_caller]
    pub fn expect_index_by_key(
        &self,
        key: &VertexKey<G::Token>,
    ) -> VertexIndex {
        self.graph.get_index_of(key).expect("Key does not exist")
    }
    pub fn get_token_index(
        &self,
        token: impl AsToken<G::Token>,
    ) -> Result<VertexIndex, NoMatch> {
        self.get_index_by_key(&VertexKey::Token(token.as_token()))
    }
    pub fn get_token_child(
        &self,
        token: impl AsToken<G::Token>,
    ) -> Result<Child, NoMatch> {
        self.get_token_index(token).map(|i| Child::new(i, 1))
    }
    #[track_caller]
    pub fn expect_token_index(
        &self,
        token: impl AsToken<G::Token>,
    ) -> VertexIndex {
        self.expect_index_by_key(&VertexKey::Token(token.as_token()))
    }
    #[track_caller]
    pub fn expect_pattern_range_width(
        &self,
        location: impl IntoPatternLocation,
        range: impl PatternRangeIndex
    ) -> usize {
        pattern_width(self.expect_child_pattern_range(location, range))
    }
    #[track_caller]
    pub fn expect_token_child(
        &self,
        token: impl AsToken<G::Token>,
    ) -> Child {
        Child::new(self.expect_token_index(token), 1)
    }
    pub fn to_token_indices_iter<'a>(
        &'a self,
        tokens: impl IntoIterator<Item = impl AsToken<G::Token>> + 'a,
    ) -> impl Iterator<Item = Result<VertexIndex, NoMatch>> + 'a {
        tokens
            .into_iter()
            .map(move |token| self.get_token_index(token))
    }
    pub fn to_token_indices(
        &self,
        tokens: impl IntoIterator<Item = impl AsToken<G::Token>>,
    ) -> Result<IndexPattern, NoMatch> {
        tokens
            .into_iter()
            .map(|token| self.get_token_index(token))
            .collect()
    }
    pub fn to_token_children_iter<'a>(
        &'a self,
        tokens: impl IntoIterator<Item = impl AsToken<G::Token>> + 'a,
    ) -> impl Iterator<Item = Result<Child, NoMatch>> + 'a {
        self.to_token_indices_iter(tokens)
            .map(move |index| index.map(|index| Child::new(index, 1)))
    }
    pub fn to_token_children(
        &self,
        tokens: impl IntoIterator<Item = impl AsToken<G::Token>>,
    ) -> Result<impl IntoPattern, NoMatch> {
        self.to_token_children_iter(tokens)
            .collect::<Result<Pattern, _>>()
    }
    #[track_caller]
    pub fn expect_token_pattern(
        &self,
        tokens: impl IntoIterator<Item = impl AsToken<G::Token>>,
    ) -> Pattern {
        self.to_token_children(tokens)
            .expect("Failed to convert tokens to children")
            .into_pattern()
    }
    pub fn get_token_indices(
        &self,
        tokens: impl Iterator<Item = impl AsToken<G::Token>>,
    ) -> Result<IndexPattern, NoMatch> {
        let mut v = IndexPattern::with_capacity(tokens.size_hint().0);
        for token in tokens {
            let index = self.get_token_index(token)?;
            v.push(index);
        }
        Ok(v)
    }
    #[track_caller]
    pub fn expect_parent(
        &self,
        index: impl Indexed,
        parent: impl Indexed,
    ) -> &Parent {
        self.expect_vertex_data(index).expect_parent(parent)
    }
    #[track_caller]
    pub fn expect_parent_mut(
        &mut self,
        index: impl Indexed,
        parent: impl Indexed,
    ) -> &mut Parent {
        self.expect_vertex_data_mut(index).expect_parent_mut(parent)
    }
    #[track_caller]
    pub fn expect_parents(
        &self,
        index: impl Indexed,
    ) -> &VertexParents {
        self.expect_vertex_data(index).get_parents()
    }
    #[track_caller]
    pub fn expect_parents_mut(
        &mut self,
        index: impl Indexed,
    ) -> &mut VertexParents {
        self.expect_vertex_data_mut(index).get_parents_mut()
    }
    pub fn expect_is_at_end(
        &self,
        location: &ChildLocation
    ) -> bool {
        self.expect_vertex_data(location.vertex_index())
            .expect_pattern_len(&location.pattern_id) == location.sub_index + 1
    }
    pub fn get_child(
        &self,
        index: impl Indexed,
    ) -> Child {
        self.to_child(index)
    }
    pub fn to_child(
        &self,
        index: impl Indexed,
    ) -> Child {
        Child::new(index.vertex_index(), self.index_width(&index))
    }
    pub fn to_children(
        &self,
        indices: impl IntoIterator<Item = impl Indexed>,
    ) -> Pattern {
        indices.into_iter().map(|i| self.to_child(i)).collect()
    }
    pub fn get_pattern_parents(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
        parent: impl Indexed,
    ) -> Result<Vec<Parent>, NoMatch> {
        pattern
            .into_iter()
            .map(|index| {
                let vertex = self.expect_vertex_data(index);
                vertex.get_parent(parent.vertex_index()).map(Clone::clone)
            })
            .collect()
    }
    pub fn get_common_pattern_in_parent(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
        parent: impl Indexed,
    ) -> Result<PatternIndex, NoMatch> {
        let mut parents = self
            .get_pattern_parents(pattern, parent)?
            .into_iter()
            .enumerate();
        parents
            .next()
            .and_then(|(_, first)| {
                first
                    .pattern_indices
                    .iter()
                    .find(|pix| {
                        parents.all(|(i, post)| post.exists_at_pos_in_pattern(pix.pattern_id, pix.sub_index + i))
                    })
                    .cloned()
            })
            .ok_or(NoMatch::NoChildPatterns)
    }
    #[track_caller]
    pub fn expect_common_pattern_in_parent(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
        parent: impl Indexed,
    ) -> PatternIndex {
        self.get_common_pattern_in_parent(pattern, parent)
            .expect("No common pattern in parent for children.")
    }
    pub fn get_child_pattern_range<'a, R: PatternRangeIndex>(
        &'a self,
        id: impl IntoPatternLocation,
        range: R,
    ) -> Result<&'a <R as SliceIndex<[Child]>>::Output, NoMatch> {
        let loc = id.into_pattern_location();
        self
            .get_vertex_data(loc.parent)?
            .get_child_pattern_range(
                &loc.id,
                range,
            )
    }
    #[track_caller]
    pub fn expect_child_pattern_range<'a, R: PatternRangeIndex>(
        &'a self,
        id: impl IntoPatternLocation,
        range: R,
    ) -> &'a <R as SliceIndex<[Child]>>::Output {
        let loc = id.into_pattern_location();
        self
            .expect_vertex_data(loc.parent)
            .expect_child_pattern_range(
                &loc.id,
                range,
            )
    }
}
impl<'t, 'a, G: GraphKind> Hypergraph<G> {
    //pub fn async_to_token_indices_stream(
    //    arc: Arc<RwLock<Self>>,
    //    tokens: impl TokenStream<T> + 't,
    //) -> impl PatternStream<VertexIndex, Token<T>> + 't {
    //    let handle = tokio::runtime::Handle::current();
    //    tokens.map(move |token|
    //        // is this slow?
    //        handle.block_on(async {
    //            arc.read().get_token_index(token.as_token())
    //                .map_err(|_| Token::Element(token))
    //        }))
    //}
    //pub fn async_to_token_children_stream(
    //    arc: Arc<RwLock<Self>>,
    //    tokens: impl TokenStream<T> + 't,
    //) -> impl PatternStream<Child, Token<T>> + 't {
    //    Self::async_to_token_indices_stream(arc, tokens)
    //        
    //        .map(move |index| index.into_inner().map(|index| Child::new(index, 1)))
    //}
    //pub fn to_token_indices_stream(
    //    &'a self,
    //    tokens: impl TokenStream<G::Token> + 'a,
    //) -> impl PatternStream<VertexIndex, Token<G::Token>> + 'a {
    //    tokens.map(move |token| {
    //        self.get_token_index(token.as_token())
    //            .map_err(|_| Token::Element(token))
    //    })
    //}
    //pub fn to_token_children_stream(
    //    &'a self,
    //    tokens: impl TokenStream<G::Token> + 'a,
    //) -> impl PatternStream<Child, Token<G::Token>> + 'a {
    //    self.to_token_indices_stream(tokens)
    //        .map(move |index| index.into_inner().map(|index| Child::new(index, 1)))
    //}
}
