pub mod merge_direction;
pub mod split_minimizer;
pub use {
    merge_direction::*,
    split_minimizer::*,
};
use crate::{
    graph::*,
    split::*,
    vertex::*,
};
impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    pub fn left_merger(
        &mut self,
    ) -> SplitMinimizer<T, Left> {
        SplitMinimizer::new(self)
    }
    pub fn right_merger(
        &mut self,
    ) -> SplitMinimizer<T, Right> {
        SplitMinimizer::new(self)
    }
    pub fn merge_left_split(
        &mut self,
        context: Pattern,
        inner: SplitSegment,
    ) -> SplitSegment {
        self.left_merger().merge_split(context.into(), inner)
    }
    pub fn merge_right_split(
        &mut self,
        context: Pattern,
        inner: SplitSegment,
    ) -> SplitSegment {
        self.right_merger().merge_split(context.into(), inner)
    }
    pub fn merge_left_splits(
        &mut self,
        splits: Vec<(Pattern, SplitSegment)>,
    ) -> SplitSegment {
        self.left_merger().merge_splits(splits)
    }
    pub fn merge_right_splits(
        &mut self,
        splits: Vec<(Pattern, SplitSegment)>,
    ) -> SplitSegment {
        self.right_merger().merge_splits(splits)
    }
    pub fn merge_left_optional_splits(
        &mut self,
        splits: Vec<(Pattern, Option<SplitSegment>)>,
    ) -> SplitSegment {
        self.left_merger().merge_optional_splits(splits)
    }
    pub fn merge_right_optional_splits(
        &mut self,
        splits: Vec<(Pattern, Option<SplitSegment>)>,
    ) -> SplitSegment {
        self.right_merger().merge_optional_splits(splits)
    }
    pub fn merge_inner_optional_splits(
        &mut self,
        splits: Vec<(Option<SplitSegment>, SplitSegment, Option<SplitSegment>)>,
    ) -> Child {
        self.left_merger().merge_inner_optional_splits(splits)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        split::*,
        vertex::*,
        *,
    };
    use maplit::hashset;
    use pretty_assertions::assert_eq;
    use std::{
        collections::HashSet,
        num::NonZeroUsize,
    };
    use itertools::*;
    #[test]
    fn merge_single_split_1() {
        let mut graph = Hypergraph::default();
        let (a, b, c, d) = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('c'),
            Token::Element('d'),
        ]).into_iter().next_tuple().unwrap();
        // wxabyzabbyxabyz
        let ab = graph.insert_pattern([a, b]);
        let bc = graph.insert_pattern([b, c]);
        let cd = graph.insert_pattern([c, d]);
        let abc = graph.insert_patterns([vec![ab, c], vec![a, bc]]);
        let bcd = graph.insert_patterns([vec![bc, d], vec![b, cd]]);
        let _abcd = graph.insert_patterns([vec![abc, d], vec![a, bcd]]);
        let left = vec![
            (vec![a], SplitSegment::Child(b)),
            (vec![], SplitSegment::Child(ab)),
        ];
        let right = vec![
            (vec![d], SplitSegment::Child(c)),
            (vec![], SplitSegment::Child(cd)),
        ];
        let left = graph.merge_left_splits(left);
        let right = graph.merge_right_splits(right);
        assert_eq!(left, SplitSegment::Child(ab), "left");
        assert_eq!(right, SplitSegment::Child(cd), "right");
    }
    #[test]
    fn merge_split_2() {
        let mut graph = Hypergraph::default();
        let (a, b, _w, x, y, z) = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('w'),
            Token::Element('x'),
            Token::Element('y'),
            Token::Element('z'),
        ]).into_iter().next_tuple().unwrap();
        // wxabyzabbyxabyz
        let ab = graph.insert_pattern([a, b]);
        let by = graph.insert_pattern([b, y]);
        let yz = graph.insert_pattern([y, z]);
        let xa = graph.insert_pattern([x, a]);
        let xab = graph.insert_patterns([[x, ab], [xa, b]]);
        let xaby = graph.insert_patterns([vec![xab, y], vec![xa, by]]);
        let xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);

        let mut splitter = IndexSplitter::new(&mut graph);
        let (ps, child_splits) =
            splitter.get_perfect_split_separation(xabyz, NonZeroUsize::new(2).unwrap());
        assert_eq!(ps, None);
        let (left, right) = splitter.split_child_indices(child_splits);

        let expleft = hashset![(vec![], SplitSegment::Child(xa)),];
        let expright = hashset![
            (vec![yz], SplitSegment::Child(b)),
            (vec![z], SplitSegment::Child(by)),
        ];

        let (sleft, sright): (HashSet<_>, HashSet<_>) = (
            left.clone().into_iter().collect(),
            right.clone().into_iter().collect(),
        );
        assert_eq!(sleft, expleft, "left");
        assert_eq!(sright, expright, "right");

        let left = graph.merge_left_splits(left);
        let right = graph.merge_right_splits(right);
        println!("{:#?}", graph);
        println!("left = {:#?}", left);
        println!("right = {:#?}", right);
        let byz_found = graph.find_ancestor(vec![b, y, z]);
        let byz = if let SearchFound {
            index: byz,
            parent_match:
                ParentMatch {
                    parent_range: FoundRange::Complete,
                    ..
                },
            ..
        } = byz_found.expect("byz not found")
        {
            Some(byz)
        } else {
            None
        }
        .expect("byz");
        assert_eq!(left, SplitSegment::Child(xa), "left");
        assert_eq!(right, SplitSegment::Child(byz), "left");
    }
}
