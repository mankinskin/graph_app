use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::{
    index::*,
    *,
};
use itertools::*;

#[derive(Debug, Default)]
struct ReaderCache {
    pub(crate) group: Option<Child>,
    buffer: Option<Child>,
}
impl ReaderCache {
    fn append_new<T: Tokenize, D: IndexDirection>(
        &mut self,
        reader: &'_ mut Reader<T, D>,
        new: Pattern,
    ) {
        if let Some(group) = self.group.as_mut() {
            *group = reader.append_new_pattern_to_index(group.clone(), new);
        } else {
            match new.len() {
                0 => {},
                1 => {
                    let new = new.into_iter().next().unwrap();
                    if let Some(buffer) = self.buffer.take() {
                        self.group = Some(reader.graph_mut().insert_pattern(vec![buffer, new]));
                    } else {
                        self.group = Some(reader.graph_mut().force_insert_pattern(vec![new]));
                    }
                },
                _ => {
                    let new = if let Some(buffer) = self.buffer.take() {
                        [&[buffer], &new[..]].concat()
                    } else {
                        new
                    };
                    // insert new pattern so it can be found in later queries
                    // any new indicies afterwards need to be appended
                    // to the pattern inside this index
                    let new = reader.graph_mut().insert_pattern(new);
                    self.group = Some(new);
                }
            }
        }
    }
    fn read_known<T: Tokenize, D: IndexDirection>(
        &mut self,
        reader: &'_ mut Reader<T, D>,
        known: Pattern,
    ) {
        if let Some(group) = self.group.as_mut() {
            let new = reader.overlap_index(*group, known);
            *group = new;
        } else {
            // first or second index
            if let Some(buffer) = self.buffer.take() {
                // second index
                let new = reader.overlap_index(buffer, known);
                self.group = Some(new);
            } else {
                // first index
                let (next, rem) = reader.read_prefix(known);
                self.buffer = Some(next);
                if !rem.is_empty() {
                    self.read_known(reader, rem);
                }
            }
        }
    }
    fn get(self) -> Option<Child> {
        self.group.or(self.buffer)
    }
}
#[derive(Debug)]
pub struct Reader<T: Tokenize, D: IndexDirection> {
    graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}
