use itertools::Itertools;

use crate::traversal::{
    cache::{
        key::root::RootKey, state::traversal::TraversalState,
    },
    context::TraversalStateContext,
    iterator::TraversalIterator,
    policy::DirectedTraversalPolicy,
    traversable::Traversable,
};

use super::{pruning::PruningState, NodeVisitor, OrderedTraverser};
pub trait ExtendStates {
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState)>,
        T: IntoIterator<Item = (usize, TraversalState), IntoIter = It>,
    >(
        &mut self,
        iter: T,
    );
}

impl<Trav, S, O> ExtendStates for OrderedTraverser<'_, Trav, S, O>
where
    Trav: Traversable,
    S: DirectedTraversalPolicy<Trav = Trav>,
    O: NodeVisitor,
{
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState)>,
        In: IntoIterator<Item = (usize, TraversalState), IntoIter = It>,
    >(
        &mut self,
        iter: In,
    ) {
        let states = iter
            .into_iter()
            .map(|(d, s)| {
                // count states per root
                self.pruning_map
                    .entry(s.root_key())
                    .and_modify(|ps| ps.count += 1)
                    .or_insert(PruningState {
                        count: 1,
                        prune: false,
                    });
                (d, s)
            })
            .collect_vec();
        self.collection.extend(states)
    }
}

impl<'a, 'b: 'a, I: TraversalIterator<'b>> ExtendStates for TraversalStateContext<'a, 'b, I> {
    fn extend<
        It: DoubleEndedIterator + Iterator<Item = (usize, TraversalState)>,
        In: IntoIterator<Item = (usize, TraversalState), IntoIter = It>,
    >(
        &mut self,
        iter: In,
    ) {
        self.iter.extend(iter)
    }
}
