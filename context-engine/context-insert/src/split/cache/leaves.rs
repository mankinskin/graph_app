use crate::split::vertex::output::CompleteLocations;
use context_trace::*;
use derive_more::{
    Deref,
    DerefMut,
    From,
};

use crate::split::position::{PosKey};

#[derive(Default, Debug, Deref, DerefMut, From, Clone, PartialEq, Eq)]
pub struct Leaves(Vec<PosKey>);

impl Leaves {
    pub fn collect_leaves(
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
                },
            })
            .collect()
    }
}
