pub mod cache;
pub mod states;
//
// Traversal Iterator Spec
//
// Traversable, Pattern -> RangePath -> TraversalIterator -> FinishedState
//                          search
//                                       TraversalCache
//                                       TraversalState
//                                       NextStates
//                                       TraversalContext
//                                       TraversalStateContext
//                                                             FoldFound
//