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

use hypercontext_api::{
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
        Hypergraph, HypergraphRef,
    },
    traversal::{
        path::{
            mutators::move_path::Advance,
            structs::query_range_path::{
                PatternPrefixPath,
                QueryPath,
            },
        },
        traversable::TraversableMut,
    }
};
use crate::{
    insert::context::InsertContext, read::sequence::{
        SequenceIter,
        ToNewTokenIndices,
    },
};
pub mod band;
pub mod overlaps;
pub struct BandsContext {
    pub graph: HypergraphRef,
}
impl BandsContext {
    pub fn new(graph: HypergraphRef) -> Self {
        Self {
            graph,
        }
    }
    pub fn indexer(&self) -> InsertContext {
        InsertContext::new(self.graph.clone())
    }
    #[instrument(skip(self, sequence))]
    pub fn read(
        &mut self,
        mut sequence: PatternPrefixPath,
    ) {
        //println!("reading known bands");
        while let Some(next) = self.next_known_index(&mut sequence) {
            //println!("found next {:?}", next);
            let next = self.read_overlaps(next, &mut sequence).unwrap_or(next);
            self.append_index(next);
        }
    }
    #[instrument(skip(self, context))]
    fn next_known_index(
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

}