use crate::{
    path::{
        accessors::role::{
            End,
            PathRole,
            Start,
        },
        structs::{
            match_end::{
                MatchEnd,
                MatchEndPath,
            },
            query_range_path::QueryRangePath,
            role_path::RolePath,
            rooted_path::{
                PathRoot,
                RootedRolePath,
                RootedSplitPath,
                RootedSplitPathRef,
                SearchPath,
                SubPath,
            },
        },
    }, traversal::state::query::QueryState,
};
use auto_impl::auto_impl;

/// access to the position of a child
#[auto_impl(&, & mut)]
pub trait RootChildPos<R> {
    fn root_child_pos(&self) -> usize;
}

impl<R: PathRole> RootChildPos<R> for RolePath<R> {
    fn root_child_pos(&self) -> usize {
        self.sub_path.root_entry
    }
}

impl<R: PathRole, Root: PathRoot> RootChildPos<R> for RootedRolePath<R, Root> {
    fn root_child_pos(&self) -> usize {
        RootChildPos::<R>::root_child_pos(&self.role_path)
    }
}

impl<R: PathRole, Root: PathRoot> RootChildPos<R> for RootedSplitPath<Root> {
    fn root_child_pos(&self) -> usize {
        RootChildPos::<R>::root_child_pos(&self.sub_path)
    }
}

impl<R: PathRole, Root: PathRoot> RootChildPos<R> for RootedSplitPathRef<'_, Root> {
    fn root_child_pos(&self) -> usize {
        RootChildPos::<R>::root_child_pos(self.sub_path)
    }
}

impl<R: PathRole> RootChildPos<R> for SubPath {
    fn root_child_pos(&self) -> usize {
        self.root_entry
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
        self.start.root_entry
    }
}

impl RootChildPos<End> for QueryState {
    fn root_child_pos(&self) -> usize {
        self.path.end.root_entry
    }
}

impl RootChildPos<Start> for QueryState {
    fn root_child_pos(&self) -> usize {
        self.path.start.root_entry
    }
}

impl RootChildPos<End> for QueryRangePath {
    fn root_child_pos(&self) -> usize {
        self.end.root_entry
    }
}

pub trait RootChildPosMut<R>: RootChildPos<R> {
    fn root_child_pos_mut(&mut self) -> &mut usize;
}

impl<R: PathRole> RootChildPosMut<R> for RolePath<R> {
    fn root_child_pos_mut(&mut self) -> &mut usize {
        &mut self.sub_path.root_entry
    }
}

impl<R: PathRole, Root: PathRoot> RootChildPosMut<R> for RootedRolePath<R, Root> {
    fn root_child_pos_mut(&mut self) -> &mut usize {
        self.role_path.root_child_pos_mut()
    }
}

impl RootChildPosMut<End> for QueryRangePath {
    fn root_child_pos_mut(&mut self) -> &mut usize {
        &mut self.end.sub_path.root_entry
    }
}

impl RootChildPosMut<End> for SearchPath {
    fn root_child_pos_mut(&mut self) -> &mut usize {
        self.end.root_child_pos_mut()
    }
}

impl RootChildPosMut<End> for QueryState {
    fn root_child_pos_mut(&mut self) -> &mut usize {
        &mut self.path.end.sub_path.root_entry
    }
}
//impl<R, P: RootChildPos<R>> RootChildPos<R> for OriginPath<P> {
//    fn root_child_pos(&self) -> usize {
//        self.postfix.root_child_pos()
//    }
//}
//impl RootChildPos<Start> for PatternPrefixPath {
//    fn root_child_pos(&self) -> usize {
//        0
//    }
//}
//impl RootChildPos<End> for PatternPrefixPath {
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
//impl RootChildPosMut<End> for OverlapPrimer {
//    fn root_child_pos_mut(&mut self) -> &mut usize {
//        &mut self.exit
//    }
//}
//impl RootChildPosMut<End> for PatternPrefixPath {
//    fn root_child_pos_mut(&mut self) -> &mut usize {
//        &mut self.exit
//    }
//}
//impl<P: RootChildPosMut<End>> RootChildPosMut<End> for OriginPath<P> {
//    fn root_child_pos_mut(&mut self) -> &mut usize {
//        self.postfix.root_child_pos_mut()
//    }
//}
