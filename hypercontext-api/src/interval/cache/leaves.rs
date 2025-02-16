use crate::{
    graph::{
        getters::vertex::VertexSet,
        vertex::child::Child,
    },
    traversal::{
        cache::entry::{
            position::SubSplitLocation,
            CompleteLocations,
            Offset,
        },
        traversable::Traversable,
    },
    HashMap,
};
use derive_more::{
    Deref,
    DerefMut,
    From,
};

use super::{
    PosKey,
    TraceState,
};

#[derive(Default, Debug, Deref, DerefMut, From)]
pub struct Leaves(Vec<PosKey>);

impl Leaves {
    pub fn filter_leaves(
        &mut self,
        index: &Child,
        offsets: CompleteLocations,
    ) -> HashMap<Offset, Vec<SubSplitLocation>> {
        offsets
            .into_iter()
            .filter_map(|(parent_offset, res)| match res {
                Ok(locs) => Some((parent_offset, locs)),
                Err(_) => {
                    self.push(PosKey::new(*index, parent_offset));
                    None
                }
            })
            .collect()
    }
    /// kind of like filter_leaves but from subsplits to trace states
    pub fn filter_trace_states<Trav: Traversable>(
        &mut self,
        trav: &Trav,
        index: &Child,
        pos_splits: impl IntoIterator<Item = (Offset, Vec<SubSplitLocation>)>,
    ) -> Vec<TraceState> {
        let graph = trav.graph();
        let node = graph.expect_vertex(index);
        let (perfect, next) = pos_splits
            .into_iter()
            .flat_map(|(parent_offset, locs)| {
                let len = locs.len();
                locs.into_iter().map(move |sub|
                    // filter sub locations without offset (perfect splits)
                    sub.inner_offset.map(|offset|
                        TraceState {
                            index: *node.expect_child_at(&sub.location),
                            offset,
                            prev: PosKey {
                                index: *index,
                                pos: parent_offset,
                            },
                        }
                    ).ok_or_else(||
                        (len == 1).then(||
                            PosKey::new(*index, parent_offset)
                        )
                    ))
            })
            .fold((Vec::new(), Vec::new()), |(mut p, mut n), res| {
                match res {
                    Ok(s) => n.push(s),
                    Err(Some(k)) => p.push(k),
                    Err(None) => {}
                }
                (p, n)
            });
        self.extend(perfect);
        next
    }
}
