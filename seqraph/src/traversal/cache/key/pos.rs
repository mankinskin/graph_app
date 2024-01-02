use crate::shared::*;

pub trait QueryPosition {
    fn query_pos(&self) -> &TokenLocation;
    fn query_pos_mut(&mut self) -> &mut TokenLocation;
}
macro_rules! impl_query_pos {
    {
        QueryPosition for $target:ty, $self_:ident => $func:expr
    } => {
        impl QueryPosition for $target {
            fn query_pos(& $self_) -> &TokenLocation {
                &$func
            }
            fn query_pos_mut(&mut $self_) -> &mut TokenLocation {
                &mut $func
            }
        }
    };
}
impl_query_pos!{
    QueryPosition for QueryState, self => self.pos
}
impl_query_pos! {
    QueryPosition for PathPair, self => self.query.pos
}
impl_query_pos! {
    QueryPosition for ParentState, self => self.query.pos
}
impl_query_pos! {
    QueryPosition for ChildState, self => self.paths.query.pos
}
impl_query_pos! {
    QueryPosition for StartState, self => self.query.pos
}
impl_query_pos! {
    QueryPosition for EndState, self => self.query.pos
}
//impl_query_pos! {
//    QueryPosition for IndexRoot, self => self.pos
//}
//impl QueryPosition for SearchPath {
//    fn query_pos(&self) -> &TokenLocation {
//        &self.root.pos
//    }
//    fn query_pos_mut(&mut self) -> &mut TokenLocation {
//        &mut self.root.pos
//    }
//}
impl QueryPosition for TraversalState {
    fn query_pos(&self) -> &TokenLocation {
        match &self.kind {
            InnerKind::Parent(state)
                => &state.query.pos,
            InnerKind::Child(state)
                => &state.paths.query.pos,
            //InnerKind::End(state)
            //    => &state.query.pos,
        }
    }
    fn query_pos_mut(&mut self) -> &mut TokenLocation {
        match &mut self.kind {
            InnerKind::Parent(state)
                => &mut state.query.pos,
            InnerKind::Child(state)
                => &mut state.paths.query.pos,
            //InnerKind::End(state)
            //    => &mut state.query.pos,
        }
    }
}