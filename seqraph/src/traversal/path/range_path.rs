use std::borrow::Borrow;

use crate::{
    vertex::*,
    *,
};
pub trait EntryPos {
    fn get_entry_pos(&self) -> usize;
}
pub trait ExitPos {
    fn get_exit_pos(&self) -> usize;
}
pub trait PatternEntry: EntryPos {
    fn get_entry_pattern(&self) -> &[Child];
    fn get_entry(&self) -> Child {
        self.get_entry_pattern()[self.get_entry_pos()]
    }
}
pub trait PatternExit: ExitPos {
    fn get_exit_pattern(&self) -> &[Child];
    fn get_exit(&self) -> Child {
        self.get_exit_pattern()[self.get_exit_pos()]
    }
}
pub trait GraphEntry: EntryPos {
    fn get_entry_location(&self) -> ChildLocation;
    fn get_entry_pattern<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Pattern {
        trav.graph().expect_pattern_at(self.get_entry_location())
    }
    fn get_entry<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_entry_location())
    }
}
pub trait GraphExit: ExitPos {
    fn get_exit_location(&self) -> ChildLocation;
    fn get_exit_pattern<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Pattern {
        trav.graph().expect_pattern_at(self.get_exit_location())
    }
    fn get_exit<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_exit_location())
    }
}
pub trait HasStartPath {
    fn get_start_path(&self) -> &[ChildLocation];
}
pub trait HasEndPath {
    fn get_end_path(&self) -> &[ChildLocation];
}
pub trait PatternStart: PatternEntry + HasStartPath {
    fn get_start<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        if let Some(next) = self.get_start_path().last() {
            trav.graph().expect_child_at(next)
        } else {
            self.get_entry()
        }
    }
}
pub trait PatternEnd: PatternExit + HasEndPath + End {
    fn get_pattern_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        if let Some(start) = self.get_end_path().last() {
            trav.graph().expect_child_at(start)
        } else {
            self.get_exit()
        }
    }
}
pub trait GraphStart: GraphEntry + HasStartPath {
    fn get_start_location(&self) -> ChildLocation {
        if let Some(start) = self.get_start_path().last() {
            start.clone()
        } else {
            self.get_entry_location()
        }
    }
    fn get_start<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_start_location())
    }
}
pub trait GraphEnd: GraphExit + HasEndPath + End {
    fn get_end_location(&self) -> ChildLocation {
        if let Some(end) = self.get_end_path().last() {
            end.clone()
        } else {
            self.get_exit_location()
        }
    }
    fn get_graph_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_end_location())
    }
}
pub trait EndPathMut {
    fn end_path_mut(&mut self) -> &mut ChildPath;
}
pub trait ExitMut {
    fn exit_mut(&mut self) -> &mut usize;
}
pub trait End {
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child;
}
pub trait AdvanceablePath: Clone + EndPathMut + ExitMut + End {
    fn next_exit_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize>;
    fn prev_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize>;
    fn push_next(&mut self, next: ChildLocation) {
        self.end_path_mut().push(next)
    }
    fn reduce_mismatch<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> Self {
        let graph = trav.graph();
        // remove segments pointing to mismatch at pattern head
        let end = self.end_path_mut();
        while let Some(mut location) = end.pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if let Some(prev) = D::pattern_index_prev(pattern.borrow(), location.sub_index) {
                location.sub_index = prev;
                end.push(location);
                break;
            }
        }
        if end.is_empty() {
            *self.exit_mut() = self.prev_pos::<_, D, _>(trav).unwrap();
        }
        self
    }
    fn advance_next<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) -> bool {
        let graph = trav.graph();
        // skip path segments with no successors
        let end = self.end_path_mut();
        while let Some(mut location) = end.pop() {
            let pattern = graph.expect_pattern_at(&location);
            if let Some(next) = D::pattern_index_next(pattern, location.sub_index) {
                location.sub_index = next;
                end.push(location);
                return true;
            }
        }
        // end is empty (exit is prev)
        if let Some(next) = self.next_exit_pos::<_, D, _>(trav) {
            *self.exit_mut() =  next;
            true
        } else {
            false
        }
    }
    fn get_advance<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> Option<(Child, Self)> {
        if self.advance_next::<_, D, _>(trav) {
            Some((self.get_end::<_, D, _>(trav), self))
        } else {
            None
        }
    }
}
pub(crate) struct RangePathIter<
    'a: 'g,
    'g,
    P: AdvanceablePath,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    Trav: Traversable<'a, 'g, T>,
> {
    path: P,
    trav: &'a Trav,
    _ty: std::marker::PhantomData<(&'g T, D)>
}
pub(crate) trait SequenceIterator: Sized {
    type Item;
    fn next(self) -> Result<Self::Item, Self::Item>;
}
impl<
    'a: 'g,
    'g,
    P: AdvanceablePath,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    Trav: Traversable<'a, 'g, T>,
> SequenceIterator for RangePathIter<'a, 'g, P, T, D, Trav> {
    type Item = P;
    fn next(mut self) -> Result<Self::Item, Self::Item> {
        if self.path.advance_next::<_, D, _>(self.trav) {
            Ok(self.path)
        } else {
            Err(self.path)
        }
    }
}
//impl<
//    'a: 'g,
//    'g,
//    P: RangePath,
//    T: Tokenize + 'a,
//    D: MatchDirection + 'a,
//    Trav: Traversable<'a, 'g, T>,
//> RangePathIter<'a, 'g, P, T, D, Trav> {
//    fn get_pattern(&self) -> Pattern {
//        self.path.get_pattern(self.trav)
//    }
//    fn push_next(&mut self, next: ChildLocation) {
//        self.path.push_next(next)
//    }
//    fn move_width_into_start(&mut self) {
//        self.path.move_width_into_start()
//    }
//    fn reduce_mismatch(mut self) -> P {
//        self.path.reduce_mismatch::<_, D, _>(self.trav)
//    }
//    fn get_entry_pos(&self) -> usize {
//        self.path.get_entry_pos()
//    }
//    fn get_exit_pos(&self) -> usize {
//        self.path.get_exit_pos()
//    }
//    fn get_end(&self) -> Child {
//        self.path.get_end::<_, D, _>(self.trav)
//    }
//    fn prev_pos(&self) -> Option<usize> {
//        self.path.prev_pos::<_, D, _>(self.trav)
//    }
//    fn on_match(&mut self) {
//        self.path.on_match::<_, D, _>(self.trav)
//    }
//}
pub(crate) trait IntoSequenceIterator<
    'a: 'g,
    'g,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    Trav: Traversable<'a, 'g, T>,
> {
    type Iter: SequenceIterator;
    fn into_seq_iter(self, trav: &'a Trav) -> Self::Iter;
}
impl<
    'a: 'g,
    'g,
    P: AdvanceablePath,
    T: Tokenize + 'a,
    D: MatchDirection + 'a,
    Trav: Traversable<'a, 'g, T>,
> IntoSequenceIterator<'a, 'g, T, D, Trav> for P {
    type Iter = RangePathIter<'a, 'g, P, T, D, Trav>;
    fn into_seq_iter(self, trav: &'a Trav) -> Self::Iter {
        RangePathIter {
            path: self,
            trav,
            _ty: Default::default(),
        }
    }
}