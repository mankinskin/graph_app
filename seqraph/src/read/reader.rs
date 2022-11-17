use std::iter::Peekable;
use async_std::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::{
    index::*,
    *,
};
use itertools::*;

#[derive(Debug, Clone)]
pub struct Reader<T: Tokenize, D: IndexDirection> {
    pub(crate) graph: HypergraphRef<T>,
    pub(crate) root: Option<Child>,
    _ty: std::marker::PhantomData<D>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> Traversable<'a, 'g, T> for Reader<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard {
        self.graph.try_read().unwrap()
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> TraversableMut<'a, 'g, T> for Reader<T, D> {
    type GuardMut = RwLockWriteGuard<'g, Hypergraph<T>>;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        self.graph.try_write().unwrap()
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> Traversable<'a, 'g, T> for &'a Reader<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard {
        self.graph.try_read().unwrap()
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> Traversable<'a, 'g, T> for &'a mut Reader<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard {
        self.graph.try_read().unwrap()
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> TraversableMut<'a, 'g, T> for &'a mut Reader<T, D> {
    type GuardMut = RwLockWriteGuard<'g, Hypergraph<T>>;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        self.graph.try_write().unwrap()
    }
}
//type HashMap<K, V> = DeterministicHashMap<K, V>;

impl<T: Tokenize, D: IndexDirection> Reader<T, D> {
    #[instrument(skip(self))]
    pub(crate) fn read_sequence<N, S: ToNewTokenIndices<N, T>>(
        &mut self,
        sequence: S,
    ) -> Option<Child> {
        debug!("start reading: {:?}", sequence);
        let mut sequence = sequence.to_new_token_indices(self).into_iter().peekable();
        while let Some((unknown, known)) = self.find_known_block(&mut sequence) {
            self.append_pattern(unknown);
            self.read_known(known)
        }
        //println!("reading result: {:?}", index);
        self.root
    }
    pub(crate) fn read_pattern(&mut self, known: impl IntoPattern) -> Option<Child> {
        self.read_known(known.into_pattern());
        self.root
    }
    #[instrument(skip(self, known))]
    pub(crate) fn read_known(&mut self, known: Pattern) {
        match PrefixQuery::new_directed::<D, _>(known.borrow()) {
            Ok(path) => self.read_bands(path),
            Err(err) =>
                match err {
                    NoMatch::SingleIndex(c) => {
                        self.append_index(c);
                        Ok(())
                    },
                    NoMatch::EmptyPatterns => Ok(()),
                    err => Err(err)
                }.unwrap(),
        }
    }
    #[instrument(skip(self, sequence))]
    fn read_bands(&mut self, mut sequence: PrefixQuery) {
        //println!("reading known bands");
        while let Some(next) = self.get_next(&mut sequence) {
            //println!("found next {:?}", next);
            let next = self.read_overlaps(
                    next,
                    &mut sequence,
                )
                
                .unwrap_or(next);
            self.append_index(next);
        }
    }
    #[instrument(skip(self, context))]
    fn get_next(&mut self, context: &mut PrefixQuery) -> Option<Child> {
        match self.indexer().index_query(context.clone()) {
            Ok((index, advanced)) => {
                *context = advanced;
                Some(index)
            },
            Err(_) => {
                context.advance::<_, D, _>(self)
            }
        }
    }
    pub(crate) fn indexer(&self) -> Indexer<T, D> {
        Indexer::new(self.graph.clone())
    }
    pub(crate) fn contexter<Side: IndexSide<D>>(&self) -> Contexter<T, D, Side> {
        Contexter::new(self.indexer())
    }
    pub(crate) fn splitter<Side: IndexSide<D>>(&self) -> Splitter<T, D, Side> {
        Splitter::new(self.indexer())
    }
    pub(crate) fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            root: None,
            _ty: Default::default(),
        }
    }
    //fn append_next(&mut self, end_bound: usize, index: Child) -> usize {
    //    self.append_index(index);
    //    0
    //}
    #[instrument(skip(self, index))]
    fn append_index(
        &mut self,
        index: impl ToChild,
    ) {
        let index = index.to_child();
        if let Some(root) = &mut self.root {
            let mut graph = self.graph.graph_mut();
            let vertex = (*root).vertex_mut(&mut graph);
            *root = if
                index.index() != root.index() &&
                vertex.children.len() == 1 &&
                vertex.parents.is_empty()
            {
                let (&pid, _) = vertex.expect_any_child_pattern();
                graph.append_to_pattern(*root, pid, index)
            } else {
                graph.insert_pattern([*root, index])
            };
        } else {
            self.root = Some(index);
        }
    }
    /// append a pattern of new token indices
    /// returns index of possible new index
    fn append_pattern(
        &mut self,
        new: impl IntoPattern,
    ) {
        match new.borrow().len() {
            0 => {},
            1 => {
                let new = new.borrow().iter().next().unwrap();
                self.append_index(new)
            },
            _ => if let Some(root) = &mut self.root {
                    let mut graph = self.graph.graph_mut();
                    let vertex = (*root).vertex_mut(&mut graph);
                    *root = if vertex.children.len() == 1 && vertex.parents.is_empty() {
                        let (&pid, _) = vertex.expect_any_child_pattern();
                        graph.append_to_pattern(*root, pid, new)
                    } else {
                        // some old overlaps though
                        let new = new.into_pattern();
                        graph.insert_pattern([&[*root], new.as_slice()].concat())
                    };
                } else {
                    let c = self.graph_mut().insert_pattern(new);
                    self.root = Some(c);
                }
        }
    }
    fn take_while<I, J: Iterator<Item = I> + itertools::PeekingNext>(
        iter: &mut J,
        f: impl FnMut(&I) -> bool,
    ) -> Pattern
    where
        Child: From<I>,
    {
        iter.peeking_take_while(f).map(Child::from).collect()
    }
    fn find_known_block(
        &mut self,
        sequence: &mut Peekable<impl Iterator<Item = NewTokenIndex>>,
    ) -> Option<(Pattern, Pattern)> {
        let cache = Self::take_while(sequence, |t| t.is_new());
        let known = Self::take_while(sequence, |t| t.is_known());
        if cache.is_empty() && known.is_empty() {
            None
        } else {
            Some((cache, known))
        }
    }
}

pub(crate) trait ToNewTokenIndices<N, T: Tokenize>: Debug {
    fn to_new_token_indices<
        'a: 'g,
        'g,
        Trav: TraversableMut<'a, 'g, T>,
        >(self, graph: &'a mut Trav) -> NewTokenIndices;
}

impl<T: Tokenize> ToNewTokenIndices<NewTokenIndex, T> for NewTokenIndices {
    fn to_new_token_indices<
        'a: 'g,
        'g,
        Trav: TraversableMut<'a, 'g, T>,
    >(self, _graph: &'a mut Trav) -> NewTokenIndices {
        self
    }
}
//impl<T: Tokenize> ToNewTokenIndices<T> for Vec<T> {
//    fn to_new_token_indices<
//        'a: 'g,
//        'g,
//        Trav: TraversableMut<'a, 'g, T>,
//        >(self, graph: &'a mut Trav) -> NewTokenIndices {
//        graph.graph_mut().new_token_indices(self)
//    }
//}

impl<T: Tokenize, Iter: IntoIterator<Item=T> + Debug + Send + Sync> ToNewTokenIndices<T, T> for Iter {
    fn to_new_token_indices<
        'a: 'g,
        'g,
        Trav: TraversableMut<'a, 'g, T>,
    >(self, graph: &'a mut Trav) -> NewTokenIndices {
        graph.graph_mut().new_token_indices(self)
    }
}