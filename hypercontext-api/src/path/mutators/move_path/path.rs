use crate::{
    direction::{
        Direction,
        Left,
        Right,
    },
    traversal::{
        state::query::QueryState, traversable::Traversable
    },
};
use super::super::super::{
    accessors::role::{
        End,
        PathRole,
    },
    mutators::{
        append::PathAppend,
        move_path::{
            leaf::{
                AdvanceLeaf,
                KeyedLeaf,
                RetractLeaf,
            },
            root::MoveRootPos,
        },
        pop::PathPop,
    },
    structs::{
        query_range_path::PatternPrefixPath,
        rooted_path::SearchPath,
    },
};
use std::ops::ControlFlow;
use crate::graph::vertex::location::child::ChildLocation;

pub trait MovePath<D: Direction, R: PathRole = End>:
    PathPop + PathAppend + MoveRootPos<D, R>
{
    fn move_leaf<Trav: Traversable>(
        &mut self,
        location: &mut ChildLocation,
        trav: &Trav::Guard<'_>,
    ) -> ControlFlow<()>;

    fn move_path<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        let graph = trav.graph();
        if let Some(location) = std::iter::from_fn(|| {
            self.path_pop().map(|mut location| {
                self.move_leaf::<Trav>(&mut location, &graph)
                    .is_continue()
                    .then_some(location)
            })
        })
        .find_map(|location| location)
        {
            self.path_append(location);
            ControlFlow::Continue(())
        } else {
            self.move_root_pos(trav)
        }
    }
}

impl MovePath<Right, End> for QueryState {
    fn move_leaf<Trav: Traversable>(
        &mut self,
        location: &mut ChildLocation,
        trav: &Trav::Guard<'_>,
    ) -> ControlFlow<()> {
        KeyedLeaf::new(self, location).advance_leaf(trav)
    }
}

impl MovePath<Left, End> for QueryState {
    fn move_leaf<Trav: Traversable>(
        &mut self,
        location: &mut ChildLocation,
        trav: &Trav::Guard<'_>,
    ) -> ControlFlow<()> {
        KeyedLeaf::new(self, location).retract_leaf(trav)
    }
}

impl MovePath<Right, End> for SearchPath {
    fn move_leaf<Trav: Traversable>(
        &mut self,
        location: &mut ChildLocation,
        trav: &Trav::Guard<'_>,
    ) -> ControlFlow<()> {
        location.advance_leaf(trav)
    }
}

impl MovePath<Left, End> for SearchPath {
    fn move_leaf<Trav: Traversable>(
        &mut self,
        location: &mut ChildLocation,
        trav: &Trav::Guard<'_>,
    ) -> ControlFlow<()> {
        location.retract_leaf(trav)
    }
}

impl MovePath<Right, End> for PatternPrefixPath {
    fn move_leaf<Trav: Traversable>(
        &mut self,
        location: &mut ChildLocation,
        trav: &Trav::Guard<'_>,
    ) -> ControlFlow<()> {
        location.advance_leaf(trav)
    }
}