use std::ops::ControlFlow;

use super::{
    RootedRangePath,
    role_path::{
        IndexRolePath,
        RootedRolePath,
    },
    root::{
        IndexRoot,
        PathRoot,
        RootedPath,
    },
};
use crate::{
    direction::{
        Direction,
        Left,
        Right,
        pattern::PatternDirection,
    },
    graph::vertex::{
        child::Child,
        location::{
            child::ChildLocation,
            pattern::IntoPatternLocation,
        },
        wide::Wide,
    },
    impl_root,
    path::{
        accessors::{
            child::{
                PathChild,
                RootChildIndex,
                RootChildIndexMut,
                root::{
                    GraphRootChild,
                    RootChild,
                },
            },
            has_path::{
                HasMatchPaths,
                HasPath,
                HasRolePath,
            },
            role::{
                End,
                PathRole,
                Start,
            },
            root::{
                GraphRootPattern,
                RootPattern,
            },
        },
        mutators::{
            append::PathAppend,
            lower::PathLower,
            move_path::{
                key::{
                    RetractKey,
                    TokenPosition,
                },
                leaf::MoveLeaf,
                path::MovePath,
                root::MoveRootIndex,
            },
        },
        structs::{
            query_range_path::RangePath,
            role_path::RolePath,
            sub_path::SubPath,
        },
    },
    trace::{
        cache::key::props::LeafKey,
        has_graph::{
            HasGraph,
            TravDir,
        },
    },
};

pub type IndexRangePath = RootedRangePath<IndexRoot>;

impl RangePath for IndexRangePath {
    fn new_range(
        root: Self::Root,
        entry: usize,
        exit: usize,
    ) -> Self {
        Self {
            root,
            start: SubPath::new(entry).into(),
            end: SubPath::new(exit).into(),
        }
    }
}
impl_root! { GraphRootPattern for IndexRangePath, self => self.root.location.clone() }
impl_root! { GraphRoot for IndexRangePath, self => self.root_pattern_location().parent }
impl_root! { RootPattern for IndexRangePath, self, trav => GraphRootPattern::graph_root_pattern::<G>(self, trav) }

impl<R: PathRole + 'static> HasPath<R> for IndexRangePath
where
    Self: HasRolePath<R>,
{
    fn path(&self) -> &Vec<ChildLocation> {
        HasRolePath::<R>::role_path(self).path()
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        HasRolePath::<R>::role_path_mut(self).path_mut()
    }
}
impl<R: PathRole + 'static> HasPath<R> for IndexRolePath<R>
where
    Self: HasRolePath<R>,
{
    fn path(&self) -> &Vec<ChildLocation> {
        HasRolePath::<R>::role_path(self).path()
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        HasRolePath::<R>::role_path_mut(self).path_mut()
    }
}

impl HasRolePath<Start> for IndexRangePath {
    fn role_path(&self) -> &RolePath<Start> {
        &self.start
    }
    fn role_path_mut(&mut self) -> &mut RolePath<Start> {
        &mut self.start
    }
}

impl<R: PathRole> PathChild<R> for IndexRangePath
where
    IndexRangePath: HasRolePath<R>,
{
    fn path_child_location(&self) -> Option<ChildLocation> {
        Some(
            R::bottom_up_iter(self.path().iter())
                .next()
                .cloned()
                .unwrap_or(
                    self.root
                        .location
                        .to_child_location(self.role_path().root_entry),
                ),
        )
    }
    fn path_child<G: HasGraph>(
        &self,
        trav: &G,
    ) -> Option<Child> {
        PathChild::<R>::path_child(self.role_path(), trav)
    }
}

impl HasRolePath<End> for IndexRangePath {
    fn role_path(&self) -> &RolePath<End> {
        &self.end
    }
    fn role_path_mut(&mut self) -> &mut RolePath<End> {
        &mut self.end
    }
}

impl HasMatchPaths for IndexRangePath {
    fn into_paths(self) -> (RolePath<Start>, RolePath<End>) {
        (self.start, self.end)
    }
}

