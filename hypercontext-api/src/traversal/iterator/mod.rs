
pub mod bands;
pub mod policy;

//pub type IterTrav<'a, It> = <IterKind<'a, It> as TraversalKind>::Trav;
//pub type IterKind<'a, It> = <It as TraversalIterator<'a>>::Kind;

// Traversal Iterator Spec
//
// Traversable, Pattern -> RangePath -> TraversalIterator -> TraversalResult
//                          search
//                                       TraversalCache
//                                       TraversalState
//                                       NextStates
//                                       TraversalContext
//                                       TraversalStateContext
//                                                             FoldFound
//
//

//pub trait TraversalIterator<'a>:
//    Iterator<Item = (usize, TraversalState)> + Sized + ExtendStates + PruneStates + Debug
//{
//    type Kind: TraversalKind + 'a;
//
//    fn ctx(&self) -> &'_ TraversalContext<'a, Self::Kind>;
//
//    //#[instrument(skip(self))]
//}
//
//impl<'a, K: TraversalKind + 'a> TraversalIterator<'a> for TraversalContext<'a, K> {
//    type Kind = K;
//    fn ctx(&self) -> &'_ TraversalContext<'a, Self::Kind> {
//        self
//    }
//}