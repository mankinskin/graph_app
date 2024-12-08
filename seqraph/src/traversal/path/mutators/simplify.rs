use crate::{
    graph::kind::GraphKind,
    direction::r#match::MatchDirection,
    traversal::{
        cache::state::end::{
            EndKind,
            PostfixEnd,
            PrefixEnd,
            RangeEnd,
        },
        path::{
            accessors::{
                border::PathBorder,
                role::{
                    End,
                    PathRole,
                    Start,
                },
                root::GraphRoot,
            },
            structs::{
                match_end::{
                    MatchEnd,
                    MatchEndPath,
                },
                role_path::RolePath,
                rooted_path::{
                    IndexRoot,
                    RootedRolePath,
                },
            },
        },
        result::kind::{
            Primer,
            RoleChildPath,
        },
        traversable::Traversable,
    },
};
use std::borrow::Borrow;
use crate::graph::vertex::pattern::pattern_width;

pub trait PathSimplify: Sized {
    fn into_simplified<Trav: Traversable>(
        self,
        trav: &Trav,
    ) -> Self;
    fn simplify<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) {
        unsafe {
            let old = std::ptr::read(self);
            let new = old.into_simplified(trav);
            std::ptr::write(self, new);
        }
    }
}

impl<R: PathRole> PathSimplify for RolePath<R> {
    fn into_simplified<Trav: Traversable>(
        mut self,
        trav: &Trav,
    ) -> Self {
        let graph = trav.graph();
        while let Some(loc) = self.path_mut().pop() {
            if !<R as PathBorder>::is_at_border(graph.graph(), loc) {
                self.path_mut().push(loc);
                break;
            }
        }
        self
    }
}

impl<P: MatchEndPath> PathSimplify for MatchEnd<P> {
    fn into_simplified<Trav: Traversable>(
        self,
        trav: &Trav,
    ) -> Self {
        if let Some(c) = match self.get_path() {
            Some(p) => {
                if p.single_path().is_empty() && {
                    let location = p.root_child_location();
                    let graph = trav.graph();
                    let pattern = graph.expect_pattern_at(location);
                    <Trav::Kind as GraphKind>::Direction::pattern_index_prev(
                        pattern.borrow(),
                        location.sub_index,
                    )
                    .is_none()
                } {
                    Some(p.root_parent())
                } else {
                    None
                }
            }
            None => None,
        } {
            MatchEnd::Complete(c)
        } else {
            self
        }
    }
}

impl RootedRolePath<Start, IndexRoot> {
    pub fn simplify<Trav: Traversable>(
        mut self,
        trav: &Trav,
    ) -> EndKind {
        self.role_path.simplify(trav);
        match (
            Start::is_at_border(trav.graph(), self.role_root_child_location::<Start>()),
            self.role_path.raw_child_path::<Start>().is_empty(),
        ) {
            (true, true) => EndKind::Complete(self.root_parent()),
            _ => {
                let graph = trav.graph();
                let root = self.role_root_child_location();
                let pattern = graph.expect_pattern_at(root);
                EndKind::Postfix(PostfixEnd {
                    path: self,
                    inner_width: pattern_width(&pattern[root.sub_index + 1..]),
                })
            }
        }
    }
}

impl RangeEnd {
    pub fn simplify<Trav: Traversable>(
        mut self,
        trav: &Trav,
    ) -> EndKind {
        self.path.child_path_mut::<Start>().simplify(trav);
        self.path.child_path_mut::<End>().simplify(trav);

        match (
            Start::is_at_border(trav.graph(), self.path.role_root_child_location::<Start>()),
            self.path.raw_child_path::<Start>().is_empty(),
            End::is_at_border(trav.graph(), self.path.role_root_child_location::<End>()),
            self.path.raw_child_path::<End>().is_empty(),
        ) {
            (true, true, true, true) => EndKind::Complete(self.path.root_parent()),
            (true, true, false, _) | (true, true, true, false) => EndKind::Prefix(PrefixEnd {
                path: self.path.into(),
                target: self.target,
            }),
            (false, _, true, true) | (true, false, true, true) => {
                let graph = trav.graph();
                let path: Primer = self.path.into();
                let root = path.role_root_child_location();
                let pattern = graph.expect_pattern_at(root);
                EndKind::Postfix(PostfixEnd {
                    path,
                    inner_width: pattern_width(&pattern[root.sub_index + 1..]),
                })
            }
            _ => EndKind::Range(self),
        }
    }
}
//impl<R: PathRole> PathSimplify for RolePath<R> {
//    fn into_simplified<
//        Trav: Traversable,
//    >(mut self, trav: &Trav) -> Self {
//        let graph = trav.graph();
//        // remove segments pointing to mismatch at pattern head
//        while let Some(location) = self.path_mut().pop() {
//            let pattern = graph.expect_pattern_at(&location);
//            // skip segments at end of pattern
//            if Trav::Direction::pattern_index_next(pattern.borrow(), location.sub_index).is_some() {
//                self.path_mut().push(location);
//                break;
//            }
//        }
//        self
//    }
//}
//impl<P: PathSimplify> PathSimplify for OriginPath<P> {
//    fn into_simplified<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(mut self, trav: &Trav) -> Self {
//        self.postfix.simplify::<_, D, _>(trav);
//        self
//    }
//}
