use std::{
    borrow::Borrow,
    ops::ControlFlow,
};

use crate::{
    direction::{
        Direction,
        Left,
        Right,
    },
    graph::direction::r#match::MatchDirection,
    traversal::{
        path::mutators::move_path::MoveKey,
        traversable::{
            TravDir,
            Traversable,
        },
    },
    vertex::{
        location::child::ChildLocation,
        wide::Wide,
    },
};

pub struct KeyedLeaf<'k, D: Direction, K: MoveKey<D> + 'k> {
    path: &'k mut K,
    location: &'k mut ChildLocation,
    _ty: std::marker::PhantomData<D>,
}

impl<'k, D: Direction, K: MoveKey<D>> KeyedLeaf<'k, D, K> {
    pub fn new(
        path: &'k mut K,
        location: &'k mut ChildLocation,
    ) -> Self {
        Self {
            path,
            location,
            _ty: Default::default(),
        }
    }
}

pub trait MoveLeaf<D: Direction> {
    fn move_leaf<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()>;
}

pub trait AdvanceLeaf: MoveLeaf<Right> {
    fn advance_leaf<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        self.move_leaf(trav)
    }
}

impl<T: MoveLeaf<Right>> AdvanceLeaf for T {}

pub trait RetractLeaf: MoveLeaf<Left> {
    fn retract_leaf<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        self.move_leaf(trav)
    }
}

impl<T: MoveLeaf<Left>> RetractLeaf for T {}

impl<K: MoveKey<Right, Delta = usize>> MoveLeaf<Right> for KeyedLeaf<'_, Right, K> {
    fn move_leaf<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(&*self.location);
        if let Some(next) =
            TravDir::<Trav>::pattern_index_next(pattern.borrow(), self.location.sub_index)
        {
            let prev = &pattern[self.location.sub_index];
            self.path.move_key(prev.width());
            self.location.sub_index = next;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

impl MoveLeaf<Right> for ChildLocation {
    fn move_leaf<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(&*self);
        if let Some(next) = TravDir::<Trav>::pattern_index_next(pattern.borrow(), self.sub_index) {
            self.sub_index = next;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

impl<K: MoveKey<Left, Delta = usize>> MoveLeaf<Left> for KeyedLeaf<'_, Left, K> {
    fn move_leaf<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(&*self.location);
        if let Some(prev) =
            TravDir::<Trav>::pattern_index_prev(pattern.borrow(), self.location.sub_index)
        {
            let c = &pattern[self.location.sub_index];
            self.path.move_key(c.width());
            self.location.sub_index = prev;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

impl MoveLeaf<Left> for ChildLocation {
    fn move_leaf<Trav: Traversable>(
        &mut self,
        trav: &Trav,
    ) -> ControlFlow<()> {
        let graph = trav.graph();
        let pattern = graph.expect_pattern_at(&*self);
        if let Some(prev) = TravDir::<Trav>::pattern_index_prev(pattern.borrow(), self.sub_index) {
            self.sub_index = prev;
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}
