#![allow(non_snake_case, unused)]
use crate::*;

type BuildKey = RangeInclusive<usize>;

//#[test]
pub fn test_grammar() {
    let N: usize = 100; // total length
    let k: usize = 20; // alphabet size
    //let mut graph = HypergraphRef::<BaseGraphKind>::default();
    println!("N = {}\nk = {}", N, k);
    let num_v = count_max_nodes(N, k);
    println!("num_v = {}", num_v);
    println!("1/2N^2 + 1/2 = {}", N.pow(2) as f32/2.0 + 1.5);
    println!("diff = {}", (num_v as f32 - (N.pow(2) as f32/2.0 + 1.5)).abs());

    println!("Generating saturated grammar (N = {}) ...", N);
    let g = worst_case_grammar(N, k);
    println!("num_v = {}", g.vertex_count());
    println!("num_e = {}", 4*g.vertex_count());
    let num_bytes = g.vertex_count() * (
            std::mem::size_of::<VertexData>()
            + std::mem::size_of::<VertexKey>()
        )
        + 4*g.vertex_count()
        * (
            std::mem::size_of::<Child>()
            + std::mem::size_of::<Parent>()
        );
    println!("total MB = {}",
        num_bytes as u32 / 10_u32.pow(6),
    );
    println!("mul = {}",
        num_bytes / N,
    );
}

#[derive(new, Deref)]
struct BuilderNode {
    index: Child,
    #[deref]
    range: BuildKey,
}
impl BuilderNode {
    pub fn prefix_rule(&self) -> [BuildKey; 2] {
        [
            *self.start()..=self.end()-1,
            *self.end()..=*self.end(),
        ]
    }
    pub fn postfix_rule(&self) -> [BuildKey; 2] {
        [
            *self.start()..=*self.start(),
            *self.start()+1..=*self.end(),
        ]
    }
}
struct GraphBuilder {
    range_map: HashMap<BuildKey, usize>,
    queue: VecDeque<BuilderNode>,
    graph: Hypergraph,
    N: usize,
}
impl GraphBuilder {
    pub fn new(N: usize) -> Self {
        Self {
            N,
            range_map: Default::default(),
            graph: Default::default(),
            queue: Default::default(),
        }
    }
    pub fn queue_node(
        &mut self,
        node: BuilderNode,
    ) {
        self.graph.insert_vertex(
            VertexKey::Pattern(node.index.vertex_index()),
            VertexData::new(
                node.index.vertex_index(),
                node.range.clone().count(),
            )
        );
        self.queue.push_back(node);
    }

    pub fn add_rules(
        &mut self,
        node: BuilderNode,
    ) {
        for rule in match node.index.width() {
            1 => vec![],
            2 => vec![
                node.prefix_rule(),
            ],
            _ => vec![
                node.prefix_rule(),
                node.postfix_rule(),
            ]
        } {
            let pid = self.graph.next_pattern_id();
            let pattern: Pattern = rule.iter().enumerate().map(|(sub_index, key)| {
                let loc = ChildLocation::new(node.index, pid, sub_index);
                if let Some(v) = self.range_map.get(&key) {
                    self.graph.expect_vertex_data_mut(v).add_parent(loc);
                    Child::new(*v, key.clone().count())
                } else {
                    self.range_map.insert(key.clone(), self.range_map.len());
                    let vid = self.graph.next_vertex_id();
                    let c = Child::new(vid, key.clone().count());
                    self.queue_node(BuilderNode::new(c, key.clone()));
                    c
                }
            }).collect();
            self.graph.expect_vertex_data_mut(node.index)
                .add_pattern_no_update(pid, pattern);
        }
    }
    pub fn fill_grammar(&mut self) {
        let vid = self.graph.next_vertex_id();
        self.queue_node(BuilderNode::new(Child::new(vid, self.N), 0..=self.N-1));
        while let Some(node) = self.queue.pop_front() {
            self.add_rules(node);
        }
    }
    //fn postfix_count(&self, index: Child) -> usize {
    //    self.graph.expect_vertex_data(index)
    //        .get_parents_with_index_at(1)
    //        .len()
    //}
    //fn prefix_count(&self, index: Child) -> usize {
    //    self.graph.expect_vertex_data(index)
    //        .get_parents_with_index_at(0)
    //        .len()
    //}
    pub fn saturated_grammar(mut self, k: usize) -> Hypergraph {
        self.fill_grammar();
        let mut ctx = RewireContext::new(k, self);
        ctx.rewire_grammar();
        ctx.builder.graph
    }
}
// ._._._._._._._._. n
// |0|             | 1
                             
