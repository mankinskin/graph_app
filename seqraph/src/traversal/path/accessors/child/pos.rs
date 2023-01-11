use crate::*;

/// access to the position of a child
pub trait RootChildPos<R> {
    fn root_child_pos(&self) -> usize;
}
impl<R: PathRole> RootChildPos<R> for ChildPath<R> {
    fn root_child_pos(&self) -> usize {
        <Self as GraphRootChild<R>>::root_child_location(self).sub_index
    }
}
impl RootChildPos<Start> for SearchPath {
    fn root_child_pos(&self) -> usize {
        RootChildPos::<Start>::root_child_pos(&self.start)
    }
}
impl RootChildPos<End> for SearchPath {
    fn root_child_pos(&self) -> usize {
        RootChildPos::<End>::root_child_pos(&self.end)
    }
}
impl<R, P: RootChildPos<R>> RootChildPos<R> for OriginPath<P> {
    fn root_child_pos(&self) -> usize {
        self.postfix.root_child_pos()
    }
}
impl<P: MatchEndPath> RootChildPos<Start> for MatchEnd<P> {
    fn root_child_pos(&self) -> usize {
        match self {
            Self::Complete(_) => 0,
            Self::Path(path) => path.root_child_pos(),
        }
    }
}
impl RootChildPos<Start> for QueryRangePath {
    fn root_child_pos(&self) -> usize {
        self.entry
    }
}
impl RootChildPos<End> for QueryRangePath {
    fn root_child_pos(&self) -> usize {
        self.exit
    }
}
//impl RootChildPos<Start> for PrefixQuery {
//    fn root_child_pos(&self) -> usize {
//        0
//    }
//}
//impl RootChildPos<End> for PrefixQuery {
//    fn root_child_pos(&self) -> usize {
//        self.exit
//    }
//}

//impl<R> RootChildPos<R> for PathLeaf {
//    fn root_child_pos(&self) -> usize {
//        self.entry.sub_index
//    }
//}
//impl RootChildPos<Start> for OverlapPrimer {
//    fn root_child_pos(&self) -> usize {
//        0
//    }
//}
//impl RootChildPos<End> for OverlapPrimer {
//    fn root_child_pos(&self) -> usize {
//        self.exit
//    }
//}
pub trait RootChildPosMut<R>: RootChildPos<R> {
    fn root_child_pos_mut(&mut self) -> &mut usize;
}
impl RootChildPosMut<End> for ChildPath<End> {
    fn root_child_pos_mut(&mut self) -> &mut usize {
        &mut self.child_location_mut().sub_index
    }
}
impl RootChildPosMut<End> for QueryRangePath {
    fn root_child_pos_mut(&mut self) -> &mut usize {
        &mut self.exit
    }
}
//impl RootChildPosMut<End> for OverlapPrimer {
//    fn root_child_pos_mut(&mut self) -> &mut usize {
//        &mut self.exit
//    }
//}
//impl RootChildPosMut<End> for PrefixQuery {
//    fn root_child_pos_mut(&mut self) -> &mut usize {
//        &mut self.exit
//    }
//}
impl RootChildPosMut<End> for SearchPath {
    fn root_child_pos_mut(&mut self) -> &mut usize {
        self.end.root_child_pos_mut()
    }
}
impl<P: RootChildPosMut<End>> RootChildPosMut<End> for OriginPath<P> {
    fn root_child_pos_mut(&mut self) -> &mut usize {
        self.postfix.root_child_pos_mut()
    }
}