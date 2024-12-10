use crate::graph::{
        getters::{vertex::VertexSet, NoMatch},
        kind::GraphKind,
        vertex::{
            child::Child,
            data::VertexData,
            key::VertexKey,
            pattern::{
                IntoPattern,
                Pattern,
            },
            token::{
                AsToken,
                Token,
            },
            IndexPattern,
            VertexIndex,
        },
        Hypergraph,
    };

impl<G: GraphKind> Hypergraph<G> {
    pub fn get_token_data(
        &self,
        token: &Token<G::Token>,
    ) -> Result<&VertexData, NoMatch> {
        self.get_vertex(self.get_token_index(token)?)
    }
    pub fn get_token_data_mut(
        &mut self,
        token: &Token<G::Token>,
    ) -> Result<&mut VertexData, NoMatch> {
        self.get_vertex_mut(self.get_token_index(token)?)
    }
    pub fn get_token_index(
        &self,
        token: impl AsToken<G::Token>,
    ) -> Result<VertexIndex, NoMatch> {
        Ok(self
            .graph
            .get_index_of(&self.get_token_key(token.as_token())?)
            .unwrap())
    }
    pub fn get_token_key(
        &self,
        token: impl AsToken<G::Token>,
    ) -> Result<VertexKey, NoMatch> {
        self.token_keys
            .get(&token.as_token())
            .copied()
            .ok_or(NoMatch::UnknownToken)
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
        self.get_token_index(token).expect("Token does not exist")
    }
    #[track_caller]
    pub fn expect_token_child(
        &self,
        token: impl AsToken<G::Token>,
    ) -> Child {
        Child::new(self.expect_token_index(token), 1)
    }
    pub fn to_token_keys_iter<'a>(
        &'a self,
        tokens: impl IntoIterator<Item = impl AsToken<G::Token>> + 'a,
    ) -> impl Iterator<Item = Result<VertexKey, NoMatch>> + 'a {
        tokens
            .into_iter()
            .map(move |token| self.get_token_key(token))
    }
    pub fn to_token_index_iter<'a>(
        &'a self,
        tokens: impl IntoIterator<Item = impl AsToken<G::Token>> + 'a,
    ) -> impl Iterator<Item = Result<VertexIndex, NoMatch>> + 'a {
        tokens
            .into_iter()
            .map(move |token| self.get_token_index(token))
    }
    pub fn to_token_children_iter<'a>(
        &'a self,
        tokens: impl IntoIterator<Item = impl AsToken<G::Token>> + 'a,
    ) -> impl Iterator<Item = Result<Child, NoMatch>> + 'a {
        self.to_token_index_iter(tokens)
            .map(move |r| r.map(|index| Child::new(index, 1)))
    }
    pub fn get_token_children(
        &self,
        tokens: impl IntoIterator<Item = impl AsToken<G::Token>>,
    ) -> Result<Pattern, NoMatch> {
        self.to_token_children_iter(tokens)
            .collect::<Result<Pattern, _>>()
    }
    #[track_caller]
    pub fn expect_token_children(
        &self,
        tokens: impl IntoIterator<Item = impl AsToken<G::Token>>,
    ) -> Pattern {
        self.get_token_children(tokens)
            .expect("Failed to convert tokens to children")
            .into_pattern()
    }
    pub fn get_token_indices(
        &self,
        tokens: impl IntoIterator<Item = impl AsToken<G::Token>>,
    ) -> Result<IndexPattern, NoMatch> {
        let tokens = tokens.into_iter();
        let mut v = IndexPattern::with_capacity(tokens.size_hint().0);
        for token in tokens {
            let index = self.get_token_index(token)?;
            v.push(index);
        }
        Ok(v)
    }
    pub fn expect_token_indices(
        &self,
        tokens: impl IntoIterator<Item = impl AsToken<G::Token>>,
    ) -> IndexPattern {
        self.get_token_indices(tokens)
            .expect("Failed to convert tokens to indices")
    }
}
