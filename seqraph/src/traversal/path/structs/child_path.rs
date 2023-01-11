use crate::*;

//#[derive(Clone, Debug, PartialEq, Eq, Hash)]
//pub struct PathLeaf {
//    pub entry: ChildLocation,
//    pub child: Child,
//    pub width: usize,
//    pub token_pos: usize,
//}
//impl PathLeaf {
//    pub fn new(child: Child, entry: ChildLocation) -> Self {
//        Self {
//            entry,
//            child,
//            width: child.width(),
//            token_pos: child.width(),
//        }
//    }
//    pub fn path(&self) -> &Vec<ChildLocation> {
//        &Vec::new()
//    }
//    pub fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
//        &mut Vec::new()
//    }
//    pub fn child_location(&self) -> ChildLocation {
//        self.entry
//    }
//    pub fn child_location_mut(&mut self) -> &mut ChildLocation {
//        &mut self.entry
//    }
//}
//impl WideMut for PathLeaf {
//    fn width_mut(&mut self) -> &mut usize {
//        &mut self.width
//    }
//}
//impl Wide for PathLeaf {
//    fn width(&self) -> usize {
//        self.width
//    }
//}
//impl From<ChildPath> for PathLeaf {
//    fn from(path: ChildPath) -> Self {
//        match path {
//            ChildPath::Leaf(leaf) => leaf,
//            ChildPath::Path {
//                entry,
//                child,
//                width,
//                token_pos,
//                ..
//            } => PathLeaf {
//                entry,
//                child,
//                width,
//                token_pos,
//            }
//        }
//    }
//}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChildPath<R> {
    //Leaf(PathLeaf),
    //Path {
    //pub entry: ChildLocation,
    pub path: Vec<ChildLocation>,
    pub child: Child,
    pub width: usize,
    pub token_pos: usize,
    pub _ty: std::marker::PhantomData<R>,
    //},
}
impl<R: PathRole> ChildPath<R> {
    pub fn get_child(&self) -> Child {
        self.child
        //match self {
        //    Self::Leaf(leaf) => leaf.child,
        //    Self::Path { child, .. } => *child,
        //}
    }
    //#[allow(unused)]
    //pub fn get_leaf(&self) -> Option<&PathLeaf> {
    //    match self {
    //        Self::Leaf(leaf) => Some(leaf),
    //        _ => None,
    //    }
    //}
    //#[allow(unused)]
    //pub fn into_path(self) -> LocationPath {
    //    match self {
    //        Self::Leaf(_leaf) => Vec::new(),
    //        Self::Path { path, .. } => path,
    //    }
    //}
    pub fn into_context_path(self) -> Vec<ChildLocation> {
        self.path
        //match self {
        //    Self::Leaf(leaf) => vec![leaf.entry],
        //    Self::Path {
        //        entry,
        //        path,
        //        ..
        //    } => path.tap_mut(|p|
        //        p.push(entry)
        //    ),
        //}
    }
    pub fn num_path_segments(&self) -> usize {
        self.path().len()
    }
    pub fn path(&self) -> &Vec<ChildLocation> {
        &self.path
        //match self {
        //    ChildPath::Leaf(leaf) => leaf.path(),
        //    ChildPath::Path{ path, .. } => path.borrow(),
        //}
    }
    pub fn path_mut(&mut self) -> &mut Vec<ChildLocation> {
        &mut self.path
        //match self {
        //    ChildPath::Leaf(leaf) => leaf.path_mut(),
        //    ChildPath::Path{ path, .. } => path.borrow_mut(),
        //}
    }
    pub fn child_location(&self) -> ChildLocation {
        <Self as GraphRootChild<R>>::root_child_location(self)
    }
    pub fn child_location_mut(&mut self) -> &mut ChildLocation {
        <Self as GraphRootChild<R>>::root_child_location_mut(self)
    }
}
impl<R> Deref for ChildPath<R> {
    type Target = Vec<ChildLocation>;
    fn deref(&self) -> &Self::Target {
        &self.path
    }
}
impl From<SearchPath> for ChildPath<Start> {
    fn from(p: SearchPath) -> Self {
        p.start
    }
}
impl From<SearchPath> for ChildPath<End> {
    fn from(p: SearchPath) -> Self {
        p.end
    }
}
impl<R> WideMut for ChildPath<R> {
    fn width_mut(&mut self) -> &mut usize {
        &mut self.width
        //match self {
        //    Self::Path { width, .. } => width,
        //    Self::Leaf(leaf) => leaf.width_mut(),
        //}
    }
}
//impl From<PathLeaf> for ChildPath {
//    fn from(leaf: PathLeaf) -> Self {
//        ChildPath::Leaf(leaf)
//    }
//}