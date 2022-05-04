use std::{sync::{RwLockReadGuard, RwLockWriteGuard}, collections::HashMap, borrow::Borrow};

use crate::{
    index::*,
    *,
};
use itertools::*;

#[derive(Debug, Default)]
struct ReaderBuffer {
    patterns: HashMap<usize, Pattern>,
}
impl ReaderBuffer {
    pub fn new(next: Child) -> Self {
        Self {
            patterns: HashMap::from([
                (next.width(), vec![next]),
            ])
        }
    }
    pub fn close<T: Tokenize, D: IndexDirection>(
        &mut self,
        reader: &'_ mut Reader<T, D>,
    ) -> Option<Child> {
        None
    }
}
#[derive(Debug)]
struct CacheRoot {
    index: Child,
    last_new: bool,
}
impl CacheRoot {
    pub fn new_unknown(index: Child) -> Self {
        Self {
            index,
            last_new: true,
        }
    }
    pub fn new_known(index: Child) -> Self {
        Self {
            index,
            last_new: false,
        }
    }
}
#[derive(Debug)]
pub struct Reader<T: Tokenize, D: IndexDirection> {
    graph: HypergraphRef<T>,
    //root: Option<CacheRoot>,
    //buffer: ReaderBuffer,
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
            panic!("Empty sequence")
        }
        let (unknown, known, remainder) = self.find_known_block(sequence);
        self.append_unknown(unknown);
        match PrefixPath::new_directed::<D, _>(known) {
            Ok(path) => self.read_known(path),
            Err(NoMatch::SingleIndex) => unimplemented!(),
            Err(err) => panic!("{:?}", err),
        }
        self.read_sequence(remainder)
    }
    fn read_known(&mut self, known: PrefixPath) {
        let (prefix, advanced) = self.read_prefix(known).expect("Empty known block!");
        let end_bound = prefix.width();
        let mut bands = ReadingBands::new();
        bands.insert(end_bound, vec![prefix]);
        self.read_index_overlaps_known_bands(prefix, end_bound, known, bands)
    }
    fn read_index_overlaps_known_bands(&mut self, index: Child, end_bound: usize, context: PrefixPath, mut bands: ReadingBands) {
        let bundle = vec![];
        // todo: check if context is empty
        match self.overlap_index(&mut bands, bundle, end_bound, index, context) {
            Ok((
                context_path,
                expansion,
                end_bound,
                context,
            )) => {
                // expanded
                self.read_index_overlaps_known_bands(expansion, end_bound, context, bands)
            },
            Err(bundle) => {
                // no overlap found, continue after last band (something left to do)
                let (next, advanced) = self.read_prefix(context).expect("Empty known block!");
                let old = bands.remove(&end_bound).expect("current index not in bands");
                let end_bound = end_bound + next.width();
                bands.insert(end_bound, [&old[..], next.borrow()].concat());
                self.read_index_overlaps_known_bands(next, end_bound, advanced, bands)
            }
        }
    }
    fn overlap_index_path(
        &mut self,
        bands: &mut ReadingBands,
        mut bundle: Vec<Pattern>,
        end_bound: usize,
        mut path: ChildPath,
        index: Child,
        context: PrefixPath,
    ) -> Result<(ChildPath, Child, usize, PrefixPath), Vec<Pattern>> {
        // find postfix with overlap
        let mut graph = self.graph_mut();
        let child_patterns = graph.expect_children_of(index);
        drop(graph);
        // collect postfixes in descending length
        let postfixes = child_patterns.iter().map(|(pid, pattern)| {
            let last = D::last_index(pattern);
            (ChildLocation::new(index, *pid, last), pattern[last].clone())
        })
        .sorted_by(|a, b|
            a.1.width().cmp(&b.1.width())
        ).collect_vec();
        // find largest expandable postfix
        postfixes.into_iter().fold(Err(None), |_, (loc, postfix)| {
            let start_bound = end_bound - postfix.width();
            let old = bands.remove(&start_bound);
            match self.graph.index_path_prefix(OverlapPrimer::new(postfix, context)) {
                Ok((expansion, advanced)) => {
                    // expanded
                    match bundle.len() {
                        0 => {},
                        1 => {
                            let band = bundle.into_iter().next().unwrap();
                            bands.insert(end_bound, band);
                        },
                        _ => {
                            let band = self.graph_mut().index_patterns(bundle);
                            bands.insert(end_bound, vec![band]);
                        }
                    }
                    let context = if let Some(old) = old {
                        &old[..]
                    } else {
                        &[IndexableSide::<T, D, IndexBack>::index_context_path(self, loc, path)]
                    };
                    let end_bound = start_bound + expansion.width();
                    bands.insert(end_bound, [context, expansion.borrow()].concat());
                    Ok((loc, expansion, end_bound, advanced.into_prefix_path()))
                },
                _ => {
                    // not expandable
                    if let Some(old) = old {
                        // remember if this comes right after existing band
                        bundle.push([&old[..], postfix.borrow()].concat());
                    }
                    Err(Some((loc, postfix)))
                }
            }
        })
        .map(|(loc, expansion, end_bound, advanced)| {
            path.push(loc);
            Ok((path, expansion, end_bound, advanced))
        })
        .unwrap_or_else(|res| 
            if let Some((loc, postfix)) = res {
                path.push(loc);
                self.overlap_index_path(bands, bundle, end_bound, path, postfix, context)
            } else {
                Err(bundle)
            }
        )
    }
    fn overlap_index(
        &mut self,
        bands: &mut ReadingBands,
        bundle: Vec<Pattern>,
        end_bound: usize,
        index: Child,
        context: PrefixPath,
    ) -> Result<(ChildPath, Child, usize, PrefixPath), Vec<Pattern>> {
        self.overlap_index_path(bands, bundle, end_bound, vec![], index, context)
    }
    pub(crate) fn indexer<Q: TraversalQuery>(&self) -> Indexer<T, D, Q> {
        Indexer::new(self.graph.clone())
    }
    pub(crate) fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    fn append_unknown(
        &mut self,
        new: Pattern,
    ) {
        if self.append_pattern_to_root(new).is_err() {
            match new.len() {
                0 => {},
                1 => {
                    let new = new.into_iter().next().unwrap();
                    if let Some(buffer) = self.buffer {
                        // TODO: respect direction
                        self.root = Some(CacheRoot::new_unknown(
                            self.graph_mut()
                                .index_pattern(vec![buffer, new]))
                        );
                    } else {
                        self.root = Some(CacheRoot::new_unknown(new));
                    }
                },
                _ => {
                    let new = if let Some(buffer) = self.buffer {
                        [&[buffer], &new[..]].concat()
                    } else {
                        new
                    };
                    // insert new pattern so it can be found in later queries
                    // any new indicies afterwards need to be appended
                    // to the pattern inside this index
                    let new = self.graph_mut().index_pattern(new);
                    self.root = Some(CacheRoot::new_unknown(new));
                }
            }
        }
    }
    /// append a pattern of new token indices
    /// returns index of possible new index
    fn append_pattern_to_root(
        &mut self,
        new: Pattern,
    ) -> Result<(), ()> {
        if let Some(CacheRoot {
            index: root,
            last_new,
        }) = self.root.as_mut() {
            let mut graph = &mut *self.graph_mut();
            let vertex = root.vertex_mut(&mut graph);
            *root = if vertex.children.len() == 1 && vertex.parents.len() == 0 {
                // if no old overlaps
                // append to single pattern
                // no overlaps because new
                let (&pid, _) = vertex.expect_any_pattern();
                graph.append_to_pattern(root, pid, new)
            } else {
                // some old overlaps though
                graph.index_pattern([&[*root], new.as_slice()].concat())
            };
            Result::Err(())
        } else {
            Result::Ok(())
        }
    }
    fn read_prefix(
        &mut self,
        query: PrefixPath,
    ) -> Option<(Child, PrefixPath)> {
        let mut indexer = self.indexer();
        match indexer.index_path_prefix(query) {
            Ok((index, query)) => Some((index, query)),
            Err(_not_found) => query.get_advance::<_, D, _>(&mut indexer),
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
trait ToNewTokenIndices<N, T: Tokenize> {
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
        >(self, graph: &'a mut Trav) -> NewTokenIndices {
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