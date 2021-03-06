use eframe::egui::{
    self,
    vec2,
    Frame,
    Pos2,
    Rect,
    Response,
    Shape,
    Stroke,
    Style,
    Ui,
    Vec2,
    Window,
};
#[allow(unused)]
use petgraph::{
    graph::{
        DiGraph,
        NodeIndex,
    },
    visit::EdgeRef,
};
use seqraph::{
    Hypergraph,
    HypergraphRef,
    Token,
    VertexData,
    VertexKey,
    Tokenize,
    Child,
    PatternId,
};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::sync::{
    Arc,
    RwLock,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Layout {
    Graph,
    Nested,
}
impl Layout {
    #[allow(unused)]
    pub fn is_graph(&self) -> bool {
        matches!(self, Self::Graph)
    }
    pub fn is_nested(&self) -> bool {
        matches!(self, Self::Nested)
    }
}
impl Default for Layout {
    fn default() -> Self {
        Self::Graph
    }
}
#[derive(Clone)]
pub struct Graph {
    pub graph: HypergraphRef<char>,
    pub vis: Arc<RwLock<GraphVis>>,
    pub insert_text: String,
}
impl Graph {
    pub fn new_from_graph(graph: Hypergraph<char>) -> Self {
        Self::new_from_graph_ref(HypergraphRef::from(graph))
    }
    pub fn new_from_graph_ref(graph: HypergraphRef<char>) -> Self {
        let vis = Arc::new(RwLock::new(GraphVis::default()));
        let new = Self {
            graph,
            vis,
            insert_text: String::from("heldldo"),
        };
        let g = new.clone();
        new.vis_mut().set_graph(g);
        new
    }
    pub fn new() -> Self {
        let graph = Hypergraph::default();
        Self::new_from_graph(graph)
    }
    pub(crate) fn graph(&self) -> std::sync::RwLockReadGuard<'_, Hypergraph<char>> {
        self.graph.read().unwrap()
    }
    pub(crate) fn graph_mut(&self) -> std::sync::RwLockWriteGuard<'_, Hypergraph<char>> {
        self.graph.write().unwrap()
    }
    #[allow(unused)]
    pub(crate) fn vis(&self) -> std::sync::RwLockReadGuard<'_, GraphVis> {
        self.vis.read().unwrap()
    }
    pub(crate) fn vis_mut(&self) -> std::sync::RwLockWriteGuard<'_, GraphVis> {
        self.vis.write().unwrap()
    }
    pub fn set_graph(&self, graph: Hypergraph<char>) {
        *self.graph_mut() = graph;
    }
    pub fn clear(&mut self) {
        *self = Self::new();
    }
    pub fn read(&mut self, text: impl ToString) {
        self.graph.read_sequence(text.to_string().chars());
    }
    pub fn show(&self, ui: &mut Ui) {
        self.vis_mut().update();
        self.vis_mut().show(ui);
    }
}
pub fn build_graph1() -> Hypergraph<char> {
    let mut graph = Hypergraph::default();
    if let [a, b, w, x, y, z] = graph.index_tokens([
        Token::Element('a'),
        Token::Element('b'),
        Token::Element('w'),
        Token::Element('x'),
        Token::Element('y'),
        Token::Element('z'),
    ])[..]
    {
        let ab = graph.index_pattern([a, b]);
        let by = graph.index_pattern([b, y]);
        let yz = graph.index_pattern([y, z]);
        let xa = graph.index_pattern([x, a]);
        let xab = graph.index_patterns([vec![x, ab], vec![xa, b]]);
        let xaby = graph.index_patterns([vec![xab, y], vec![xa, by]]);
        let xabyz = graph.index_patterns([vec![xaby, z], vec![xab, yz]]);
        let _wxabyzabbyxabyz = graph.insert_pattern([w, xabyz, ab, by, xabyz]);
    } else {
        panic!("Inserting tokens failed!");
    }
    graph
}
pub fn build_graph2() -> Hypergraph<char> {
    let mut graph = Hypergraph::default();
    if let [a, b, c, d, e, f, g, h, i] = graph.index_tokens([
        Token::Element('a'),
        Token::Element('b'),
        Token::Element('c'),
        Token::Element('d'),
        Token::Element('e'),
        Token::Element('f'),
        Token::Element('g'),
        Token::Element('h'),
        Token::Element('i'),
    ])[..]
    {
        let ab = graph.index_pattern([a, b]);
        let bc = graph.index_pattern([b, c]);
        let ef = graph.index_pattern([e, f]);
        let def = graph.index_pattern([d, ef]);
        let cdef = graph.index_pattern([c, def]);
        let gh = graph.index_pattern([g, h]);
        let efgh = graph.index_pattern([ef, gh]);
        let ghi = graph.index_pattern([gh, i]);
        let abc = graph.index_patterns([[ab, c], [a, bc]]);
        let cd = graph.index_pattern([c, d]);
        let bcd = graph.index_patterns([[bc, d], [b, cd]]);
        let abcd = graph.index_patterns([[abc, d], [a, bcd]]);
        let efghi = graph.index_patterns([[efgh, i], [ef, ghi]]);
        let abcdefghi = graph.index_patterns([vec![abcd, efghi], vec![ab, cdef, ghi]]);
        let aba = graph.index_pattern([ab, a]);
        let abab = graph.index_patterns([[aba, b], [ab, ab]]);
        let ababab = graph.index_patterns([[abab, ab], [ab, abab]]);
        let ababcd = graph.index_patterns([[ab, abcd], [aba, bcd], [abab, cd]]);
        let ababababcd =
            graph.index_patterns([vec![ababab, abcd], vec![abab, ababcd], vec![ab, ababab, cd]]);
        let ababcdefghi = graph.index_patterns([[ab, abcdefghi], [ababcd, efghi]]);
        let _ababababcdefghi = graph.index_patterns([
            [ababababcd, efghi],
            [abab, ababcdefghi],
            [ababab, abcdefghi],
        ]);
    } else {
        panic!("Inserting tokens failed!");
    }
    graph
}
#[derive(Default)]
pub struct GraphVis {
    graph: DiGraph<NodeVis, ()>,
    pub layout: Layout,
    handle: Option<Graph>,
}
impl GraphVis {
    pub fn set_graph(&mut self, graph: Graph) {
        self.handle = Some(graph);
        self.update();
    }
    fn handle(&self) -> Graph {
        self.handle.clone().expect("GraphVis not yet initialized!")
    }
    pub fn update(&mut self) {
        // todo reuse names in nodes
        let pg = self.handle().graph().to_petgraph();
        let node_indices: HashMap<_, _> =
            pg.nodes().map(|(idx, (key, _node))| (*key, idx)).collect();
        let old_node_indices: HashMap<_, _> = self
            .graph
            .nodes()
            .map(|(idx, node)| (node.key, idx))
            .collect();
        let new = pg.map(
            |idx, (key, node)| {
                if let Some(oid) = old_node_indices.get(key) {
                    let old = self.graph.node_weight(*oid).unwrap();
                    NodeVis::from_old(old, &node_indices, idx, node)
                } else {
                    NodeVis::new(self.handle(), &node_indices, idx, key, node)
                }
            },
            |_idx, _p| (),
        );
        self.graph = new;
    }
    pub fn edge_tip(ui: &mut Ui, source: &Pos2, target: &Pos2, size: f32) {
        let angle = (*target - *source).angle();
        let points = IntoIterator::into_iter([
            Vec2::new(0.0, 0.0),
            Vec2::angled(angle - 0.25 * PI),
            Vec2::angled(angle + 0.25 * PI),
        ])
        .map(|p| *target - p * size)
        .collect();
        ui.painter().add(Shape::convex_polygon(
            points,
            egui::Color32::WHITE,
            Stroke::new(1.0, egui::Color32::WHITE),
        ));
    }
    #[allow(unused)]
    pub fn edge(ui: &mut Ui, source: &Pos2, target: &Pos2) {
        ui.painter().add(Shape::line_segment(
            [*source, *target],
            Stroke::new(1.0, egui::Color32::WHITE),
        ));
        Self::edge_tip(ui, source, target, 10.0);
    }
    #[allow(clippy::many_single_char_names)]
    fn border_intersection_point(rect: &Rect, p: &Pos2) -> Pos2 {
        let p = *p;
        let c = rect.center();
        let v = p - c;
        let s = v.y / v.x;
        let h = rect.height();
        let w = rect.width();
        c + if -h / 2.0 <= s * w / 2.0 && s * w / 2.0 <= h / 2.0 {
            // intersects side
            if p.x > c.x {
                // right
                vec2(w / 2.0, w / 2.0 * s)
            } else {
                // left
                vec2(-w / 2.0, -w / 2.0 * s)
            }
        } else {
            // intersects top or bottom
            if p.y > c.y {
                // top
                vec2(h / (2.0 * s), h / 2.0)
            } else {
                // bottom
                vec2(-h / (2.0 * s), -h / 2.0)
            }
        }
    }
    pub fn show(&mut self, ui: &mut Ui) {
        //println!("{}", self.graph.vertex_count());
        let rects: HashMap<_, _> = self
            .graph
            .nodes()
            .map(|(idx, node)| {
                let response = node.show(ui, self).unwrap();
                (idx, response.rect)
            })
            .collect();
        self.graph.edge_references().for_each(|edge| {
            let a_pos = rects
                .get(&edge.source())
                .expect("No position for edge endpoint.")
                .center();
            let b = rects
                .get(&edge.target())
                .expect("No position for edge endpoint.");

            let p = Self::border_intersection_point(b, &a_pos);
            Self::edge(ui, &a_pos, &p);
        });
    }
}
#[allow(unused)]
#[derive(Clone)]
pub struct NodeVis {
    key: VertexKey<char>,
    idx: NodeIndex,
    name: String,
    data: VertexData,
    child_patterns: ChildPatternsVis,
    state: Arc<RwLock<NodeState>>,
    graph: Graph,
}
#[allow(unused)]
pub struct NodeState {
    split_lower: usize,
    split_upper: usize,
}
impl NodeState {
    pub fn new() -> Self {
        Self {
            split_lower: 1,
            split_upper: 7,
        }
    }
}
impl std::ops::Deref for NodeVis {
    type Target = VertexData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl NodeVis {
    pub fn new(
        graph: Graph,
        node_indices: &HashMap<VertexKey<char>, NodeIndex>,
        idx: NodeIndex,
        key: &VertexKey<char>,
        data: &VertexData,
    ) -> Self {
        Self::new_impl(
            graph,
            node_indices,
            idx,
            key,
            data,
            Arc::new(RwLock::new(NodeState::new())),
        )
    }
    pub fn from_old(
        old: &NodeVis,
        node_indices: &HashMap<VertexKey<char>, NodeIndex>,
        idx: NodeIndex,
        data: &VertexData,
    ) -> Self {
        Self::new_impl(
            old.graph.clone(),
            node_indices,
            idx,
            &old.key,
            data,
            old.state.clone(),
        )
    }
    pub fn new_impl(
        graph: Graph,
        node_indices: &HashMap<VertexKey<char>, NodeIndex>,
        idx: NodeIndex,
        key: &VertexKey<char>,
        data: &VertexData,
        state: Arc<RwLock<NodeState>>,
    ) -> Self {
        let (name, child_patterns) = {
            let graph = &*graph.graph();
            let name = graph.key_data_string(key, data);
            let child_patterns = Self::child_patterns_vis(graph, node_indices, data);
            (name, child_patterns)
        };
        Self {
            key: *key,
            graph,
            idx,
            name,
            data: data.clone(),
            child_patterns,
            state,
        }
    }
    #[allow(unused)]
    fn state(&self) -> std::sync::RwLockReadGuard<'_, NodeState> {
        self.state.read().unwrap()
    }
    #[allow(unused)]
    fn state_mut(&self) -> std::sync::RwLockWriteGuard<'_, NodeState> {
        self.state.write().unwrap()
    }
    fn child_patterns_vis<T: Tokenize + std::fmt::Display>(
        graph: &Hypergraph<T>,
        node_indices: &HashMap<VertexKey<T>, NodeIndex>,
        data: &VertexData,
    ) -> ChildPatternsVis {
        data.get_children()
            .iter()
            .map(|(&id, pat)| {
                (
                    id,
                    PatternVis::new(
                        pat.iter()
                            .map(|c| ChildVis::new(graph, node_indices, *c))
                            .collect(),
                    ),
                )
            })
            .collect()
    }
    pub fn child_patterns(&self, ui: &mut Ui, gvis: &GraphVis) -> Response {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(0.0);
            self.child_patterns.iter().for_each(|(_pid, cpat)| {
                let r = self.measure_pattern(ui, cpat, gvis);
                let height = r.rect.height();
                self.pattern(ui, cpat, Some(height), gvis);
            })
        })
        .response
    }
    fn measure_pattern(&self, ui: &mut Ui, pat: &PatternVis, gvis: &GraphVis) -> Response {
        let old_clip_rect = ui.clip_rect();
        let old_cursor = ui.cursor();
        ui.set_clip_rect(Rect::NOTHING);
        let r = self.pattern(ui, pat, None, gvis);
        ui.set_clip_rect(old_clip_rect);
        ui.set_cursor(old_cursor);
        r
    }
    fn pattern(
        &self,
        ui: &mut Ui,
        pat: &PatternVis,
        height: Option<f32>,
        gvis: &GraphVis,
    ) -> Response {
        ui.horizontal(|ui| {
            if let Some(height) = height {
                ui.set_min_height(height);
            }
            pat.pattern.iter().for_each(|child| {
                self.child_index(ui, child, gvis);
            })
        })
        .response
    }
    fn child_index(&self, ui: &mut Ui, child: &ChildVis, gvis: &GraphVis) -> Response {
        Frame::group(&Style::default())
            .margin((3.0, 3.0))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(0.0);
                //ui.set_min_width(UNIT_WIDTH * child.width as f32);
                if gvis.layout.is_nested() && child.child.width > 1 {
                    let node = gvis
                        .graph
                        .node_weight(child.idx)
                        .expect("Invalid NodeIndex in ChildVis!");
                    node.child_patterns(ui, gvis)
                } else {
                    ui.monospace(&child.name)
                }
            })
            .response
    }
    pub fn show(&self, ui: &mut Ui, gvis: &GraphVis) -> Option<Response> {
        Window::new(&format!("{}({})", self.name, self.idx.index()))
            //Window::new(&self.name)
            .vscroll(true)
            .default_width(80.0)
            .show(ui.ctx(), |ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(0.0);
                self.child_patterns(ui, gvis)
            })
            .map(|ir| ir.response)
    }
}
#[derive(Clone)]
struct ChildVis {
    name: String,
    child: Child,
    idx: NodeIndex,
}
impl std::ops::Deref for ChildVis {
    type Target = Child;
    fn deref(&self) -> &Self::Target {
        &self.child
    }
}
impl ChildVis {
    fn new<T: Tokenize + std::fmt::Display>(
        graph: &Hypergraph<T>,
        node_indices: &HashMap<VertexKey<T>, NodeIndex>,
        child: Child,
    ) -> Self {
        let key = graph.expect_vertex_key(child.index);
        let name = graph.index_string(child.get_index());
        let idx = *node_indices
            .get(key)
            .expect("Missing NodeIndex for VertexKey!");
        Self { name, child, idx }
    }
}
#[derive(Clone)]
struct PatternVis {
    pattern: Vec<ChildVis>,
}
impl PatternVis {
    fn new(pattern: Vec<ChildVis>) -> Self {
        Self { pattern }
    }
}
type ChildPatternsVis = Vec<(PatternId, PatternVis)>;
