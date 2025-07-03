use context_trace::{
    graph::{
        kind::BaseGraphKind,
        vertex::{
            child::Child,
            pattern::Pattern,
            token::{
                NewTokenIndex,
                NewTokenIndices,
            },
        },
    },
    trace::has_graph::HasGraphMut,
};
use derive_more::{
    Deref,
    DerefMut,
};
use itertools::Itertools;

use std::{
    fmt::Debug,
    str::Chars,
};

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct BlockIter {
    iter: std::iter::Peekable<std::vec::IntoIter<NewTokenIndex>>,
}

#[derive(Debug, Clone)]
pub struct NextBlock {
    pub known: Pattern,
    pub unknown: Pattern,
}
impl Iterator for BlockIter {
    type Item = NextBlock;
    fn next(&mut self) -> Option<Self::Item> {
        let unknown = self.next_pattern_where(|t| t.is_new());
        let known = self.next_pattern_where(|t| t.is_known());
        if unknown.is_empty() && known.is_empty() {
            None
        } else {
            Some(NextBlock { unknown, known })
        }
    }
}

impl BlockIter {
    pub fn new(sequence: NewTokenIndices) -> Self {
        Self {
            iter: sequence.into_iter().peekable(),
        }
    }
    fn next_pattern_where(
        &mut self,
        f: impl FnMut(&NewTokenIndex) -> bool,
    ) -> Pattern {
        self.iter.peeking_take_while(f).map(Child::from).collect()
    }
}

pub trait ToNewTokenIndices: Debug {
    fn to_new_token_indices<'a: 'g, 'g, G: HasGraphMut<Kind = BaseGraphKind>>(
        self,
        graph: &'a mut G,
    ) -> NewTokenIndices;
}

impl ToNewTokenIndices for NewTokenIndices {
    fn to_new_token_indices<
        'a: 'g,
        'g,
        G: HasGraphMut<Kind = BaseGraphKind>,
    >(
        self,
        _graph: &'a mut G,
    ) -> NewTokenIndices {
        self
    }
}
impl ToNewTokenIndices for Chars<'_> {
    fn to_new_token_indices<
        'a: 'g,
        'g,
        G: HasGraphMut<Kind = BaseGraphKind>,
    >(
        self,
        graph: &'a mut G,
    ) -> NewTokenIndices {
        graph.graph_mut().new_token_indices(self)
    }
}
//impl<T: Tokenize> ToNewTokenIndices<T> for Vec<T> {
//    fn to_new_token_indices<'a: 'g, 'g, G: HasGraphMut>(
//        self,
//        graph: &'a mut G,
//    ) -> NewTokenIndices {
//        graph.graph_mut().new_token_indices(self)
//    }
//}

//impl<Iter: IntoIterator<Item = DefaultToken> + Debug + Send + Sync> ToNewTokenIndices<DefaultToken>
//    for Iter
//{
//    fn to_new_token_indices<'a: 'g, 'g, G: HasGraphMut<Kind = BaseGraphKind>>(
//        self,
//        graph: &'a mut G,
//    ) -> NewTokenIndices {
//        graph.graph_mut().new_token_indices(self)
//    }
//}
