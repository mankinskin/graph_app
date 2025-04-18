use crate::split::vertex::output::CompleteLocations;
use context_search::{
    HashMap,
    graph::vertex::child::Child,
    trace::cache::entry::position::{
        Offset,
        SubSplitLocation,
    },
};
use derive_more::{
    Deref,
    DerefMut,
    From,
};

use super::position::PosKey;

#[derive(Default, Debug, Deref, DerefMut, From)]
pub struct Leaves(Vec<PosKey>);

impl Leaves
{
    pub fn collect_leaves(
        &mut self,
        index: &Child,
        offsets: CompleteLocations,
    ) -> HashMap<Offset, Vec<SubSplitLocation>>
    {
        offsets
            .into_iter()
            .filter_map(|(parent_offset, res)| {
                match res
                {
                    Ok(locs) => Some((parent_offset, locs)),
                    Err(_) =>
                    {
                        self.push(PosKey::new(*index, parent_offset));
                        None
                    }
                }
            })
            .collect()
    }
}
