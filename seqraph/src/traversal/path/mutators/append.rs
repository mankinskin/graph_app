use crate::*;

pub trait PathAppend {
    type Result;
    fn append<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &Trav, parent_entry: ChildLocation) -> Self::Result;
}

//impl PathAppend for PathLeaf {
//    type Result = ChildPath;
//    fn append<
//        'a: 'g,
//        'g,
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>
//    >(self, trav: &'a Trav, parent_entry: ChildLocation) -> Self::Result {
//        let graph = trav.graph();
//        let pattern = graph.expect_pattern_at(self.entry);
//        if self.entry.sub_index == D::head_index(pattern.borrow()) {
//            ChildPath::Leaf(PathLeaf {
//                entry: parent_entry,
//                child: self.entry.parent,
//                width: self.width,
//                token_pos: self.token_pos,
//            })
//        } else {
//            ChildPath::Path {
//                entry: parent_entry,
//                path: vec![self.entry],
//                width: self.width,
//                child: self.child,
//                token_pos: self.token_pos,
//            }
//        }
//    }
//}

impl<R: PathRole> PathAppend for ChildPath<R> {
    type Result = Self;
    fn append<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(mut self, trav: &Trav, parent_entry: ChildLocation) -> Self::Result {
        //println!("path {} -> {}, {}", entry.parent.index, parent_entry.parent.index, width);
        let entry = self.child_location();
        let graph = trav.graph();
        let pattern = self.graph_root_pattern::<_, Trav>(&graph);
        // start paths only at a non-head index position 
        if entry.sub_index != D::head_index(pattern.borrow()) {
            self.path.push(parent_entry);
        } else {
            self.path = vec![parent_entry]
        }
        self
    }
}
impl<P: PathAppend> PathAppend for OriginPath<P>
    where <P as PathAppend>::Result: PathAppend<Result=<P as PathAppend>::Result> + GraphRootChild<Start>
{
    type Result = OriginPath<<P as PathAppend>::Result>;
    fn append<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &Trav, parent_entry: ChildLocation) -> Self::Result {
        OriginPath {
            origin: MatchEnd::Path(self.origin.append::<_, D, _>(trav, parent_entry)),
            postfix: self.postfix.append::<_, D, _>(trav, parent_entry),
        }
    }
}
impl<P: MatchEndPath + PathAppend> PathAppend for MatchEnd<P> {
    type Result = <P as PathAppend>::Result;
    fn append<
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &Trav, parent_entry: ChildLocation) -> Self::Result {
        match self {
            MatchEnd::Path(path) => path.append::<_, D, _>(trav, parent_entry),
            MatchEnd::Complete(child) => ChildPath {
                path: vec![parent_entry],
                width: child.width(),
                child,
                token_pos: 0,
                _ty: Default::default(),
            }.into(),
        }
    }
}