// |0:1|           | 2

// |0 1:1|         | 3
// | |1:1|         | 2

// |0 1 1:0|       | 4
// | |1 1:0|       | 3
// |   |1:0|       | 2

// |0 1 1 0:0|     | 5
// | |1 1 0:0|     | 4
// |   |1 0:0|     | 3
// |     |0:0|     | 2 <- last 2 gram

// |0 1 1 0 0:0|   | 6
// | |1 1 0 0:0|   | 5
// |   |1 0 0:0|   | 4
// |     |0 0:0|   | 3
// |       |0:0|   | 2 <- repeat n-gram, replace with previous index

// |0 1 1 0 0 0:1| | 7
// | |1 1 0 0 0:1| | 6
// |   |1 0 0 0:1| | 5
// |     |0 0 0:1| | 4
// |       |0 0:1| | 3
// |         |0:1| | 2

// |0 1 1 0 0 0 1:0| 8
// | |1 1 0 0 0 1:0| 7
// |   |1 0 0 0 1:0| 6
// |     |0 0 0 1:0| 5
// |       |0 0 1:0| 4
// |         |0 1:0| 3
// |           |1:0| 2
// |             |0| 1

// |0|1|1|0|0|0|1|0| N = 8
// 
struct RewireContext {
    builder: GraphBuilder,
    prefix_counts: HashMap<VertexIndex, usize>,
    k: usize,
}
impl RewireContext {
    pub fn new(k: usize, builder: GraphBuilder) -> Self {
        Self {
            builder,
            prefix_counts: Default::default(),
            k,
        }
    }
    //fn select_next_token(&self, prefixes: &Vec<VertexIndex>) -> VertexIndex {
    //    if prefixes.iter().any(|c| self.prefix_counts.get(c) == Some(&self.k)) {

    //    } else {

    //    }
    //}
    //fn extend_prefixes(&self, prefixes: Vec<VertexIndex>) -> Vec<VertexIndex> {
    //    // determine 1-gram at next position in range_map
    //    // select new k-token for position
    //    // rewire all parents of previous index to new k-token index
    //    let next_token = self.select_next_token(&prefixes);
    //    let x = (&self.builder.graph).find_parent(vec![next_token, next_token]);
    //    prefixes.into_iter()
    //        .map(|p| {
    //        })
    //}
    pub fn rewire_grammar(&mut self) {
        // - fix first token
        // - store number of prefix uses for each index   
        // - implement function selecting the next token given the previous n-grams
        // - call next token with previous n grams until counts reach k
        // - implement rewire function to point index edges to previous index
        // - 
        let first = *self.builder.range_map.get(&(0..=0)).expect("Must include range 0..=0 in range_map");
        let mut prefixes = vec![first];
    }
}
fn worst_case_grammar(N: usize, k: usize) -> Hypergraph {
    GraphBuilder::new(N).saturated_grammar(k)
}
fn count_max_nodes(N: usize, k: usize) -> usize {
    // idea: approximate root of k^n = N + 1 - n
    // decide which maximum number of variants to count
    let root = ((N + 1) as f32).log(k as f32);
    println!("n0: {}", root);
    let root: f32 = nrfind::find_root(
        &|x| (k as f32).powf(x) as f32 - N as f32 + x - 1.0,
        &|x| (k as f32).powf(x) as f32 * (k as f32).ln() + 1.0,
        root,
        0.0001,
        50,
    ).unwrap();
    let root: u32 = root.floor() as u32;
    println!("root: {}", root);

    (2..=root).into_iter().map(|n|
        k.pow(n)
    ).chain(
        ((root as usize + 1)..=N).into_iter().map(|n|
            N + 1 - n
        )
    ).sum()
}
