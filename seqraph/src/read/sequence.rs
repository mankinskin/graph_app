use crate::shared::*;
#[derive(Debug, Deref, DerefMut)]
pub struct SequenceIter<'it> {
    iter: std::iter::Peekable<std::slice::Iter<'it, NewTokenIndex>>
}
impl<'it> Iterator for SequenceIter<'it> {
    type Item = &'it NewTokenIndex;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
impl<'it> PeekingNext for SequenceIter<'it> {
    fn peeking_next<F>(&mut self, accept: F) -> Option<Self::Item>
        where
            Self: Sized,
            F: FnOnce(&Self::Item) -> bool {
        self.iter.peeking_next(accept)
    }
}
impl<'it> SequenceIter<'it> {
    pub fn new<N, S: ToNewTokenIndices<N>>(
        ctx: &mut ReadContext<'it>,
        sequence: S,
    ) -> Self {
        Self {
            iter: sequence.to_new_token_indices(ctx).iter().peekable(),
        }
    }
    pub fn next_block<'g>(
        &mut self,
        ctx: &mut ReadContext<'g>,
    ) -> Option<(Pattern, Pattern)> {
        let cache = self.take_while(|t| t.is_new());
        let known = self.take_while(|t| t.is_known());
        if cache.is_empty() && known.is_empty() {
            None
        } else {
            Some((cache, known))
        }
    }
    fn take_while(
        &mut self,
        f: impl FnMut(&<Self as Iterator>::Item) -> bool,
    ) -> Pattern
        where
            Child: From<<Self as Iterator>::Item>,
    {
        self.peeking_take_while(f).map(Child::from).collect()
    }
}

pub trait ToNewTokenIndices<N>: Debug {
    fn to_new_token_indices<
        'a: 'g,
        'g,
        Trav: TraversableMut<Kind=BaseGraphKind>,
        >(self, graph: &'a mut Trav) -> NewTokenIndices;
}

impl ToNewTokenIndices<NewTokenIndex> for NewTokenIndices {
    fn to_new_token_indices<
        'a: 'g,
        'g,
        Trav: TraversableMut<Kind=BaseGraphKind>,
    >(self, _graph: &'a mut Trav) -> NewTokenIndices {
        self
    }
}
//impl<T: Tokenize> ToNewTokenIndices<T> for Vec<T> {
//    fn to_new_token_indices<
//        'a: 'g,
//        'g,
//        Trav: TraversableMut<T>,
//        >(self, graph: &'a mut Trav) -> NewTokenIndices {
//        graph.graph_mut().new_token_indices(self)
//    }
//}

impl<Iter: IntoIterator<Item=DefaultToken> + Debug + Send + Sync> ToNewTokenIndices<DefaultToken> for Iter {
    fn to_new_token_indices<
        'a: 'g,
        'g,
        Trav: TraversableMut<Kind=BaseGraphKind>,
    >(self, graph: &'a mut Trav) -> NewTokenIndices {
        graph.graph_mut().new_token_indices(self)
    }
}