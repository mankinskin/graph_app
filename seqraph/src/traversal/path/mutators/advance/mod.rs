use crate::*;

pub mod exit;
pub mod from_advanced;
pub use exit::*;
pub use from_advanced::*;


pub trait IntoAdvanced<R: ResultKind>: Sized + Clone + Into<R::Advanced> {
    fn into_advanced<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(
        self,
        trav: &'a Trav,
    ) -> Result<R::Advanced, Self> {
        let mut new: R::Advanced = self.clone().into();
        match new.advance_exit_pos::<_, D, _>(trav) {
            Ok(()) => Ok(new),
            Err(()) => Err(self)
        }
    }
}
impl<
    R: ResultKind,
    T: Sized + Clone + Into<R::Advanced>
> IntoAdvanced<R> for T {
}

pub trait Advance:
    AdvanceExit
    + HasPath<End>
    + PathChild<End>
    + AdvanceWidth
    + Sized
{
    fn advance<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&mut self, trav: &'a Trav) -> Option<Child> {
        if self.is_finished(trav) {
            None
        } else {
            let current = self.path_child(trav);
            let graph = trav.graph();
            // skip path segments with no successors
            while let Some(mut location) = self.path_mut().pop() {
                let pattern = graph.expect_pattern_at(&location);
                if let Some(next) = D::pattern_index_next(pattern.borrow(), location.sub_index) {
                    location.sub_index = next;
                    //let next = pattern[next];
                    //self.advance_width(next.width);
                    self.path_mut().push(location);
                    return current;
                }
            }
            // end is empty (exit is prev)
            let _ = self.advance_exit_pos::<_, D, _>(trav);
            current
        }
    }
}
impl<T: 
    AdvanceExit
    + AdvanceWidth
    + HasPath<End>
    + PathChild<End>
    + Sized
> Advance for T {
}

pub trait AdvanceWidth {
    fn advance_width(&mut self, width: usize);
}
impl <T: WideMut> AdvanceWidth for T {
    fn advance_width(&mut self, width: usize) {
        *self.width_mut() += width;
    }
}

pub trait AddMatchWidth: AdvanceWidth + LeafChild<End> {
    fn add_match_width<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>,
    >(&mut self, trav: &'a Trav) {
        let leaf = self.leaf_child(trav);
        self.advance_width(leaf.width);
    }
}
impl<T: AdvanceWidth + LeafChild<End>> AddMatchWidth for T {
}