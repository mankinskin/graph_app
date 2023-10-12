use petgraph::graph::EdgeIndex;
use std::{
    fmt::{
        self,
        Debug,
        Display,
    },
    hash::Hash, borrow::Borrow,
};

use crate::*;

pub fn tokenizing_iter<T: Tokenize, C: AsToken<T>>(
    seq: impl Iterator<Item = C>
) -> impl Iterator<Item = Token<T>> {
    seq.map(|c| c.as_token())
}
/// Trait for token that can be mapped in a sequence
pub trait Tokenize: TokenData + Wide + Hash + Eq + Copy + Debug + Send + Sync + 'static + Unpin {
    fn tokenize<T: AsToken<Self>, I: Iterator<Item = T>>(seq: I) -> Vec<Token<Self>> {
        let mut v = vec![];
        v.extend(tokenizing_iter(seq));
        //v.push(Token::End);
        v
    }
    fn into_token(self) -> Token<Self> {
        Token::Element(self)
    }
}
impl<T: TokenData + Wide + Hash + Eq + Copy + Debug + Send + Sync + 'static + Unpin> Tokenize for T {}

pub trait TokenData: Debug + PartialEq + Clone + Wide {}
impl<T: Debug + PartialEq + Clone + Wide> TokenData for T {}

#[derive(Hash, Debug, Clone, PartialEq, Eq, Copy)]
pub struct NoToken;

impl Wide for NoToken {
    fn width(&self) -> usize {
        0
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum NewTokenIndex {
    New(VertexIndex),
    Known(VertexIndex),
}
impl NewTokenIndex {
    pub fn is_known(&self) -> bool {
        matches!(self, Self::Known(_))
    }
    pub fn is_new(&self) -> bool {
        matches!(self, Self::New(_))
    }
}
impl Wide for NewTokenIndex {
    fn width(&self) -> usize {
        1
    }
}
impl Indexed for NewTokenIndex {
    fn vertex_index(&self) -> VertexIndex {
        match self {
            Self::New(i) => *i,
            Self::Known(i) => *i,
        }
    }
}
impl Borrow<VertexIndex> for &'_ NewTokenIndex {
    fn borrow(&self) -> &VertexIndex {
        match self {
            NewTokenIndex::New(i) => i,
            NewTokenIndex::Known(i) => i,
        }
    }
}
impl Borrow<VertexIndex> for &'_ mut NewTokenIndex {
    fn borrow(&self) -> &VertexIndex {
        match self {
            NewTokenIndex::New(i) => i,
            NewTokenIndex::Known(i) => i,
        }
    }
}
pub type NewTokenIndices = Vec<NewTokenIndex>;

pub trait AsToken<T: Tokenize> {
    fn as_token(&self) -> Token<T>;
}
impl<T: Tokenize> AsToken<T> for Token<T> {
    fn as_token(&self) -> Token<T> {
        *self
    }
}
impl<T: Tokenize> AsToken<T> for T {
    fn as_token(&self) -> Token<T> {
        Token::Element(*self)
    }
}

#[derive(Debug, Clone)]
pub struct ContextInfo<T: Tokenize> {
    pub token: Token<T>,
    pub incoming_groups: Vec<Vec<Token<T>>>,
    pub outgoing_groups: Vec<Vec<Token<T>>>,
}
pub trait ContextLink: Sized + Clone {
    fn index(&self) -> &EdgeIndex;
    fn into_index(self) -> EdgeIndex {
        *self.index()
    }
}
impl ContextLink for EdgeIndex {
    fn index(&self) -> &EdgeIndex {
        self
    }
}
pub trait ContextMapping<E: ContextLink> {
    /// Get distance groups for incoming edges
    fn incoming(&self) -> &Vec<E>;
    fn outgoing(&self) -> &Vec<E>;

    ///// Get distance groups for incoming edges
    //fn incoming_distance_groups(
    //    &self,
    //    graph: &SequenceGraph<T>,
    //) -> Vec<Vec<Self::Context>> {
    //    graph.distance_group_source_weights(self.incoming().iter().map(|e| e.into_index()))
    //}
    ///// Get distance groups for outgoing edges
    //fn outgoing_distance_groups(
    //    &self,
    //    graph: &SequenceGraph<T>,
    //) -> Vec<Vec<Self::Context>> {
    //    graph.distance_group_target_weights(self.outgoing().iter().map(|e| e.into_index()))
    //}
}

pub trait TokenContext<T: Tokenize, E: ContextLink>: Sized {
    type Mapping: ContextMapping<E>;
    fn token(&self) -> &Token<T>;
    fn into_token(self) -> Token<T>;
    fn map_to_tokens(groups: Vec<Vec<Self>>) -> Vec<Vec<Token<T>>> {
        groups
            .into_iter()
            .map(|g| g.into_iter().map(|m| m.into_token()).collect())
            .collect()
    }
    fn mapping(&self) -> &Self::Mapping;
    fn mapping_mut(&mut self) -> &mut Self::Mapping;
    //fn get_info(&self, graph: &SequenceGraph<T>) -> ContextInfo<T> {
    //    let mut incoming_groups = self.mapping().incoming_distance_groups(graph);
    //    incoming_groups.reverse();
    //    let outgoing_groups = self.mapping().outgoing_distance_groups(graph);
    //    ContextInfo {
    //        token: self.token().clone(),
    //        incoming_groups: Self::map_to_tokens(incoming_groups),
    //        outgoing_groups: Self::map_to_tokens(outgoing_groups),
    //    }
    //}
}
pub fn groups_to_string<T: Tokenize, E: ContextLink, C: TokenContext<T, E> + Display>(
    groups: Vec<Vec<C>>
) -> String {
    let mut lines = Vec::new();
    let max = groups.iter().map(Vec::len).max().unwrap_or(0);
    for i in 0..max {
        let mut line = Vec::new();
        for group in &groups {
            line.push(group.get(i).map(ToString::to_string));
        }
        lines.push(line);
    }
    lines.iter().fold(String::new(), |a, line| {
        format!(
            "{}{}\n",
            a,
            line.iter().fold(String::new(), |a, elem| {
                format!("{}{} ", a, elem.clone().unwrap_or_default())
            })
        )
    })
}

/// Type for storing elements of a sequence
#[derive(Copy, Debug, PartialEq, Clone, Eq, Hash)]
pub enum Token<T: Tokenize> {
    Element(T),
    Start,
    End,
}
impl<T: Tokenize + Display> Display for Token<T> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Token::Element(t) => t.to_string(),
                Token::Start => "START".to_string(),
                Token::End => "END".to_string(),
            }
        )
    }
}
impl<T: Tokenize> Wide for Token<T> {
    fn width(&self) -> usize {
        match self {
            Token::Element(t) => t.width(),
            Token::Start => 0,
            Token::End => 0,
        }
    }
}
impl<T: Tokenize> From<T> for Token<T> {
    fn from(e: T) -> Self {
        Token::Element(e)
    }
}
impl<T: Tokenize> PartialEq<T> for Token<T> {
    fn eq(
        &self,
        rhs: &T,
    ) -> bool {
        match self {
            Token::Element(e) => *e == *rhs,
            _ => false,
        }
    }
}