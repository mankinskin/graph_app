use std::{sync::{RwLockReadGuard, RwLockWriteGuard}, iter::Peekable};

use crate::{
    index::*,
    *,
};
use itertools::*;
use tap::Tap;

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
type HashMap<K, V> = DeterministicHashMap<K, V>;

struct ReadingBands {
    bands: HashMap<usize, Vec<Pattern>>,
}
impl Deref for ReadingBands {
    type Target = HashMap<usize, Vec<Pattern>>;
    fn deref(&self) -> &Self::Target {
        &self.bands
    }
}
impl DerefMut for ReadingBands {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bands
    }
}
impl ReadingBands {
    pub fn new(first: Child) -> Self {
        let mut bands = HashMap::default();
        bands.insert(first.width(), vec![vec![first]]);
        Self {
            bands,
        }
    }
    pub fn take_band<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        Trav: TraversableMut<'a, 'g, T> + 'a,
    >(&mut self, trav: &'a mut Trav, end_bound: usize) -> Option<Pattern> {
        let bundle = self.bands.remove(&end_bound)?;
        Some(match bundle.len() {
            0 => panic!("Empty bundle in bands!"),
            1 => bundle.into_iter().next().unwrap(),
            _ => {
                let mut graph = trav.graph_mut();
                let bundle = graph.index_patterns(bundle);
                vec![bundle]
            }
        })
    }
    pub fn append_index<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        Trav: TraversableMut<'a, 'g, T> + 'a,
    >(&mut self, trav: &'a mut Trav, end_bound: usize, index: Child) -> usize {
        let (end_bound, bundle) = self.take_band(trav, end_bound)
            .map(|mut bundle|
                (
                    end_bound + index.width(),
                    vec![{
                        bundle.push(index);
                        bundle
                    }]
                )
            )
            .unwrap_or_else(|| (index.width(), vec![vec![index]]));
        self.bands.insert(end_bound, bundle);
        end_bound
    }
    pub fn add_band(&mut self, end_bound: usize, band: Pattern) {
        self.bands.insert(end_bound, vec![band]);
    }
}
struct OverlapNode {
    end_bound: usize,
    back_context: Pattern,
    index: Child,
}
impl<T: Tokenize, D: IndexDirection> Reader<T, D> {
    pub(crate) fn read_sequence<N, S: ToNewTokenIndices<N, T>>(
        &mut self,
        sequence: S,
    ) -> Child {
        let mut sequence = sequence.to_new_token_indices(self).into_iter().peekable();
        while let Some((unknown, known)) = self.find_known_block(&mut sequence) {
            self.append_pattern(unknown);
            self.read_known(known)
        }
        self.root.unwrap()
    }
    fn read_known(&mut self, known: Pattern) {
        PrefixPath::new_directed::<D, _>(known.borrow())
            .map(|path| self.read_bands(path))
            .or_else(|err|
                match err {
                    NoMatch::SingleIndex => {
                        self.append_index(*known.first().unwrap());
                        Ok(())
                    },
                    NoMatch::EmptyPatterns => Ok(()),
                    err => Err(err)
                }
            )
            .unwrap()
    }
    fn read_bands(&mut self, mut sequence: PrefixPath) {
        self.read_recursive(sequence)
    }
    fn read_recursive(&mut self, mut context: PrefixPath) {
        //bands.insert(end_bound, vec![back_context.push(last)
        if let Some(next) = self.read_single(&mut context) {
            self.read_recursive(context)
        }
        //self.close_bands(bands, end_bound)
    }
    fn read_single(
        &mut self,
        context: &mut PrefixPath,
    ) -> Option<Child> {
        let next = self.get_next(context)?;
        let next = (next.width() != 1)
            .then(||
                self.read_overlaps(
                    ReadingBands::new(next),
                    OverlapNode {
                        end_bound: next.width(),
                        back_context: vec![],
                        index: next
                    },
                    context,
                )
            )
            .flatten()
            .unwrap_or_else(|| {
                next
            });
        self.append_index(next);
        Some(next)
    }
    fn read_overlaps(
        &mut self,
        mut bands: ReadingBands,
        node: OverlapNode,
        context: &mut PrefixPath,
    ) -> Option<Child> {
        let end_bound = node.end_bound;
        if let Some(node) = self.find_index_overlap(
            &mut bands,
            node,
            context,
        ) {
            self.read_overlaps(
                bands,
                node,
                context,
            )
        } else {
            self.close_bands(bands, end_bound)
        }
    }
    /// finds the earliest overlap
    fn find_index_overlap(
        &mut self,
        bands: &mut ReadingBands,
        node: OverlapNode,
        prefix_path: &mut PrefixPath,
    ) -> Option<OverlapNode> {
        let OverlapNode {
            end_bound,
            back_context,
            index,
        } = node;
        // find largest expandable postfix
        let mut path = vec![];
        // bundles of bands with same lengths for new index
        PostfixIterator::<_, D, _>::new(&self.indexer(), index)
            .find_map(|(path_segment, loc, postfix)| {
                if let Some(segment) = path_segment {
                    path.push(segment);
                }
                let start_bound = end_bound - postfix.width();
                // if at band boundary and bundle exists, index band
                let band = bands.take_band(&mut self.indexer(), start_bound);
                // expand
                match self.graph.index_query(OverlapPrimer::new(postfix, prefix_path.clone()))
                    .map(|(expansion, advanced)| {
                        *prefix_path = advanced.into_prefix_path();
                        expansion
                    }) {
                        Ok(expansion) => {
                            let next_bound = start_bound + expansion.width();
                            let node =
                            OverlapNode {
                                end_bound: next_bound,
                                back_context: band.unwrap_or_else(||
                                    // create band with generated back context
                                    vec![
                                        SideIndexable::<T, D, IndexBack>::context_path(
                                            self,
                                            loc,
                                            path.clone(),
                                            postfix,
                                        ).0,
                                    ]
                                ),
                                index: expansion,
                            };
                            bands.add_band(
                                node.end_bound,
                                node.back_context.clone().tap_mut(|b| b.push(node.index)),
                            );
                            Some(node)
                        },
                        Err(_) => {
                            // if not expandable, at band boundary and no bundle exists, create bundle
                            if let Some(mut band) = band {
                                band.push(postfix);
                                bands.insert(
                                    end_bound,
                                    vec![
                                        back_context.clone().tap_mut(|b| b.push(index)),
                                        band,
                                    ]
                                );
                            }
                            None
                        },
                    }
            })
    }
    fn close_bands(
        &mut self,
        mut bands: ReadingBands,
        end_bound: usize,
    ) -> Option<Child> {
        (!bands.is_empty()).then(|| {
            let bundle = bands.remove(&end_bound).expect("Bands not finished at end_bound!");
            assert!(bands.is_empty(), "Bands not finished!");
            self.indexer().graph_mut().index_patterns(bundle)
        })
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
    fn append_next(&mut self, end_bound: usize, index: Child) -> usize {
        self.append_index(index);
        0
    }
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
                let (&pid, _) = vertex.expect_any_pattern();
                graph.append_to_pattern(*root, pid, index)
            } else {
                graph.index_pattern([*root, index])
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
                        let (&pid, _) = vertex.expect_any_pattern();
                        graph.append_to_pattern(*root, pid, new)
                    } else {
                        // some old overlaps though
                        let new = new.into_pattern();
                        graph.index_pattern([&[*root], new.as_slice()].concat())
                    };
                } else {
                    let c = self.graph_mut().index_pattern(new);
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