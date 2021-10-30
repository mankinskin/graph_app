use eframe::{egui::{self, vec2, DragValue, Frame, Pos2, Response, Shape, Stroke, Style, Ui, Vec2, Window, Rect}, epi};
#[allow(unused)]
use petgraph::{
    visit::EdgeRef,
    graph::{
        NodeIndex,
        DiGraph,
    },
};
use seqraph::{
    hypergraph::*,
    token::{Token, Tokenize},
};
use std::collections::{
    HashMap,
};
use std::f32::consts::PI;
use std::num::NonZeroUsize;
use std::sync::{
    Arc,
    RwLock,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Layout {
    Graph,
    Nested,
    GraphAndNested,
}
impl Layout {
    pub fn is_graph(&self) -> bool {
        matches!(self, Self::Graph | Self::GraphAndNested)
    }
    pub fn is_nested(&self) -> bool {
        matches!(self, Self::Nested | Self::GraphAndNested)
    }
}
impl Default for Layout {
    fn default() -> Self {
        Self::Graph
    }
}
#[derive(Clone)]
pub struct Graph {
    graph: Arc<RwLock<Hypergraph<char>>>,
    vis: Arc<RwLock<GraphVis>>,
}
impl Graph {
    pub fn new() -> Self {
        //let graph = Self::build_hypergraph();
        let graph = Arc::new(RwLock::new(Default::default()));
        let vis = Arc::new(RwLock::new(GraphVis::default()));
        let new = Self {
            graph,
            vis,
        };
        let g = new.clone();
        new.vis_mut().set_graph(g);
        new
    }
    pub(crate) fn graph(&self) -> std::sync::RwLockReadGuard<'_, Hypergraph<char>> {
        self.graph.read().unwrap()
    }
    pub(crate) fn graph_mut(&self) -> std::sync::RwLockWriteGuard<'_, Hypergraph<char>> {
        self.graph.write().unwrap()
    }
    pub(crate) fn vis(&self) -> std::sync::RwLockReadGuard<'_, GraphVis> {
        self.vis.read().unwrap()
    }
    pub(crate) fn vis_mut(&self) -> std::sync::RwLockWriteGuard<'_, GraphVis> {
        self.vis.write().unwrap()
    }
    fn build_hypergraph() -> Hypergraph<char> {
        let mut graph = Hypergraph::default();
        if let [a, b, c, d, e, f, g, h, i] = graph.insert_tokens(
            [
                Token::Element('a'),
                Token::Element('b'),
                Token::Element('c'),
                Token::Element('d'),
                Token::Element('e'),
                Token::Element('f'),
                Token::Element('g'),
                Token::Element('h'),
                Token::Element('i'),
            ])[..] {
            // abcdefghi
            // ababababcdbcdefdefcdefefghefghghi
            // ->
            // abab ab abcdbcdefdefcdefefghefghghi
            // ab abab abcdbcdefdefcdefefghefghghi

            // abcdbcdef def cdef efgh efgh ghi

            // abcd b cdef
            // abcd bcd ef

            // ab cd
            // abc d
            // a bcd

            let ab = graph.insert_pattern([a, b]);
            let bc = graph.insert_pattern([b, c]);
            let ef = graph.insert_pattern([e, f]);
            let def = graph.insert_pattern([d, ef]);
            let cdef = graph.insert_pattern([c, def]);
            let gh = graph.insert_pattern([g, h]);
            let efgh = graph.insert_pattern([ef, gh]);
            let ghi = graph.insert_pattern([gh, i]);
            let abc = graph.insert_patterns([
                [ab, c],
                [a, bc],
            ]);
            let cd = graph.insert_pattern([c, d]);
            let bcd = graph.insert_patterns([
                [bc, d],
                [b, cd],
            ]);
            let abcd = graph.insert_patterns([
                [abc, d],
                [a, bcd],
            ]);
            let efghi = graph.insert_patterns([
                [efgh, i],
                [ef, ghi],
            ]);
            //let abcdefghi = graph.insert_patterns([
            //    vec![abcd, efghi],
            //    vec![ab, cdef, ghi],
            //]);
            //let aba = graph.insert_pattern([ab, a]);
            //let abab = graph.insert_patterns([
            //    [aba, b],
            //    [ab, ab],
            //]);
            //let ababab = graph.insert_patterns([
            //    [abab, ab],
            //    [ab, abab],
            //]);
            //let ababcd = graph.insert_patterns([
            //    [ab, abcd],
            //    [aba, bcd],
            //    [abab, cd],
            //]);
            //let ababababcd = graph.insert_patterns([
            //    vec![ababab, abcd],
            //    vec![abab, ababcd],
            //    vec![ab, ababab, cd],
            //]);
            //let ababcdefghi = graph.insert_patterns([
            //    [ab, abcdefghi],
            //    [ababcd, efghi],
            //]);
            //let _ababababcdefghi = graph.insert_patterns([
            //    [ababababcd, efghi],
            //    [abab, ababcdefghi],
            //    [ababab, abcdefghi],
            //]);
        } else {
            panic!("Inserting tokens failed!");
        }
        graph
    }
    pub fn split_range(&self, index: VertexIndex, lower: NonZeroUsize, upper: NonZeroUsize) {
        let lower = lower.get();
        let upper = upper.get();
        let _res = self.graph_mut().index_subrange(index, lower..upper);
    }
    pub fn split(&self, index: VertexIndex, pos: NonZeroUsize) {
        let _res = self.graph_mut().split_index(index, pos);
    }
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    pub fn read(&self, text: impl ToString) {
        self.graph_mut().read_sequence(text.to_string().chars());
    }
    pub fn show(&self, ui: &mut Ui) {
        self.vis_mut().update();
        self.vis_mut().show(ui);
    }
}
#[derive(Default)]
pub struct GraphVis {
    graph: DiGraph<NodeVis, ()>,
    pub layout: Layout,
    handle: Option<Graph>,
}
//impl std::ops::Deref for GraphVis {
//    type Target = DiGraph<NodeVis, ()>;
//    fn deref(&self) -> &Self::Target {
//        &self.graph
//    }
//}
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
        let node_indices: HashMap<_, _> = pg.nodes().map(|(idx, (key, _node))| (*key, idx)).collect();
        let old_node_indices: HashMap<_, _> = self.graph.nodes().map(|(idx, node)| (node.key, idx)).collect();
        let new = pg.map(|idx, (key, node)|
                    if let Some(oid) = old_node_indices.get(key) {
                        let old = self.graph.node_weight(*oid).unwrap();
                        NodeVis::from_old(
                            old,
                            &node_indices,
                            idx,
                            node,
                        )
                    } else {
                        NodeVis::new(
                            self.handle(),
                            &node_indices,
                            idx,
                            key,
                            node,
                        )
                    },
                |_idx, _p| ()
            );
        self.graph = new;
    }
    pub fn edge_tip(ui: &mut Ui, source: &Pos2, target: &Pos2, size: f32) {
        let angle = (*target - *source).angle();
        let points = IntoIterator::into_iter(
            [
                Vec2::new(0.0, 0.0),
                Vec2::angled(angle - 0.25*PI),
                Vec2::angled(angle + 0.25*PI)
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
        let s = v.y/v.x;
        let h = rect.height();
        let w = rect.width();
        c + if -h/2.0 <= s*w/2.0 && s*w/2.0 <= h/2.0 { // intersects side
            if p.x > c.x { // right
                vec2(w/2.0, w/2.0*s)
            } else { // left
                vec2(-w/2.0, -w/2.0*s)
            }
        } else { // intersects top or bottom
            if p.y > c.y { // top
                vec2(h/(2.0*s), h/2.0)
            } else { // bottom
                vec2(-h/(2.0*s), -h/2.0)
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
        self.graph.edge_references()
            .for_each(|edge| {
                let a_pos = rects.get(&edge.source()).expect("No position for edge endpoint.").center();
                let b = rects.get(&edge.target()).expect("No position for edge endpoint.");

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
    fn state(&self) -> std::sync::RwLockReadGuard<'_, NodeState>{
        self.state.read().unwrap()
    }
    fn state_mut(&self) -> std::sync::RwLockWriteGuard<'_, NodeState>{
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
        ui.vertical(
            |ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(0.0);
            self.child_patterns
                .iter()
                .for_each(|(_pid, cpat)| {
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
    fn pattern(&self, ui: &mut Ui, pat: &PatternVis, height: Option<f32>, gvis: &GraphVis) -> Response {
        ui.horizontal(|ui| {
            if let Some(height) = height {
                ui.set_min_height(height);
            }
            pat.pattern
                .iter()
                .for_each(|child| {
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
                    let node = gvis.graph.node_weight(child.idx).expect("Invalid NodeIndex in ChildVis!");
                    node.child_patterns(ui, gvis)
                } else {
                    ui.monospace(&child.name)
                }
            })
            .response
    }
    pub fn context_menu(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let mut state = self.state_mut();
            ui.add(DragValue::new(&mut state.split_lower));
            ui.add(DragValue::new(&mut state.split_upper));
            state.split_upper = state.split_lower.max(state.split_upper);
            if ui.button("Split").clicked() {
                match (NonZeroUsize::new(state.split_lower), NonZeroUsize::new(state.split_upper)) {
                    (Some(lower), Some(upper)) => self.graph.split_range(self.index, lower, upper),
                    (None, Some(single)) |
                    (Some(single), None) => self.graph.split(self.index, single),
                    (None, None) => {},
                }
                ui.close_menu();
            }
        });
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
            .map(|ir| ir.response.context_menu(|ui| self.context_menu(ui)))
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
        let idx = *node_indices.get(key).expect("Missing NodeIndex for VertexKey!");
        Self { name, child, idx}
    }
}
#[derive(Clone)]
struct PatternVis {
    width: TokenPosition,
    pattern: Vec<ChildVis>,
}
impl PatternVis {
    fn new(pattern: Vec<ChildVis>) -> Self {
        let width = pattern_width((&pattern).iter().map(|c| &c.child).collect::<Vec<_>>());
        Self { width, pattern }
    }
}
type ChildPatternsVis = Vec<(PatternId, PatternVis)>;