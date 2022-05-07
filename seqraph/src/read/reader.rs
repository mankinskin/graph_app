use std::{sync::{RwLockReadGuard, RwLockWriteGuard}, collections::HashMap, borrow::Borrow, ops::ControlFlow};

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
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> Traversable<'a, 'g, T> for Reader<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard {
        self.graph.read().unwrap()
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> TraversableMut<'a, 'g, T> for Reader<T, D> {
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
        let sequence: NewTokenIndices = sequence.to_new_token_indices(self);
        if sequence.is_empty() {
            self.root.unwrap()
        } else {
            let (unknown, known, remainder) = self.find_known_block(sequence);
            self.append_pattern_to_root(unknown);
            self.read_known(known);
            self.read_sequence(remainder)
        }
    }
    fn read_known(&mut self, known: Pattern) {
        match known.len() {
            0 => {},
            1 => self.append_pattern_to_root(known),
            _ => if let Ok(known) = PrefixPath::new_directed::<D, _>(known)
                .and_then(|path| self.read_known_path(path))
            {
                self.append_pattern_to_root(known);
            }
        }
    }
    fn read_known_path(&mut self, known: PrefixPath) -> Result<Pattern, NoMatch> {
        self.read_next_bands(known, ReadingBands::new(), 0)
    }
    fn read_next_bands(&mut self, known: PrefixPath, mut bands: ReadingBands, end_bound: usize) -> Result<Pattern, NoMatch> {
        let mut indexer = self.indexer();
        let (next, advanced) = match indexer.index_path_prefix(known.clone()) {
            Ok((index, query)) => (index, query),
            Err(_) => known.get_advance::<_, D, _>(&mut indexer),
        };
        let (end_bound, band) = if let Some(old) = bands.remove(&end_bound) {
            (end_bound + next.width(), [&old[..], next.borrow()].concat())
        } else {
            (next.width(), vec![next])
        };
        bands.insert(end_bound, band);
        self.overlap_index(next, end_bound, advanced, bands)
    }
    fn overlap_index(&mut self, index: Child, end_bound: usize, context: PrefixPath, mut bands: ReadingBands) -> Result<Pattern, NoMatch> {
        if context.is_finished() {
            Ok(self.close_bands(&mut bands, end_bound, index))
        } else if index.width() > 1 {
            match self.overlap_index_path(&mut bands, end_bound, index, context) {
                Ok((
                    _context_path,
                    expansion,
                    end_bound,
                    context,
                )) => {
                    // expanded, continue overlapping
                    self.overlap_index(expansion, end_bound, context, bands)
                },
                Err((_bundle, mut context)) => {
                    // no overlap found, continue after last band
                    context.advance_next::<T, D, _>(self);
                    self.read_next_bands(context, bands, end_bound)
                }
            }
        } else {
            self.read_next_bands(context, bands, end_bound)
        }
    }
    fn overlap_index_path(
        &mut self,
        bands: &mut ReadingBands,
        end_bound: usize,
        index: Child,
        context: PrefixPath,
    ) -> Result<(ChildPath, Child, usize, PrefixPath), (Vec<Pattern>, PrefixPath)> {
        let mut bundle = vec![];
        // find largest expandable postfix
        let mut path = vec![];
        match PostfixIterator::<_, D, _>::new(&self.indexer(), index)
            .find_map(|(path_segment, loc, postfix)| {
                if let Some(segment) = path_segment {
                    path.push(segment);
                }
                let start_bound = end_bound - postfix.width();
                let old = bands.remove(&start_bound);
                match self.graph.index_path_prefix(OverlapPrimer::new(postfix, context.clone())) {
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
                                IndexableSide::<T, D, IndexBack>::index_context_path(
                                    self,
                                    loc,
                                    path.clone(),
                                )
                            ]
                        };
                        let end_bound = start_bound + expansion.width();
                        bands.insert(end_bound, [&context[..], &[expansion]].concat());
                        Some((loc, expansion, end_bound, advanced.into_prefix_path()))
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
            }) {
            Some((loc, expansion, end_bound, advanced)) => {
                path.push(loc);
                Ok((path, expansion, end_bound, advanced))
            },
            None => Err((bundle, context))
        }
    }
    fn close_bands(
        &mut self,
        bands: &mut ReadingBands,
        end_bound: usize,
        index: Child,
    ) -> Pattern {
        let finisher = bands.remove(&end_bound).expect("closing on at empty end_bound!");
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
            .filter_map(|item| item)
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
    fn append_pattern_to_root(
        &mut self,
        new: Pattern,
    ) {
        if let Some(root) = &mut self.root {
            let mut graph = self.graph.graph_mut();
            let vertex = (*root).vertex_mut(&mut graph);
            *root = if vertex.children.len() == 1 && vertex.parents.len() == 0 {
                // if no old overlaps
                // append to single pattern
                // no overlaps because new
                let (&pid, _) = vertex.expect_any_pattern();
                graph.append_to_pattern(*root, pid, new)
            } else {
                // some old overlaps though
                graph.index_pattern([&[*root], new.as_slice()].concat())
            };
        } else {
            match new.len() {
                0 => {},
                1 => {
                    let new = new.into_iter().next().unwrap();
                    self.root = Some(new);
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