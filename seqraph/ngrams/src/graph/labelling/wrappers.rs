use crate::{
    *,
    shared::*,
};

#[derive(Debug, Deref, From, DerefMut)]
pub struct WrapperCtx<'a, 'b: 'a> {
    #[deref]
    #[deref_mut]
    ctx: &'a mut LabellingCtx<'b>,
}
// - run bottom up (lower nodes need to be labelled)
// - for each node x:
//  - run top down to find largest frequent children to cover whole range
//  - label node x if there are multiple overlapping labelled child nodes

impl<'a, 'b: 'a> WrapperCtx<'a, 'b> {
    pub fn on_node(&mut self, vocab: &Vocabulary, node: VertexIndex) -> Vec<VertexIndex> {
        let entry = vocab.get(&node).unwrap();
        let mut queue: VecDeque<_> = TopDown::next_nodes(&entry).into_iter().collect();
        let mut ranges: HashSet<_> = HashSet::default();


        while !queue.is_empty() {
            let mut visited: HashSet<_> = Default::default();
            let mut next_layer: Vec<_> = Default::default();
            while let Some((off, node)) = queue.pop_front() {
                visited.insert(node.index);
                if self.labels.contains(&node.index) {
                    ranges.insert(off..off + node.width());
                } else {
                    let ne = vocab.get(&node.index).unwrap();
                    next_layer.extend(
                        TopDown::next_nodes(&ne).into_iter().filter_map(|(o, c)|
                            (!visited.contains(&c.index)).then(||
                                (o + off, c)
                            )
                        )
                    );
                }
            }
            queue.extend(next_layer)
        }
        //println!("ranges finished");
        if ranges.iter().cartesian_product(&ranges).find(|(l, r)|
            l.does_intersect(*r)
        ).is_some() {
            println!("wrapper");
            self.labels.insert(node);
        }
        BottomUp::next_nodes(&entry)
    }
    pub fn wrapping_pass(&mut self, vocab: &Vocabulary) {

        let mut queue: VecDeque<VertexIndex> = BottomUp::starting_nodes(&vocab);
        let mut n = 0;
        while !queue.is_empty() {
            n += 1;
            println!("{}", n);
            let mut visited: HashSet<_> = Default::default();
            let mut next_layer: Vec<_> = Default::default();
            while let Some(node) = queue.pop_front() {
                if !visited.contains(&node) && !self.labels.contains(&node) {
                    visited.insert(node);
                    let next = self.on_node(&vocab, node);
                    next_layer.extend(next);
                }
            }
            queue.extend(next_layer)
        }
    }

}