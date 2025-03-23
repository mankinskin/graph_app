use crate::traversal::{
    cache::key::props::RootKey,
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
use itertools::Itertools;
use std::fmt::Debug;

#[derive(Debug, Default)]
pub struct PrunedStates<K: TraversalKind> {
    pub pruning_map: PruningMap,
    pub states: K::Container,
}
impl<K: TraversalKind> PruneStates for PrunedStates<K> {
    fn clear(&mut self) {
        self.states.clear();
    }
    fn pruning_map(&mut self) -> &mut PruningMap {
        &mut self.pruning_map
    }
}

impl<K: TraversalKind> StateContainer for PrunedStates<K> {
    fn clear(&mut self) {
        self.pruning_map.clear();
        self.states.clear();
    }
}
impl<K: TraversalKind> FromIterator<(usize, TraversalState)> for PrunedStates<K> {
    fn from_iter<T: IntoIterator<Item = (usize, TraversalState)>>(iter: T) -> Self {
        let mut r = Self::default();
        r.extend(iter.into_iter().collect_vec());
        r
    }
}

impl<K: TraversalKind> ExtendStates for PrunedStates<K> {
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
impl<K: TraversalKind> Iterator for PrunedStates<K> {
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
