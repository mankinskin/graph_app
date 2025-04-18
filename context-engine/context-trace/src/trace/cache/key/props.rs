use crate::graph::vertex::location::child::ChildLocation;

pub trait LeafKey {
    fn leaf_location(&self) -> ChildLocation;
}
use crate::path::mutators::move_path::key::TokenPosition;

use super::directed::{
    up::UpKey,
    DirectedKey,
};

/// get the token position in a query
pub trait CursorPosition {
    fn cursor_pos(&self) -> &TokenPosition;
    fn cursor_pos_mut(&mut self) -> &mut TokenPosition;
}
#[macro_export]
macro_rules! impl_cursor_pos {
    {
        $(< $( $par:ident $( : $bhead:tt $( + $btail:tt )*)? ),* >)? CursorPosition for $target:ty, $self_:ident => $func:expr
    } => {
        impl <$( $( $par $(: $bhead $( + $btail )* )? ),* )?> $crate::trace::cache::key::props::CursorPosition for $target {
            fn cursor_pos(& $self_) -> &$crate::path::mutators::move_path::key::TokenPosition {
                &$func
            }
            fn cursor_pos_mut(&mut $self_) -> &mut $crate::path::mutators::move_path::key::TokenPosition {
                &mut $func
            }
        }
    };
}
pub trait RootKey {
    fn root_key(&self) -> UpKey;
}

pub trait TargetKey {
    fn target_key(&self) -> DirectedKey;
}
