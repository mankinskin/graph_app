use crate::*;

pub trait PathSimplify: Sized {
    fn into_simplified<
        Trav: Traversable,
    >(self, trav: &Trav) -> Self;
    fn simplify<
        Trav: Traversable,
    >(&mut self, trav: &Trav) {
	    unsafe {
	    	let old = std::ptr::read(self);
	    	let new = old.into_simplified(trav);
	    	std::ptr::write(self, new);
	    }
    }
}
impl<R: PathRole> PathSimplify for RolePath<R> {
    fn into_simplified<
        Trav: Traversable,
    >(mut self, trav: &Trav) -> Self {
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
    fn into_simplified<
        Trav: Traversable,
    >(self, trav: &Trav) -> Self {
        if let Some(c) = match self.get_path() {
            Some(p) => if p.single_path().is_empty() && {
                let location = p.root_child_location();
                let graph = trav.graph();
                let pattern = graph.expect_pattern_at(&location);
                <Trav::Kind as GraphKind>::Direction::pattern_index_prev(pattern.borrow(), location.sub_index).is_none()
            } {
                Some(p.root_parent())
            } else {
                None
            },
            None => None,
        } {
            MatchEnd::Complete(c)
        } else {
            self
        }
    }
}
impl PathSimplify for (EndState, TokenLocation) {
    fn into_simplified<
        Trav: Traversable,
    >(mut self, trav: &Trav) -> Self {
        match self.0.kind {
            EndKind::Complete(_) |
            EndKind::Postfix(_) |
            EndKind::Prefix(_) => self,
            EndKind::Range(mut s) => {
                s.path.child_path_mut::<Start>().simplify(trav);
                s.path.child_path_mut::<End>().simplify(trav);

                while s.path.role_root_child_pos::<Start>() == s.path.role_root_child_pos::<End>() {
                    if (&mut self.0.root_pos, &mut s).path_lower(trav).is_break() {
                        let graph = trav.graph();
                        let pattern = graph.expect_pattern_at(&s.path.root.location);
                        let entry = s.path.start.sub_path.root_entry;
                        self.0.root_pos.retract_key(
                            self.0.root_pos.pos - self.1.pos
                        );
                        self.0.kind = EndKind::Complete(pattern[entry]);
                        return self;
                    }
                }

                self.0.kind = match (
                    Start::is_at_border(trav.graph(), s.path.role_root_child_location::<Start>()),
                    s.path.raw_child_path::<Start>().is_empty(),
                    End::is_at_border(trav.graph(), s.path.role_root_child_location::<End>()),
                    s.path.raw_child_path::<End>().is_empty(),
                ) 
                {
                    (true, true, true, true) =>
                        EndKind::Complete(s.path.root_parent()),
                    (true, true, false, _) |
                    (true, true, true, false) =>
                        EndKind::Prefix(s.path.into()),
                    (false, _, true, true) |
                    (true, false, true, true) =>
                        EndKind::Postfix(s.path.into()),
                    _ => EndKind::Range(s),
                };
                self
            }
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
