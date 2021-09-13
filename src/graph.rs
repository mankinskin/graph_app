use eframe::egui::{self, pos2, vec2, Frame, Pos2, Response, Shape, Stroke, Style, Ui, Vec2, Window, Rect};
#[allow(unused)]
use petgraph::{
    visit::EdgeRef,
    graph::{
        NodeIndex,
        DiGraph,
    },
};
use seqraph::{
    hypergraph::{Child, Hypergraph, VertexData, VertexKey, *},
    token::{Token, Tokenize},
};
use std::collections::{
    HashMap,
};
use std::f32::consts::PI;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, PartialEq)]
pub enum Layout {
    Graph,
    Nested,
    GraphAndNested,
}
impl Layout {
    pub fn is_graph(&self) -> bool {
        match self {
            Self::Graph | Self::GraphAndNested => true,
            _ => false,
        }
    }
    pub fn is_nested(&self) -> bool {
        match self {
            Self::Nested | Self::GraphAndNested => true,
            _ => false,
        }
    }
}
impl Default for Layout {
    fn default() -> Self {
        Self::Graph
    }
}
pub struct Graph {
    #[allow(unused)]
    #[cfg_attr(feature = "persistence", serde(skip))]
    graph: Hypergraph<char>,
    vis: GraphVis,
}
impl Graph {
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
            let abcdefghi = graph.insert_patterns([
                vec![abcd, efghi],
                vec![ab, cdef, ghi],
            ]);
            let aba = graph.insert_pattern([ab, a]);
            let abab = graph.insert_patterns([
                [aba, b],
                [ab, ab],
            ]);
            let ababab = graph.insert_patterns([
                [abab, ab],
                [ab, abab],
            ]);
            let ababcd = graph.insert_patterns([
                [ab, abcd],
                [aba, bcd],
                [abab, cd],
            ]);
            let ababababcd = graph.insert_patterns([
                [ababab, abcd],
                [abab, ababcd],
            ]);
            let ababcdefghi = graph.insert_patterns([
                [ab, abcdefghi],
                [ababcd, efghi],
            ]);
            let _ababababcdefghi = graph.insert_patterns([
                [ababababcd, efghi],
                [abab, ababcdefghi],
                [ababab, abcdefghi]
            ]);
        } else {
            panic!("Inserting tokens failed!");
        }
        graph
    }
    pub fn split(&mut self) {
        let (ababababcdefghi, _, _) = self.graph.find_sequence("ababababcdefghi".chars()).expect("not found");
        let (left, right) = self.graph.split_index_at_pos(ababababcdefghi, NonZeroUsize::new(7).unwrap());
        self.vis = GraphVis::new(&self.graph);
    }
    pub fn new() -> Self {
        let graph = Self::build_hypergraph();
        Self {
            vis: GraphVis::new(&graph),
            graph,
        }
    }
    pub fn get_layout_mut(&mut self) -> &mut Layout {
        &mut self.vis.layout
    }
    pub fn show(&mut self, ui: &mut Ui) {
        self.vis.show(ui)
    }
}
pub struct GraphVis {
    #[cfg_attr(feature = "persistence", serde(skip))]
    graph: DiGraph<Node, ()>,
    pub layout: Layout,
}
impl std::ops::Deref for GraphVis {
    type Target = DiGraph<Node, ()>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl GraphVis {
    pub fn new(g: &Hypergraph<char>) -> Self {
        // todo reuse names in nodes
        let pg = g.to_petgraph();
        let node_indices: HashMap<_, _> = pg.nodes().map(|(idx, (key, _))| (*key, idx)).collect();
        let graph =
            pg.map(|idx, (key, node)| Node::new(g, &node_indices, idx, &key, node),
                |_idx, _p| ()
            );
        Self {
            graph,
            layout: Default::default()
        }
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
                let response = node.clone().show(ui, &self).unwrap();
                (idx, response.rect)
            })
            .collect();
        self.graph.edge_references()
            .for_each(|edge| {
                let a_pos = rects.get(&edge.source()).expect("No position for edge endpoint.").center();
                let b = rects.get(&edge.target()).expect("No position for edge endpoint.");

                let p = Self::border_intersection_point(&b, &a_pos);
                Self::edge(ui, &a_pos, &p);
            });
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
        let idx = node_indices.get(key).expect("Missing NodeIndex for VertexKey!").clone();
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
#[allow(unused)]
#[derive(Clone)]
pub struct Node {
    idx: NodeIndex,
    name: String,
    data: VertexData,
    child_patterns: ChildPatternsVis,
}
impl std::ops::Deref for Node {
    type Target = VertexData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl Node {
    pub fn new<T: Tokenize + std::fmt::Display>(
        graph: &Hypergraph<T>,
        node_indices: &HashMap<VertexKey<T>, NodeIndex>,
        idx: NodeIndex,
        key: &VertexKey<T>,
        data: &VertexData,
    ) -> Self {
        let name = graph.key_data_string(key, data);
        let child_patterns = Self::child_patterns_vis(graph, node_indices, data);
        Self {
            idx,
            name,
            data: data.clone(),
            child_patterns,
        }
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
                            .map(|c| ChildVis::new(graph, node_indices, c.clone()))
                            .collect(),
                    ),
                )
            })
            .collect()
    }
    pub fn child_patterns(&self, ui: &mut Ui, graph: &GraphVis) -> Response {
        ui.vertical(
            |ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(0.0);
            self.child_patterns
                .iter()
                .for_each(|(_pid, cpat)| {
                    let r = self.measure_pattern(ui, cpat, graph);
                    let height = r.rect.height();
                    self.pattern(ui, cpat, graph, Some(height));
                })
        })
        .response
    }
    fn measure_pattern(&self, ui: &mut Ui, pat: &PatternVis, graph: &GraphVis) -> Response {
        let old_clip_rect = ui.clip_rect();
        let old_cursor = ui.cursor();
        ui.set_clip_rect(Rect::NOTHING);
        let r = self.pattern(ui, pat, graph, None);
        ui.set_clip_rect(old_clip_rect);
        ui.set_cursor(old_cursor);
        r
    }
    fn pattern(&self, ui: &mut Ui, pat: &PatternVis, graph: &GraphVis, height: Option<f32>) -> Response {
        ui.horizontal(|ui| {
            //ui.spacing_mut().item_spacing = Vec2::splat(0.0);
            if let Some(height) = height {
                ui.set_min_height(height);
            }
            pat.pattern
                .iter()
                .for_each(|child| {
                    self.child_index(ui, child, graph);
                })
        })
        .response
    }
    fn child_index(&self, ui: &mut Ui, child: &ChildVis, graph: &GraphVis) -> Response {
        Frame::group(&Style::default())
            .margin((3.0, 3.0))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(0.0);
                //ui.set_min_width(UNIT_WIDTH * child.width as f32);
                if graph.layout.is_nested() && child.child.width > 1 {
                    let node = graph.node_weight(child.idx).expect("Invalid NodeIndex in ChildVis!");
                    node.child_patterns(ui, graph)
                } else {
                    ui.monospace(format!("{}", child.name))
                }
            })
            .response
    }
    pub fn show(self, ui: &mut Ui, graph: &GraphVis) -> Option<Response> {
        Window::new(&format!("{}({})", self.name, self.idx.index()))
            .vscroll(true)
            .default_width(80.0)
            .show(ui.ctx(), |ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(0.0);
                self.child_patterns(ui, graph)
            })
            .map(|ir| ir.response)
    }
}
