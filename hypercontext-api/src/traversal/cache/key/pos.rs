use crate::{
    path::{
        mutators::move_path::key::TokenPosition,
        structs::pair::PathPair,
    },
    traversal::state::{
        child::ChildState,
        cursor::RangeCursor,
        end::EndState,
        parent::ParentState,
        start::StartState,
        traversal::TraversalState,
        InnerKind,
    },
};

/// get the token position in a query
pub trait CursorPosition {
    fn cursor_pos(&self) -> &TokenPosition;
    fn cursor_pos_mut(&mut self) -> &mut TokenPosition;
}
macro_rules! impl_cursor_pos {
    {
        CursorPosition for $target:ty, $self_:ident => $func:expr
    } => {
        impl CursorPosition for $target {
            fn cursor_pos(& $self_) -> &TokenPosition {
                &$func
            }
            fn cursor_pos_mut(&mut $self_) -> &mut TokenPosition {
                &mut $func
            }
        }
    };
}
impl_cursor_pos! {
    CursorPosition for RangeCursor, self => self.relative_pos
}
impl_cursor_pos! {
    CursorPosition for PathPair, self => self.cursor.relative_pos
}
impl_cursor_pos! {
    CursorPosition for ParentState, self => self.cursor.relative_pos
}
impl_cursor_pos! {
    CursorPosition for ChildState, self => self.paths.cursor.relative_pos
}
impl_cursor_pos! {
    CursorPosition for StartState, self => self.cursor.relative_pos
}
impl_cursor_pos! {
    CursorPosition for EndState, self => self.cursor.relative_pos
}
//impl_cursor_pos! {
//    CursorPosition for IndexRoot, self => self.pos
//}
//impl CursorPosition for IndexRangePath {
//    fn cursor_pos(&self) -> &TokenLocation {
//        &self.root.pos
//    }
//    fn cursor_pos_mut(&mut self) -> &mut TokenLocation {
//        &mut self.root.pos
//    }
//}
impl CursorPosition for TraversalState {
    fn cursor_pos(&self) -> &TokenPosition {
        match &self.kind {
            InnerKind::Parent(state) => state.cursor_pos(),
            InnerKind::Child(state) => state.cursor_pos(),
            //InnerKind::End(state)
            //    => &state.query.pos,
        }
    }
    fn cursor_pos_mut(&mut self) -> &mut TokenPosition {
        match &mut self.kind {
            InnerKind::Parent(state) => state.cursor_pos_mut(),
            InnerKind::Child(state) => state.cursor_pos_mut(),
            //InnerKind::End(state)
            //    => &mut state.query.pos,
        }
    }
}
