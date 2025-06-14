use crate::traversal::ControlFlow;
use context_trace::{
    direction::Direction,
    graph::vertex::{
        child::Child,
        location::child::ChildLocation,
        wide::Wide,
    },
    impl_cursor_pos,
    path::{
        accessors::{
            child::{
                root::RootChild,
                PathChild,
                RootChildIndex,
            },
            has_path::HasPath,
            role::PathRole,
            root::RootPattern,
        },
        mutators::{
            append::PathAppend,
            move_path::{
                key::{
                    MoveKey,
                    TokenPosition,
                },
                path::MovePath,
                root::MoveRootIndex,
            },
            pop::PathPop,
        },
        structs::{
            query_range_path::FoldablePath,
            rooted::pattern_range::{
                PatternPostfixPath,
                PatternRangePath,
            },
        },
        RolePathUtils,
    },
    trace::has_graph::HasGraph,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathCursor<P> {
    pub path: P,
    /// position relative to start of path
    pub relative_pos: TokenPosition,
}

impl<P> Wide for PathCursor<P> {
    fn width(&self) -> usize {
        self.relative_pos.into()
    }
}
impl<P: PathPop> PathPop for PathCursor<P> {
    fn path_pop(&mut self) -> Option<ChildLocation> {
        self.path.path_pop()
    }
}
impl<P: PathAppend> PathAppend for PathCursor<P> {
    fn path_append(
        &mut self,
        parent_entry: ChildLocation,
    ) {
        self.path.path_append(parent_entry);
    }
}
impl<R: PathRole, P: RootChild<R> + FoldablePath> RootChild<R>
    for PathCursor<P>
{
    fn root_child<G: HasGraph>(
        &self,
        trav: &G,
    ) -> Child {
        self.path.root_child(trav)
    }
}
impl<R: PathRole, P: FoldablePath + HasPath<R>> HasPath<R> for PathCursor<P> {
    fn path(&self) -> &Vec<ChildLocation> {
        self.path.path()
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        self.path.path_mut()
    }
}

impl<R: PathRole, P: FoldablePath + PathChild<R>> PathChild<R>
    for PathCursor<P>
{
    fn path_child_location(&self) -> Option<ChildLocation> {
        self.path.path_child_location()
    }
    fn path_child<G: HasGraph>(
        &self,
        trav: &G,
    ) -> Option<Child> {
        self.path.path_child(trav)
    }
}
impl<R: PathRole, P: RootChildIndex<R>> RootChildIndex<R> for PathCursor<P> {
    fn root_child_index(&self) -> usize {
        RootChildIndex::<R>::root_child_index(&self.path)
    }
}
impl_cursor_pos! {
    <R: FoldablePath> CursorPosition for PathCursor<R>, self => self.relative_pos
}

pub type PatternRangeCursor = PathCursor<PatternRangePath>;
pub type PatternCursor = PathCursor<PatternPostfixPath>;

impl From<PatternRangeCursor> for PatternCursor {
    fn from(value: PathCursor<PatternRangePath>) -> Self {
        Self {
            path: value.path.into(),
            relative_pos: value.relative_pos,
        }
    }
}

impl<D: Direction, P> MoveKey<D> for PathCursor<P>
where
    TokenPosition: MoveKey<D>,
{
    fn move_key(
        &mut self,
        delta: usize,
    ) {
        self.relative_pos.move_key(delta)
    }
}

pub trait MovablePath<D: Direction, R: PathRole>:
    MovePath<D, R> + RootChildIndex<R> + RootPattern
{
}
impl<
        D: Direction,
        R: PathRole,
        P: MovePath<D, R> + RootChildIndex<R> + RootPattern,
    > MovablePath<D, R> for P
{
}
impl<D: Direction, R: PathRole, P: MovablePath<D, R>> MovePath<D, R>
    for PathCursor<P>
where
    Self: MoveKey<D>,
{
    fn move_path_segment<G: HasGraph>(
        &mut self,
        location: &mut ChildLocation,
        trav: &G::Guard<'_>,
    ) -> ControlFlow<()> {
        let flow = self.path.move_path_segment::<G>(location, trav);
        if let ControlFlow::Continue(()) = flow {
            let graph = trav.graph();
            self.move_key(graph.expect_child_at(location.clone()).width());
        }
        flow
    }
}

impl<D: Direction, R: PathRole, P: MovablePath<D, R>> MoveRootIndex<D, R>
    for PathCursor<P>
where
    Self: MoveKey<D> + RootChildIndex<R>,
{
    fn move_root_index<G: HasGraph>(
        &mut self,
        trav: &G,
    ) -> ControlFlow<()> {
        let flow = self.path.move_root_index(trav);
        if let ControlFlow::Continue(()) = flow {
            let graph = trav.graph();
            let pattern = self.path.root_pattern::<G>(&graph);
            self.move_key(pattern[self.role_root_child_index()].width());
        }
        flow
    }
}
pub trait ToCursor: FoldablePath {
    fn to_cursor<G: HasGraph>(
        self,
        trav: &G,
    ) -> PathCursor<Self>;
}
impl<P: FoldablePath> ToCursor for P {
    fn to_cursor<G: HasGraph>(
        self,
        trav: &G,
    ) -> PathCursor<Self> {
        PathCursor {
            relative_pos: self.calc_width(trav).into(),
            path: self,
        }
    }
}
