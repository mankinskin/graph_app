use crate::*;

pub trait GraphKind: std::fmt::Debug + Clone {
    type Token: Tokenize;
    type Direction: MatchDirection;
}

#[derive(Debug, Clone)]
pub struct BaseGraphKind;

impl GraphKind for BaseGraphKind {
    type Token = char;
    type Direction = Right;
}
//impl<G: GraphKind> GraphKind for Hypergraph<G> {
//    type Token = G::Token;
//    type Direction = G::Direction;
//}
//impl<G: GraphKind> GraphKind for HypergraphRef<G> {
//    type Token = G::Token;
//    type Direction = G::Direction;
//}
//impl<G: GraphKind> GraphKind for &'_ G {
//    type Token = G::Token;
//    type Direction = G::Direction;
//}
//impl<G: GraphKind> GraphKind for &'_ mut G {
//    type Token = G::Token;
//    type Direction = G::Direction;
//}
//impl<G: GraphKind> GraphKind for RwLockReadGuard<'_, G> {
//    type Token = G::Token;
//    type Direction = G::Direction;
//}
//impl<G: GraphKind> GraphKind for RwLockWriteGuard<'_, G> {
//    type Token = G::Token;
//    type Direction = G::Direction;
//}
//impl<G: GraphKind> GraphKind for Indexer<G> {
//    type Token = G::Token;
//    type Direction = G::Direction;
//}
//impl<G: GraphKind> GraphKind for Searcher<G> {
//    type Token = G::Token;
//    type Direction = G::Direction;
//}
//impl<G: GraphKind, Side: IndexSide<G::Direction>> GraphKind for Pather<G, Side> {
//    type Token = G::Token;
//    type Direction = G::Direction;
//}