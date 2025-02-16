use crate::{
    traversal::cache::key::directed::up::UpKey,
    HashMap,
};

#[derive(Clone, Debug)]
pub struct PruningState {
    pub count: usize,
    pub prune: bool,
}

pub type PruningMap = HashMap<UpKey, PruningState>;

pub trait PruneStates {
    fn clear(&mut self);
    fn pruning_map(&mut self) -> &mut PruningMap;
    //fn prune_not_below(
    //    &mut self,
    //    root: UpKey,
    //) {
    //    self.pruning_map()
    //        .iter_mut()
    //        .filter(|(k, _)| {
    //            k.index.width > root.index.width
    //                || (k.index.width == root.index.width && k.index != root.index)
    //        })
    //        .for_each(|(_, v)| {
    //            v.prune = true;
    //        });
    //}
    //fn prune_smaller(
    //    &mut self,
    //    root: Child,
    //) {
    //    self.pruning_map()
    //        .iter_mut()
    //        .filter(|(k, _)| {
    //            k.index.width < root.width || (k.index.width == root.width && k.index != root)
    //        })
    //        .for_each(|(_, v)| {
    //            v.prune = true;
    //        });
    //}
    fn prune_below(
        &mut self,
        root: UpKey,
    ) {
        if let Some(entry) = self.pruning_map().get_mut(&root) {
            entry.prune = true;
        }
    }
}

//impl<'a, 'b: 'a, K: TraversalKind> PruneStates for TraversalStateContext<'a, 'b, K> {
//    fn clear(&mut self) {
//        self.ctx.clear();
//    }
//    fn pruning_map(&mut self) -> &mut PruningMap {
//        self.ctx.pruning_map()
//    }
//}
//
