use super::{
    container::{
        extend::ExtendStates,
        pruning::{
            PruneStates,
            PruningMap,
            PruningState,
        },
        StateContainer,
    },
    state::traversal::TraversalState,
    TraversalKind,
};
use crate::traversal::cache::{
    key::root::RootKey,
    TraversalCache,
};
use itertools::Itertools;
use std::fmt::Debug;

//  1. Input
//      - Pattern
//      - QueryState
//  2. Init
//      - Trav
//      - start index
//      - start states
//  3. Fold
//      - TraversalCache
//      - FoldStepState

/// context for generating next states
#[derive(Debug)]
pub struct TraversalContext<'a, K: TraversalKind> {
    pub states: &'a mut StatesContext<K>,
    pub trav: &'a K::Trav,
}

impl<K: TraversalKind> Unpin for TraversalContext<'_, K> {}

#[derive(Debug)]
pub struct StatesContext<K: TraversalKind> {
    pub cache: TraversalCache,
    pub pruning_map: PruningMap,
    pub states: K::Container,
}
impl<K: TraversalKind> PruneStates for StatesContext<K> {
    fn clear(&mut self) {
        self.states.clear();
    }
    fn pruning_map(&mut self) -> &mut PruningMap {
        &mut self.pruning_map
    }
}
impl<K: TraversalKind> ExtendStates for StatesContext<K> {
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
        self.states.extend(states)
    }
}
impl<K: TraversalKind> Iterator for StatesContext<K> {
    type Item = (usize, TraversalState);

    fn next(&mut self) -> Option<Self::Item> {
        for (d, s) in self.states.by_ref() {
            let e = self.pruning_map.get_mut(&s.root_key()).unwrap();
            e.count -= 1;
            let pass = !e.prune;
            if e.count == 0 {
                self.pruning_map.remove(&s.root_key());
            }
            if pass {
                return Some((d, s));
            }
        }
        None
    }
}
