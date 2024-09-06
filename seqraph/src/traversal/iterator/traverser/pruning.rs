use crate::{
    HashMap,
    traversal::{
        cache::key::UpKey,
        context::TraversalContext,
        iterator::{
            TraversalIterator,
            traverser::{
                NodeVisitor,
                OrderedTraverser,
            },
        },
        policy::DirectedTraversalPolicy,
        traversable::Traversable,
    },
};
use crate::graph::vertex::child::Child;

#[derive(Clone, Debug)]
pub struct PruningState {
    pub count: usize,
    pub prune: bool,
}

pub type PruningMap = HashMap<UpKey, PruningState>;

pub trait PruneStates {
    fn clear(&mut self);
    fn pruning_map(&mut self) -> &mut PruningMap;
    fn prune_not_below(
        &mut self,
        root: UpKey,
    ) {
        self.pruning_map()
            .iter_mut()
            .filter(|(k, _)| {
                k.index.width > root.index.width
                    || (k.index.width == root.index.width && k.index != root.index)
            })
            .for_each(|(_, v)| {
                v.prune = true;
            });
    }
    fn prune_smaller(
        &mut self,
        root: Child,
    ) {
        self.pruning_map()
            .iter_mut()
            .filter(|(k, _)| {
                k.index.width < root.width || (k.index.width == root.width && k.index != root)
            })
            .for_each(|(_, v)| {
                v.prune = true;
            });
    }
    fn prune_below(
        &mut self,
        root: UpKey,
    ) {
        if let Some(entry) = self.pruning_map().get_mut(&root) {
            entry.prune = true;
        }
    }
}

impl<'a, Trav, S, O> PruneStates for OrderedTraverser<'a, Trav, S, O>
where
    Trav: Traversable,
    S: DirectedTraversalPolicy<Trav = Trav>,
    O: NodeVisitor,
{
    fn clear(&mut self) {
        self.collection.clear();
    }
    fn pruning_map(&mut self) -> &mut PruningMap {
        &mut self.pruning_map
    }
}

impl<'a, 'b: 'a, I: TraversalIterator<'b>> PruneStates for TraversalContext<'a, 'b, I> {
    fn clear(&mut self) {
        self.iter.clear();
    }
    fn pruning_map(&mut self) -> &mut PruningMap {
        self.iter.pruning_map()
    }
}
