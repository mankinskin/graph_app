use crate::*;

pub trait PathAppend {
    type Result;
    fn append<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &'a Trav, parent_entry: ChildLocation) -> Self::Result;
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

impl PathAppend for ChildPath<Start> {
    type Result = Self;
    fn append<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &'a Trav, parent_entry: ChildLocation) -> Self::Result {
        //println!("path {} -> {}, {}", entry.parent.index, parent_entry.parent.index, width);
        let entry = self.child_location();
        let pattern = self.graph_root_pattern(trav);
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
    where <P as PathAppend>::Result: PathAppend<Result=<P as PathAppend>::Result> + RangePath + GraphRootChild<Start>
{
    type Result = OriginPath<<P as PathAppend>::Result>;
    fn append<
        'a: 'g,
        'g,
        T: Tokenize,
        D: MatchDirection,
        Trav: Traversable<T>
    >(self, trav: &'a Trav, parent_entry: ChildLocation) -> Self::Result {
        OriginPath {
            origin: MatchEnd::Path(self.origin.append::<_, D, _>(trav, parent_entry)),
            postfix: self.postfix.append::<_, D, _>(trav, parent_entry),
        }
    }
}