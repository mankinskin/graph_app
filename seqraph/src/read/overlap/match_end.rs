use std::borrow::Borrow;

use hypercontext_api::{
    direction::pattern::PatternDirection,
    graph::{
        kind::GraphKind,
        vertex::child::Child,
    },
    path::{
        accessors::{
            child::{
                root::{
                    GraphRootChild,
                    RootChild,
                },
                RootChildPos,
            },
            complete::PathComplete,
            has_path::HasSinglePath,
            role::Start,
            root::GraphRoot,
        },
        mutators::simplify::PathSimplify,
        structs::rooted::{
            role_path::RootedRolePath,
            root::IndexRoot,
        },
        BasePath,
    },
    traversal::{
        iterator::policy::NodePath,
        traversable::Traversable,
    },
};

pub trait MatchEndPath:
    NodePath<Start>
    + Into<RootedRolePath<Start, IndexRoot>>
    + From<RootedRolePath<Start, IndexRoot>>
    + HasSinglePath
    + GraphRootChild<Start>
    + BasePath
{
}

impl<
        T: NodePath<Start>
            + Into<RootedRolePath<Start, IndexRoot>>
            + From<RootedRolePath<Start, IndexRoot>>
            + HasSinglePath
            + GraphRootChild<Start>
            + BasePath,
    > MatchEndPath for T
{
}

/// Used to represent results after traversal with only a start path
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatchEnd<P: MatchEndPath> {
    Path(P),
    Complete(Child),
}
impl<P: MatchEndPath + GraphRoot> GraphRoot for MatchEnd<P> {
    fn root_parent(&self) -> Child {
        match self {
            Self::Complete(c) => *c,
            Self::Path(path) => path.root_parent(),
        }
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
impl<P: MatchEndPath> PathComplete for MatchEnd<P> {
    fn as_complete(&self) -> Option<Child> {
        match self {
            Self::Complete(c) => Some(*c),
            _ => None,
        }
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

pub trait IntoMatchEndStartPath {
    fn into_mesp(self) -> MatchEnd<RootedRolePath<Start, IndexRoot>>;
}

impl<P: MatchEndPath> IntoMatchEndStartPath for MatchEnd<P> {
    fn into_mesp(self) -> MatchEnd<RootedRolePath<Start, IndexRoot>> {
        match self {
            MatchEnd::Path(p) => MatchEnd::Path(p.into()),
            MatchEnd::Complete(c) => MatchEnd::Complete(c),
        }
    }
}

//impl<P: MatchEndPath> IntoMatchEndStartPath for OriginPath<MatchEnd<P>> {
//    fn into_mesp(self) -> MatchEnd<RolePath<Start>> {
//        self.postfix.into_mesp()
//    }
//}
//impl From<OriginPath<MatchEnd<RolePath<Start>>>> for MatchEnd<RolePath<Start>> {
//    fn from(start: OriginPath<MatchEnd<RolePath<Start>>>) -> Self {
//        start.postfix
//    }
//}
impl<P: MatchEndPath + From<Q>, Q: Into<RootedRolePath<Start, IndexRoot>>> From<Q> for MatchEnd<P> {
    fn from(start: Q) -> Self {
        // todo: handle complete
        MatchEnd::Path(P::from(start))
    }
}

impl<P: MatchEndPath> RootChild<Start> for MatchEnd<P> {
    fn root_child<Trav: Traversable>(
        &self,
        trav: &Trav,
    ) -> Child {
        match self {
            Self::Complete(c) => *c,
            Self::Path(path) => path.root_child(trav),
        }
    }
}

impl<P: MatchEndPath> MatchEnd<P> {
    #[allow(unused)]
    pub fn unwrap_path(self) -> P {
        match self {
            Self::Path(path) => Some(path),
            _ => None,
        }
        .unwrap()
    }
    pub fn get_path(&self) -> Option<&P> {
        match self {
            Self::Path(start) => Some(start),
            _ => None,
        }
    }
    //pub fn into_result(self, start: RolePath) -> R::Result<P> {
    //    match self {
    //        Self::Path(start) => Some(start),
    //        _ => None,
    //    }
    //}
}
