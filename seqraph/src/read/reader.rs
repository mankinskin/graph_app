use std::{sync::{RwLockReadGuard, RwLockWriteGuard}, collections::HashMap, borrow::Borrow};

use crate::{
    index::*,
    *,
};
use itertools::*;

#[derive(Debug)]
pub struct Reader<T: Tokenize, D: IndexDirection> {
    graph: HypergraphRef<T>,
    root: Option<Child>,
    _ty: std::marker::PhantomData<D>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection> Traversable<'a, 'g, T> for Reader<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard {
        self.graph.read().unwrap()
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection> TraversableMut<'a, 'g, T> for Reader<T, D> {
    type GuardMut = RwLockWriteGuard<'g, Hypergraph<T>>;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        self.graph.write().unwrap()
    }
}
type ReadingBands = HashMap<usize, Pattern>;
impl<T: Tokenize, D: IndexDirection> Reader<T, D> {
    pub(crate) fn read_sequence<N, S: ToNewTokenIndices<N, T>>(
        &mut self,
        sequence: S,
    ) -> Child {
        let mut sequence: NewTokenIndices = sequence.to_new_token_indices(self);
        while !sequence.is_empty() {
            let (unknown, known, remainder) = self.find_known_block(sequence);
            self.append_to_root(unknown);
            self.read_known(known);
            sequence = remainder;
        }
        self.root.unwrap()
    }
    fn read_known(&mut self, known: Pattern) {
        if let Ok(known) = PrefixPath::new_directed::<D, _>(known.borrow())
            .and_then(|path| self.read_context_path(path))
            .or_else(|err|
                (err == NoMatch::SingleIndex)
                    .then(|| known)
                    .ok_or(())
            )
        {
            self.append_to_root(known);
        }
    }
    fn read_context_path(&mut self, mut context: PrefixPath) -> Result<Pattern, NoMatch> {
        let mut bands = ReadingBands::new();
        let mut end_bound = 0;
        let mut last = self.get_next(&mut context).unwrap();
        while !context.is_finished(self) {
            if last.width() == 1 {
                end_bound = self.append_index(&mut bands, end_bound, last);
                last = self.get_next(&mut context).unwrap();
            } else {
                match self.overlap_index_path(&mut bands, end_bound, last, &context) {
                    Some((
                        expansion,
                        next_bound,
                        advanced,
                    )) => {
                        // expanded, continue overlapping
                        last = expansion;
                        end_bound = next_bound;
                        context = advanced;
                    },
                    None => {
                        self.append_index(&mut bands, end_bound, last);
                        last = context.advance::<_, D, _>(self).unwrap();
                        if let Some(next) = self.get_next(&mut context) {
                            last = next;
                        }
                    }
                }
            }
        }
        Ok(self.close_bands(&mut bands, end_bound, last))
    }
    fn get_next(&mut self, context: &mut PrefixPath) -> Option<Child> {
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
    fn expand_bands(bands: &mut ReadingBands, end_bound: usize, next: Child) -> usize {
        let (end_bound, band) = if let Some(old) = bands.remove(&end_bound) {
            (end_bound + next.width(), [&old[..], next.borrow()].concat())
        } else {
            (next.width(), vec![next])
        };
        bands.insert(end_bound, band);
        end_bound
    }
    fn append_index(&mut self, bands: &mut ReadingBands, end_bound: usize, index: Child) -> usize {
        if bands.is_empty() {
            self.append_to_root(index);
            end_bound
        } else {
            Self::expand_bands(bands, end_bound, index)
        }
    }
    fn overlap_index_path(
        &mut self,
        bands: &mut ReadingBands,
        end_bound: usize,
        index: Child,
        context: &PrefixPath,
    ) -> Option<(Child, usize, PrefixPath)> {
        let mut bundle = vec![vec![index]];
        // find largest expandable postfix
        let mut path = vec![];
        let end_bound = end_bound + index.width();
        PostfixIterator::<_, D, _>::new(&self.indexer(), index)
            .find_map(|(path_segment, loc, postfix)| {
                if let Some(segment) = path_segment {
                    path.push(segment);
                }
                let start_bound = end_bound - postfix.width();
                let old = bands.remove(&start_bound);
                match self.graph.index_query(OverlapPrimer::new(postfix, context.clone())) {
                    Ok((expansion, advanced)) => {
                        // expanded
                        match bundle.len() {
                            0 => {},
                            1 => {
                                let band = bundle.pop().unwrap();
                                bands.insert(end_bound, band);
                            },
                            _ => {
                                let band = self.graph_mut().index_patterns(bundle.drain(..));
                                bands.insert(end_bound, vec![band]);
                            }
                        }
                        let context = if let Some(old) = old {
                            old
                        } else {
                            vec![
                                SideIndexable::<T, D, IndexBack>::context_path(
                                    self,
                                    loc,
                                    path.clone(),
                                )
                            ]
                        };
                        let end_bound = start_bound + expansion.width();
                        bands.insert(end_bound, [&context[..], &[expansion]].concat());
                        Some((expansion, end_bound, advanced.into_prefix_path()))
                    },
                    _ => {
                        // not expandable
                        if let Some(old) = old {
                            // remember if this comes right after existing band
                            bundle.push([&old[..], postfix.borrow()].concat());
                        }
                        None
                    }
                }
            })
    }
    fn close_bands(
        &mut self,
        bands: &mut ReadingBands,
        end_bound: usize,
        index: Child,
    ) -> Pattern {
        let finisher = bands.remove(&end_bound);
        match (finisher, bands.is_empty()) {
            (None, _) => vec![index],
            (Some(mut finisher), true) => {
                finisher.push(index);
                finisher
            },
            (Some(finisher), _) => {
                let bundle = PostfixIterator::<_, D, _>::new(&self.indexer(), index)
                    .map_while(|(_path_segment, _loc, postfix)|
                        if !bands.is_empty() {
                            let start_bound = end_bound - postfix.width();
                            Some(bands.remove(&start_bound).map(|old|
                                [&old[..], &[postfix]].concat()
                            ))
                        } else {
                            None
                        }
                    )
                    .flatten()
                    .collect_vec();
                if bundle.is_empty() {
                    finisher
                } else {
                    vec![self.graph_mut().index_patterns([
                        &[finisher],
                        &bundle[..],
                    ].concat())]
                }
            }
        }
    }
    pub(crate) fn indexer(&self) -> Indexer<T, D> {
        Indexer::new(self.graph.clone())
    }
    pub(crate) fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            root: None,
            _ty: Default::default(),
        }
    }
    /// append a pattern of new token indices
    /// returns index of possible new index
    fn append_to_root(
        &mut self,
        new: impl IntoPattern,
    ) {
        if new.is_empty() {
            return;
        }
        if let Some(root) = &mut self.root {
            let mut graph = self.graph.graph_mut();
            let vertex = (*root).vertex_mut(&mut graph);
            *root = if vertex.children.len() == 1 && vertex.parents.is_empty() {
                // if no old overlaps
                // append to single pattern
                // no overlaps because new
                let (&pid, _) = vertex.expect_any_pattern();
                graph.append_to_pattern(*root, pid, new)
            } else {
                // some old overlaps though
                let new = new.into_pattern();
                graph.index_pattern([&[*root], new.as_slice()].concat())
            };
        } else {
            match new.borrow().len() {
                0 => {},
                1 => {
                    let new = new.borrow().iter().next().unwrap();
                    self.root = Some(*new);
                },
                _ => {
                    // insert new pattern so it can be found in later queries
                    // any new indicies afterwards need to be appended
                    // to the pattern inside this index
                    let new = self.graph_mut().index_pattern(new);
                    self.root = Some(new);
                }
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
        sequence: NewTokenIndices,
    ) -> (Pattern, Pattern, NewTokenIndices) {
        let mut seq_iter = sequence.into_iter().peekable();
        let cache = Self::take_while(&mut seq_iter, |t| t.is_new());
        let known = Self::take_while(&mut seq_iter, |t| t.is_known());
        (cache, known, seq_iter.collect())
    }
}
pub(crate) trait ToNewTokenIndices<N, T: Tokenize> {
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
impl<T: Tokenize, Iter: IntoIterator<Item=T>> ToNewTokenIndices<T, T> for Iter {
    fn to_new_token_indices<
        'a: 'g,
        'g,
        Trav: TraversableMut<'a, 'g, T>,
        >(self, graph: &'a mut Trav) -> NewTokenIndices {
        graph.graph_mut().new_token_indices(self)
    }
}