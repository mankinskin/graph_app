use std::borrow::Borrow;

use crate::{
    vertex::*,
    *,
};
pub(crate) struct RangePathIter<
    'a: 'g,
    'g,
    P: RangePath,
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
    P: RangePath,
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
    P: RangePath,
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

pub trait RangePath: Clone {
    fn get_exit_pos(&self) -> usize;
    fn get_entry_pos(&self) -> usize;
    fn get_pattern<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Pattern;
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child;
    fn get_exit<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        self.get_pattern(trav)[self.get_exit_pos()]
    }
    fn push_next(&mut self, next: ChildLocation);
    fn reduce_mismatch<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> Self;
    fn prev_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize>;
    fn advance_next<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) -> bool;
}
pub trait QueryPath: RangePath {
    fn get_entry(&self) -> Child;
    fn complete(pattern: impl IntoPattern) -> Self;
}
pub(crate) trait GraphPath: RangePath + Into<GraphRangePath> {
    fn from_start_path(start: StartPath) -> Self;
    fn into_start_path(self) -> StartPath;
    fn reduce_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(self, trav: &'a Trav) -> FoundPath;
    fn move_width_into_start(&mut self);
    fn on_match<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav);
}
impl GraphPath for GraphRangePath {
    fn from_start_path(start: StartPath) -> Self {
        Self::new(start)
    }
    fn into_start_path(self) -> StartPath {
        self.start
    }
    fn reduce_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> FoundPath {
        let graph = trav.graph();
        //self.reduce_end_path::<T, D>(&*graph);
        // remove segments pointing to mismatch at pattern head
        while let Some(location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if D::pattern_index_next(pattern.borrow(), location.sub_index).is_some() {
                self.end.push(location);
                break;
            }
        }
        if self.end.is_empty() {
            self.move_width_into_start();
        }
        FoundPath::new::<_, D, _>(trav, self)
    }
    fn move_width_into_start(&mut self) {
        *self.start.width_mut() += self.inner_width + self.end_width;
        self.inner_width = 0;
        self.end_width = 0;
    }
    fn on_match<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&mut self, trav: &'a Trav) {
        let width = self.get_end::<_, D, _>(trav).width;
        let wmut = if self.end.is_empty() {
            &mut self.inner_width
        } else {
            &mut self.end_width
        };
        *wmut += width;
    }
}
impl RangePath for GraphRangePath {
    fn get_pattern<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Pattern {
        trav.graph().expect_pattern_at(self.get_end_location())
    }
    fn push_next(&mut self, next: ChildLocation) {
        self.end.push(next);
    }
    fn reduce_mismatch<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(mut self, trav: &'a Trav) -> Self {
        let graph = trav.graph();
        //self.reduce_end_path::<T, D>(&*graph);
        // remove segments pointing to mismatch at pattern head
        while let Some(mut location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if let Some(prev) = D::pattern_index_prev(pattern.borrow(), location.sub_index) {
                location.sub_index = prev;
                self.end.push(location);
                break;
            }
        }
        if self.end.is_empty() {
            self.exit = self.prev_pos::<_, D, _>(trav).unwrap();
        }

        self
    }
    fn get_entry_pos(&self) -> usize {
        self.start.entry().sub_index
    }
    fn get_exit_pos(&self) -> usize {
        self.exit
    }
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        trav.graph().expect_child_at(self.get_end_location())
    }
    fn prev_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        let location = self.get_end_location();
        let pattern = trav.graph().expect_pattern_at(&location);
        D::pattern_index_prev(pattern, location.sub_index)
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
        while let Some(mut location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(&location);
            if let Some(next) = D::pattern_index_next(pattern, location.sub_index) {
                location.sub_index = next;
                self.end.push(location);
                return true;
            }
        }
        // end is empty (exit is prev)
        if let Some(next) = self.next_pos::<_, D, _>(trav) {
            self.exit = next;
            true
        } else {
            false
        }
    }
}
impl QueryPath for QueryRangePath {
    fn get_entry(&self) -> Child {
        self.query[self.entry]
    }
    fn complete(query: impl IntoPattern) -> Self {
        let query = query.into_pattern();
        Self {
            entry: 0,
            exit: query.len() - 1,
            query,
            start: vec![],
            end: vec![],
        }
    }
}
impl RangePath for QueryRangePath {
    fn get_pattern<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        Trav: Traversable<'a, 'g, T>
    >(&self, _trav: &'a Trav) -> Pattern {
        self.query.clone()
    }
    fn get_entry_pos(&self) -> usize {
        self.entry
    }
    fn get_exit_pos(&self) -> usize {
        self.exit
    }
    fn get_end<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Child {
        if let Some(next) = self.end.last() {
            trav.graph().expect_child_at(next)
        } else {
            self.get_exit()
        }
    }
    fn push_next(&mut self, next: ChildLocation) {
        self.end.push(next);
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
        while let Some(mut location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(&location);
            // skip segments at end of pattern
            if let Some(prev) = D::pattern_index_prev(pattern.borrow(), location.sub_index) {
                location.sub_index = prev;
                self.end.push(location);
                break;
            }
        }
        if self.end.is_empty() {
            self.exit = self.prev_pos::<_, D, _>(trav).unwrap();
        }

        self
    }
    fn prev_pos<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        D: MatchDirection + 'a,
        Trav: Traversable<'a, 'g, T>,
    >(&self, trav: &'a Trav) -> Option<usize> {
        if self.end.is_empty() {
            D::pattern_index_prev(self.query.borrow(), self.exit)
        } else {
            let location = self.end.last().unwrap().clone();
            let pattern = trav.graph().expect_pattern_at(&location);
            D::pattern_index_prev(pattern, location.sub_index)
        }
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
        while let Some(mut location) = self.end.pop() {
            let pattern = graph.expect_pattern_at(location);
            if let Some(next) = D::pattern_index_next(pattern, location.sub_index) {
                location.sub_index = next;
                self.end.push(location);
                return true;
            }
        }
        // end is empty (exit is prev)
        if let Some(next) = D::pattern_index_next(self.query.borrow(), self.exit) {
            self.exit = next;
            true
        } else {
            false
        }
    }
}