use std::{
    borrow::Borrow,
    ops::{
        Deref,
        DerefMut,
    },
    sync::RwLockWriteGuard,
};

use tracing::{
    debug,
    instrument,
};

use crate::{
    direction::Right,
    graph::{
        vertex::{
            child::Child,
            has_vertex_data::HasVertexDataMut,
            has_vertex_index::{
                HasVertexIndex,
                ToChild,
            },
            pattern::{
                IntoPattern,
                Pattern,
            },
        },
        Hypergraph,
    },
    read::sequence::{
        SequenceIter,
        ToNewTokenIndices,
    },
    search::NoMatch,
    traversal::{
        path::{
            mutators::move_path::Advance,
            structs::query_range_path::{
                PatternPrefixPath,
                QueryPath,
            },
        },
        traversable::TraversableMut,
    },
};

#[derive(Debug)]
pub struct ReadContext<'g> {
    pub graph: RwLockWriteGuard<'g, Hypergraph>,
    pub root: Option<Child>,
}
impl Deref for ReadContext<'_> {
    type Target = Hypergraph;
    fn deref(&self) -> &Self::Target {
        self.graph.deref()
    }
}
impl DerefMut for ReadContext<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.graph.deref_mut()
    }
}

impl<'g> ReadContext<'g> {
    pub fn new(graph: RwLockWriteGuard<'g, Hypergraph>) -> Self {
        Self { graph, root: None }
    }
    #[instrument(skip(self))]
    pub fn read_sequence<N, S: ToNewTokenIndices<N>>(
        &mut self,
        sequence: S,
    ) -> Option<Child> {
        debug!("start reading: {:?}", sequence);
        let sequence = sequence.to_new_token_indices(self);
        let mut sequence = SequenceIter::new(&sequence);
        while let Some((unknown, known)) = sequence.next_block(self) {
            // todo: read to result type
            self.append_pattern(unknown);
            self.read_known(known)
        }
        //println!("reading result: {:?}", index);
        self.root
    }
    pub fn read_pattern(
        &mut self,
        known: impl IntoPattern,
    ) -> Option<Child> {
        self.read_known(known.into_pattern());
        self.root
    }
    #[instrument(skip(self, known))]
    pub fn read_known(
        &mut self,
        known: Pattern,
    ) {
        match PatternPrefixPath::new_directed::<Right, _>(known.borrow()) {
            Ok(path) => self.read_bands(path),
            Err((err, _)) => match err {
                NoMatch::SingleIndex(c) => {
                    self.append_index(c);
                    Ok(())
                }
                NoMatch::EmptyPatterns => Ok(()),
                err => Err(err),
            }
            .unwrap(),
        }
    }
    #[instrument(skip(self, sequence))]
    fn read_bands(
        &mut self,
        mut sequence: PatternPrefixPath,
    ) {
        //println!("reading known bands");
        while let Some(next) = self.get_next(&mut sequence) {
            //println!("found next {:?}", next);
            let next = self.read_overlaps(next, &mut sequence).unwrap_or(next);
            self.append_index(next);
        }
    }
    #[instrument(skip(self, context))]
    fn get_next(
        &mut self,
        context: &mut PatternPrefixPath,
    ) -> Option<Child> {
        match self.indexer().index_query(context.clone()) {
            Ok((index, advanced)) => {
                *context = advanced;
                Some(index)
            }
            Err(_) => {
                context.advance(self);
                None
            }
        }
    }
    //pub fn indexer(&self) -> Indexer {
    //    Indexer::new(self.graph.clone())
    //}
    //pub fn contexter<Side: IndexSide<D>>(&self) -> Contexter<Side> {
    //    Contexter::new(self.indexer())
    //}
    //pub fn splitter<Side: IndexSide<D>>(&self) -> Splitter<Side> {
    //    Splitter::new(self.indexer())
    //}
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
            *root = if index.vertex_index() != root.vertex_index()
                && vertex.children.len() == 1
                && vertex.parents.is_empty()
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
            0 => {}
            1 => {
                let new = new.borrow().iter().next().unwrap();
                self.append_index(new)
            }
            _ => {
                if let Some(root) = &mut self.root {
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
                    let c = self.graph.insert_pattern(new);
                    self.root = Some(c);
                }
            }
        }
    }
}