//impl<T: Tokenize, D: IndexDirection> std::ops::Deref for Reader<T, D> {
//    type Target = Hypergraph<T>;
//    fn deref(&self) -> &Self::Target {
//        &*self.graph.read().unwrap()
//    }
//}
//impl<T: Tokenize, D: IndexDirection> std::ops::DerefMut for Reader<T, D> {
//    fn deref_mut(&mut self) -> &mut Self::Target {
//        &mut *self.graph.write().unwrap()
//    }
//}
impl<T: Tokenize, D: IndexDirection> Reader<T, D> {
    pub(crate) fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    pub(crate) fn graph(&self) -> RwLockReadGuard<'_, Hypergraph<T>> {
        self.graph.read().unwrap()
    }
    pub(crate) fn graph_mut(&mut self) -> RwLockWriteGuard<'_, Hypergraph<T>> {
        self.graph.write().unwrap()
    }
    #[allow(unused)]
    pub(crate) fn indexer(&self) -> Indexer<T, D> {
        Indexer::new(self.graph.clone())
    }
    fn new_token_indices(
        &mut self,
        sequence: impl IntoIterator<Item = T>,
    ) -> NewTokenIndices {
        sequence
            .into_iter()
            .map(|t| Token::Element(t))
            .map(|t| match {
                let graph = self.graph();
                graph.get_token_index(t)
            } {
                Ok(i) => NewTokenIndex::Known(i),
                Err(_) => {
                    let i = self.graph_mut().insert_token(t);
                    NewTokenIndex::New(i.index)
                }
            })
            .collect()
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
    pub(crate) fn read_sequence(
        &mut self,
        sequence: impl IntoIterator<Item = T>,
    ) -> Child {
        let sequence: NewTokenIndices = self.new_token_indices(sequence);
        self.try_read_sequence(ReaderCache::default(), sequence)
    }
    fn try_read_sequence(
        &mut self,
        mut cache: ReaderCache,
        sequence: NewTokenIndices,
    ) -> Child {
        if sequence.is_empty() {
            cache.get().expect("Empty sequence")
        } else {
            let (new, known, remainder) = self.find_known_block(sequence);
            cache.append_new(self, new);
            cache.read_known(self, known);
            self.try_read_sequence(cache, remainder)
        }
    }
    fn read_prefix(
        &mut self,
        pattern: Vec<Child>,
    ) -> (Child, Pattern) {
        //let _pat_str = self.graph.pattern_string(&pattern);
        match self.indexer().index_pattern(&pattern) {
            Ok(_indexed_path) => {
                unimplemented!();
                //let (_lctx, inner, _rctx, rem) = self.index_found(found_path);
                //(inner, rem)
            }
            Err(_not_found) => {
                let (c, rem) = D::split_context_head(pattern).unwrap();
                (c, rem)
            },
        }
    }
    pub fn overlap_index(&mut self, index: Child, mut _context: Pattern) -> Child {
        // keep going down into next smallest postfi
        //unimplemented!();
        //while !context.is_empty() {
        //    let mut extensions = vec![];
        //    let mut smallest_postfix = Some(index);
        //    while let Some(current) = smallest_postfix.take() {
        //        let graph = &*self.graph();
        //        let vertex = current.vertex(&graph);
        //        // find extensions from index into context
        //        for (pid, mut p) in vertex.get_children().clone().into_iter() {
        //            let postfix = p.pop().unwrap().clone();
        //            // remember smallest postfix
        //            smallest_postfix = smallest_postfix.map_or(Some(postfix), |smallest|
        //                if postfix.width > 1 && smallest.width() > postfix.width() {
        //                    Some(postfix)
        //                } else {
        //                    Some(smallest)
        //                }
        //            );
        //            // if extension found
        //            if let Some(found) = graph.find_ancestor_in_context(postfix, context.clone()).ok() {
        //                // create index for extension
        //                let (_, extension, _, _rem) = self.indexer().index_found(found);
        //                let pre_context = self.indexer().index_pre_context_at(&ChildLocation::new(current, pid, p.len())).unwrap();
        //                // find pid with postfix context in extension
        //                extensions.push((pre_context, postfix, extension));
        //            }
        //        }
        //    }
        //    let (next, rem) = self.read_prefix(context.clone());
        //    index = self.append_new_pattern_to_index(index, next.into_pattern());
        //    for (pre_context, postfix, extension) in extensions.into_iter() {
        //        let n_pid = {
        //            let graph = &*self.graph();
        //            let pid = extension.vertex(&graph).find_child_pattern_id(|(_pid, pat)|
        //                *pat.first().unwrap() == postfix
        //            ).unwrap();
        //            let p = extension.vertex(&graph).get_child_pattern(&pid).unwrap();
        //            // postfix of extension
        //            let postfix = *p.last().unwrap();
        //            // find context of extension in next
        //            next.vertex(&graph).find_child_pattern_id(|(_pid, pat)|
        //                *pat.first().unwrap() == postfix
        //            ).unwrap()
        //        };
        //        let post_context = self.indexer().index_post_context_at(&ChildLocation::new(next, n_pid, 0)).unwrap();
        //        self.graph_mut().add_pattern_with_update(index, [pre_context, extension, post_context].as_slice());
        //    }
        //    context = rem;
        //}
        index
    }
    /// append a pattern of new token indices
    /// returns index of possible new index
    pub fn append_new_pattern_to_index(
        &mut self,
        parent: Child,
        new: Pattern,
    ) -> Child {
        let mut graph = &mut *self.graph_mut();
        let vertex = parent.vertex_mut(&mut graph);
        if vertex.children.len() == 1 && vertex.parents.len() == 0 {
            // if no old overlaps
            // append to single pattern
            // no overlaps because new
            let (&pid, _) = vertex.expect_any_pattern();
            graph.append_to_pattern(parent, pid, new)
        } else {
            // some old overlaps though
            graph.insert_pattern([&[parent], new.as_slice()].concat())
        }
    }
}