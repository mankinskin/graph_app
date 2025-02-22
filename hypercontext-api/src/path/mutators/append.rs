use super::super::{
    accessors::role::{
        End,
        PathRole,
    },
    structs::role_path::RolePath,
};
use crate::{
    graph::vertex::location::child::ChildLocation,
    path::structs::{
        query_range_path::FoldablePath,
        rooted::{
            index_range::IndexRangePath,
            pattern_range::PatternRangePath,
            role_path::RootedRolePath,
            root::PathRoot,
        },
        sub_path::SubPath,
    },
    traversal::state::cursor::PathCursor, //traversal::state::query::QueryState,
};

/// move path leaf position one level deeper
pub trait PathAppend {
    fn path_append(
        &mut self,
        parent_entry: ChildLocation,
    );
}

impl<Role: PathRole, Root: PathRoot> PathAppend for RootedRolePath<Role, Root> {
    fn path_append(
        &mut self,
        parent_entry: ChildLocation,
    ) {
        self.role_path.sub_path.path_append(parent_entry);
    }
}

impl PathAppend for SubPath {
    fn path_append(
        &mut self,
        parent_entry: ChildLocation,
    ) {
        self.path.push(parent_entry)
    }
}

impl PathAppend for RolePath<End> {
    fn path_append(
        &mut self,
        parent_entry: ChildLocation,
    ) {
        self.sub_path.path.push(parent_entry)
    }
}

impl PathAppend for IndexRangePath {
    fn path_append(
        &mut self,
        parent_entry: ChildLocation,
    ) {
        self.end.sub_path.path.push(parent_entry);
    }
}

impl PathAppend for PatternRangePath {
    fn path_append(
        &mut self,
        parent_entry: ChildLocation,
    ) {
        self.end.sub_path.path.push(parent_entry);
    }
}
impl<P: FoldablePath> PathAppend for PathCursor<P> {
    fn path_append(
        &mut self,
        parent_entry: ChildLocation,
    ) {
        self.path.path_append(parent_entry);
    }
}

//impl PathAppend for QueryState {
//    fn path_append(
//        &mut self,
//        parent_entry: ChildLocation,
//    ) {
//        self.path.end.path_append(parent_entry)
//    }
//}
//impl PathAppend for SubPath {
//    fn path_append<
//        Trav: Traversable,
//    >(
//        &mut self,
//        trav: &Trav,
//        parent_entry: ChildLocation,
//    ) {
//        self.path.push(parent_entry);
//    }
//}
//impl PathAppend for RolePath<Start> {
//    fn path_append<
//        T: Tokenize,
//        Trav: Traversable<T>,
//    >(
//        &mut self,
//        trav: &Trav,
//        parent_entry: ChildLocation,
//    ) {
//        self.path.path_append(trav, parent_entry)
//    }
//}
//impl<P: MatchEndPath + PathAppend> PathAppend for MatchEnd<P> {
//    fn path_append(&mut self, parent_entry: ChildLocation) {
//        match self {
//            MatchEnd::Path(path) => path.path_append(parent_entry),
//            MatchEnd::Complete(child) => RolePath {
//                path: vec![parent_entry],
//                width: child.width(),
//                child,
//                token_pos: 0,
//                _ty: Default::default(),
//            }.into(),
//        }
//    }
//}
//impl<P: PathAppend> PathAppend for OriginPath<P>
//    where <P as PathAppend>::Result: PathAppend<Result=<P as PathAppend>::Result> + GraphRootChild<Start>
//{
//    type Result = OriginPath<<P as PathAppend>::Result>;
//    fn path_append(self, parent_entry: ChildLocation) -> Self::Result {
//        OriginPath {
//            origin: MatchEnd::Path(self.origin.path_append(parent_entry)),
//            postfix: self.postfix.path_append(parent_entry),
//        }
//    }
//}
//impl PathAppend for PathLeaf {
//    type Result = RolePath;
//    fn path_append<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: ,
//        Trav: Traversable<T>
//    >(self, trav: &'a Trav, parent_entry: ChildLocation) -> Self::Result {
//        let graph = trav.graph();
//        let pattern = graph.expect_pattern_at(self.entry);
//        if self.entry.sub_index == D::head_index(pattern.borrow()) {
//            RolePath::Leaf(PathLeaf {
//                entry: parent_entry,
//                child: self.entry.parent,
//                width: self.width,
//                token_pos: self.token_pos,
//            })
//        } else {
//            RolePath::Path {
//                entry: parent_entry,
//                path: vec![self.entry],
//                width: self.width,
//                child: self.child,
//                token_pos: self.token_pos,
//            }
//        }
//    }
//}

//impl<R: PathRole> PathAppend for RolePath<R> {
//    type Result = Self;
//    fn path_append<
//        T: Tokenize,
//        D: ,
//        Trav: Traversable<T>
//    >(mut self, trav: &Trav, parent_entry: ChildLocation) -> Self::Result {
//        //println!("path {} -> {}, {}", entry.parent.index, parent_entry.parent.index, width);
//        let entry = self.child_location();
//        let graph = trav.graph();
//        let pattern = self.graph_root_pattern::<_, Trav>(&graph);
//        // start paths only at a non-head index position
//        if entry.sub_index != D::head_index(pattern.borrow()) {
//            self.path.push(parent_entry);
//        } else {
//            self.path = vec![parent_entry]
//        }
//        self
//    }
//}
