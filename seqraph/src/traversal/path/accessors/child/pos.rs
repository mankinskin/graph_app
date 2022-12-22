use crate::*;

/// access to the position of a child
pub trait ChildPos<R> {
    fn child_pos(&self) -> usize;
}
impl<R> ChildPos<R> for ChildPath {
    fn child_pos(&self) -> usize {
        self.child_location().sub_index
    }
}
impl ChildPos<Start> for SearchPath {
    fn child_pos(&self) -> usize {
        ChildPos::<Start>::child_pos(&self.start)
    }
}
impl ChildPos<End> for SearchPath {
    fn child_pos(&self) -> usize {
        ChildPos::<End>::child_pos(&self.end)
    }
}
impl<R, P: ChildPos<R>> ChildPos<R> for OriginPath<P> {
    fn child_pos(&self) -> usize {
        self.postfix.child_pos()
    }
}
impl ChildPos<Start> for QueryRangePath {
    fn child_pos(&self) -> usize {
        self.entry
    }
}
impl ChildPos<End> for QueryRangePath {
    fn child_pos(&self) -> usize {
        self.exit
    }
}
impl ChildPos<Start> for PrefixQuery {
    fn child_pos(&self) -> usize {
        0
    }
}
impl ChildPos<End> for PrefixQuery {
    fn child_pos(&self) -> usize {
        self.exit
    }
}
impl<R> ChildPos<R> for PathLeaf {
    fn child_pos(&self) -> usize {
        self.entry.sub_index
    }
}
impl ChildPos<Start> for OverlapPrimer {
    fn child_pos(&self) -> usize {
        0
    }
}
impl ChildPos<End> for OverlapPrimer {
    fn child_pos(&self) -> usize {
        self.exit
    }
}
pub trait ChildPosMut<R>: ChildPos<R> {
    fn child_pos_mut(&mut self) -> &mut usize;
}
impl ChildPosMut<End> for ChildPath {
    fn child_pos_mut(&mut self) -> &mut usize {
        &mut self.child_location_mut().sub_index
    }
}
impl ChildPosMut<End> for OverlapPrimer {
    fn child_pos_mut(&mut self) -> &mut usize {
        &mut self.exit
    }
}
impl ChildPosMut<End> for QueryRangePath {
    fn child_pos_mut(&mut self) -> &mut usize {
        &mut self.exit
    }
}
impl ChildPosMut<End> for PrefixQuery {
    fn child_pos_mut(&mut self) -> &mut usize {
        &mut self.exit
    }
}
impl ChildPosMut<End> for SearchPath {
    fn child_pos_mut(&mut self) -> &mut usize {
        self.end.child_pos_mut()
    }
}
impl<P: ChildPosMut<End>> ChildPosMut<End> for OriginPath<P> {
    fn child_pos_mut(&mut self) -> &mut usize {
        self.postfix.child_pos_mut()
    }
}