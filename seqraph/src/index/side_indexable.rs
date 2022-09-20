use crate::*;
use super::*;
use context::*;
use split::*;

type HashSet<T> = DeterministicHashSet<T>;
type HashMap<K, V> = DeterministicHashMap<K, V>;

pub(crate) trait SideIndexable<'a: 'g, 'g, T: Tokenize, D: IndexDirection, Side: IndexSide<D>>: IndexSplit<'a, 'g, T, D, Side> + IndexContext<'a, 'g, T, D, Side> {
}

impl<
    'a: 'g,
    'g,
    T: Tokenize,
    D: IndexDirection,
    Trav: Indexing<'a, 'g, T, D>,
    S: IndexSide<D>,
> SideIndexable<'a, 'g, T, D, S> for Trav {}