impl MoveRootIndex<Right, End> for IndexRangePath {
    fn move_root_index<G: HasGraph>(
        &mut self,
        trav: &G,
    ) -> ControlFlow<()> {
        let graph = trav.graph();
        let pattern = self.root_pattern::<G>(&graph);
        if let Some(next) = TravDir::<G>::pattern_index_next(
            pattern,
            RootChildIndex::<End>::root_child_index(self),
        ) {
            *self.root_child_index_mut() = next;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

impl MoveRootIndex<Left, End> for IndexRangePath {
    fn move_root_index<G: HasGraph>(
        &mut self,
        trav: &G,
    ) -> ControlFlow<()> {
        let graph = trav.graph();
        let pattern = self.root_pattern::<G>(&graph);
        if let Some(prev) = TravDir::<G>::pattern_index_prev(
            pattern,
            RootChildIndex::<End>::root_child_index(self),
        ) {
            *self.root_child_index_mut() = prev;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}
impl<D: Direction, Root: PathRoot> MovePath<D, End> for RootedRangePath<Root>
where
    Self: MoveRootIndex<D> + PathAppend,
    ChildLocation: MoveLeaf<D>,
{
    fn move_path_segment<G: HasGraph>(
        &mut self,
        location: &mut ChildLocation,
        trav: &G::Guard<'_>,
    ) -> ControlFlow<()> {
        location.move_leaf(trav)
    }
}

impl<D: Direction, Root: PathRoot> MovePath<D, End>
    for RootedRolePath<End, Root>
where
    Self: MoveRootIndex<D> + PathAppend,
    ChildLocation: MoveLeaf<D>,
{
    fn move_path_segment<G: HasGraph>(
        &mut self,
        location: &mut ChildLocation,
        trav: &G::Guard<'_>,
    ) -> ControlFlow<()> {
        location.move_leaf(trav)
    }
}
impl RootChild<Start> for IndexRangePath {
    fn root_child<G: HasGraph>(
        &self,
        trav: &G,
    ) -> Child {
        trav.graph()
            .expect_child_at(
                self.path_root()
                    .location
                    .to_child_location(self.start.sub_path.root_entry),
            )
            .clone()
    }
}

impl RootChild<End> for IndexRangePath {
    fn root_child<G: HasGraph>(
        &self,
        trav: &G,
    ) -> Child {
        trav.graph()
            .expect_child_at(
                self.path_root()
                    .location
                    .to_child_location(self.end.sub_path.root_entry),
            )
            .clone()
    }
}
impl GraphRootChild<Start> for IndexRangePath {
    fn root_child_location(&self) -> ChildLocation {
        self.root.location.to_child_location(self.start.root_entry)
    }
}

impl LeafKey for IndexRangePath {
    fn leaf_location(&self) -> ChildLocation {
        self.end.path.last().cloned().unwrap_or(
            self.root
                .location
                .to_child_location(self.end.sub_path.root_entry),
        )
    }
}

impl GraphRootChild<End> for IndexRangePath {
    fn root_child_location(&self) -> ChildLocation {
        self.root.location.to_child_location(self.end.root_entry)
    }
}

impl RootChildIndex<Start> for IndexRangePath {
    fn root_child_index(&self) -> usize {
        RootChildIndex::<Start>::root_child_index(&self.start)
    }
}

impl RootChildIndex<End> for IndexRangePath {
    fn root_child_index(&self) -> usize {
        RootChildIndex::<End>::root_child_index(&self.end)
    }
}

impl RootChildIndexMut<End> for IndexRangePath {
    fn root_child_index_mut(&mut self) -> &mut usize {
        self.end.root_child_index_mut()
    }
}

impl PathLower for (&mut TokenPosition, &mut IndexRangePath) {
    fn path_lower<G: HasGraph>(
        &mut self,
        trav: &G,
    ) -> ControlFlow<()> {
        let (root_pos, range) = self;
        let (start, end, root) = (
            &mut range.start.sub_path,
            &mut range.end.sub_path,
            &mut range.root,
        );
        if let Some(prev) = start.path.pop() {
            let graph = trav.graph();
            let pattern = graph.expect_pattern_at(prev.clone());
            root_pos.retract_key(
                pattern[prev.sub_index + 1..]
                    .iter()
                    .fold(0, |a, c| a + c.width()),
            );
            start.root_entry = prev.sub_index;
            end.root_entry = pattern.len() - 1;
            end.path.clear();
            root.location = prev.into_pattern_location();

            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}
