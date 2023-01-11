use crate::*;

/// access to a rooted path pointing to a descendant
pub trait HasPath<R> {
    fn path(&self) -> &Vec<ChildLocation>;
    fn path_mut(&mut self) -> &mut Vec<ChildLocation>;
}
impl HasPath<End> for QueryRangePath {
    fn path(&self) -> &Vec<ChildLocation> {
        &self.end
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        &mut self.end
    }
}
impl HasPath<Start> for QueryRangePath {
    fn path(&self) -> &Vec<ChildLocation> {
        &self.start
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        &mut self.start
    }
}
//impl HasPath<End> for PrefixQuery {
//    fn path(&self) -> &Vec<ChildLocation> {
//        self.end.map(|p| p.path()).unwrap_or_default()
//    }
//    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
//        self.end.map(|p| p.path_mut()).unwrap_or_default()
//    }
//}
//impl HasPath<End> for OverlapPrimer {
//    fn path(&self) -> &Vec<ChildLocation> {
//        if self.exit == 0 {
//            self.end.borrow()
//        } else {
//            self.context.end.borrow()
//        }
//    }
//    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
//        if self.exit == 0 {
//            self.end.borrow_mut()
//        } else {
//            self.context.end.borrow_mut()
//        }
//    }
//}
impl<R: 'static> HasPath<R> for SearchPath
    where SearchPath: HasRolePath<R>
{
    fn path(&self) -> &Vec<ChildLocation> {
        HasRolePath::<R>::role_path(self).path()
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        HasRolePath::<R>::role_path_mut(self).path_mut()
    }
}
//impl<R, T: HasRolePath<R>> HasPath<R> for T {
//    fn path(&self) -> &Vec<ChildLocation> {
//        self.role_path().path()
//    }
//    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
//        self.role_path_mut().path_mut()
//    }
//}
impl<R, P: HasPath<R>> HasPath<R> for OriginPath<P> {
    fn path(&self) -> &Vec<ChildLocation> {
        HasPath::<R>::path(&self.postfix)
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        HasPath::<R>::path_mut(&mut self.postfix)
    }
}
// todo: does not give complete path (missing first segment)
//impl<R> HasPath<R> for PathLeaf {
//    fn path(&self) -> &Vec<ChildLocation> {
//        self.path()
//    }
//    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
//        self.path_mut()
//    }
//}
impl<R> HasPath<R> for ChildPath<R> {
    fn path(&self) -> &Vec<ChildLocation> {
        &self.path
    }
    fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        &mut self.path
    }
}

/// access to a rooted path pointing to a descendant
pub trait HasRolePath<R>: HasPath<R> {
    fn role_path(&self) -> &ChildPath<R>;
    fn role_path_mut(&mut self) -> &mut ChildPath<R>;
    fn num_path_segments(&self) -> usize {
        self.role_path().num_path_segments()
    }
}
impl<R> HasRolePath<R> for ChildPath<R> {
    fn role_path(&self) -> &ChildPath<R> {
        self
    }
    fn role_path_mut(&mut self) -> &mut ChildPath<R> {
        self
    }
}
impl HasRolePath<Start> for SearchPath {
    fn role_path(&self) -> &ChildPath<Start> {
        &self.start
    }
    fn role_path_mut(&mut self) -> &mut ChildPath<Start> {
        &mut self.start
    }
}
impl HasRolePath<End> for SearchPath {
    fn role_path(&self) -> &ChildPath<End> {
        &self.end
    }
    fn role_path_mut(&mut self) -> &mut ChildPath<End> {
        &mut self.end
    }
}
impl<R, P: HasRolePath<R>> HasRolePath<R> for OriginPath<P> {
    fn role_path(&self) -> &ChildPath<R> {
        self.postfix.role_path()
    }
    fn role_path_mut(&mut self) -> &mut ChildPath<R> {
        self.postfix.role_path_mut()
    }
}
pub trait HasMatchPaths: HasRolePath<Start> + HasRolePath<End> {
    fn into_paths(self) -> (ChildPath<Start>, ChildPath<End>);
    fn num_path_segments(&self) -> usize {
        HasRolePath::<Start>::role_path(self).num_path_segments() + HasRolePath::<End>::role_path(self).num_path_segments()
    }
    fn min_path_segments(&self) -> usize {
        HasRolePath::<Start>::role_path(self).num_path_segments().min(
            HasRolePath::<End>::role_path(self).num_path_segments()
        )
    }
    //fn root(&self) -> Child {
    //    self.role_path().root()
    //}
}
impl HasMatchPaths for SearchPath {
    fn into_paths(self) -> (ChildPath<Start>, ChildPath<End>) {
        (self.start, self.end)
    }
}

pub trait HasSinglePath {
    fn single_path(&self) -> &[ChildLocation];
}
impl<R> HasSinglePath for ChildPath<R> {
    fn single_path(&self) -> &[ChildLocation] {
        self.path().borrow()
    }
}
