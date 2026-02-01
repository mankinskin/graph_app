use std::ops::Range;

use context_trace::graph::{
    vertex::{
        data::VertexData,
        has_vertex_index::HasVertexIndex,
        pattern::id::PatternId,
        token::Token,
        wide::Wide,
    },
    Hypergraph,
};
use eframe::egui::{
    self,
    Frame,
    Response,
    Style,
    Ui,
    Vec2,
};
use indexmap::IndexMap;

#[derive(Clone, Debug)]
struct ChildVis {
    name: String,
    child: Token,
}

impl std::ops::Deref for ChildVis {
    type Target = Token;
    fn deref(&self) -> &Self::Target {
        &self.child
    }
}
pub struct ChildResponse {
    #[allow(unused)]
    pub response: Response,
    pub range: Option<Range<usize>>,
}

impl ChildVis {
    fn new(
        graph: &Hypergraph,
        child: Token,
    ) -> Self {
        let name = graph.index_string(child.vertex_index());
        Self { name, child }
    }
    fn show(
        &self,
        ui: &mut Ui,
    ) -> ChildResponse {
        let response =
            Frame::group(&Style::default())
                .inner_margin(3.0)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(0.0);
                    ui.monospace(&self.name).selected_char_range()
                });
        ChildResponse {
            response: response.response,
            range: response.inner,
        }
    }
}

#[derive(Clone, Debug)]
struct PatternVis {
    pattern: Vec<ChildVis>,
}
pub struct PatternResponse {
    pub response: Response,
    pub ranges: Vec<Option<Range<usize>>>,
}
impl PatternVis {
    fn new(pattern: Vec<ChildVis>) -> Self {
        Self { pattern }
    }
    fn measure(
        &self,
        ui: &mut Ui,
    ) -> PatternResponse {
        let old_clip_rect = ui.clip_rect();
        let old_cursor = ui.cursor();
        ui.set_clip_rect(egui::Rect::NOTHING);
        let r = self.show(ui, None);
        ui.set_clip_rect(old_clip_rect);
        ui.set_cursor(old_cursor);
        r
    }
    fn show(
        &self,
        ui: &mut Ui,
        height: Option<f32>,
    ) -> PatternResponse {
        let response = ui.horizontal(|ui| {
            if let Some(height) = height {
                ui.set_min_height(height);
            }
            self.pattern
                .iter()
                .map(|child| child.show(ui).range)
                .collect()
        });
        PatternResponse {
            response: response.response,
            ranges: response.inner,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChildPatternsVis {
    patterns: Vec<(PatternId, PatternVis)>,
}
#[allow(unused)]
#[derive(Clone, Debug)]
pub struct ChildPatternsResponse {
    pub response: Response,
    pub ranges: IndexMap<PatternId, Range<usize>>,
}

impl ChildPatternsVis {
    pub fn new(
        graph: &Hypergraph,
        data: &VertexData,
    ) -> Self {
        Self {
            patterns: data
                .child_patterns()
                .iter()
                .map(|(&id, pat)| {
                    (
                        id,
                        PatternVis::new(
                            pat.iter()
                                .map(|c| ChildVis::new(graph, *c))
                                .collect(),
                        ),
                    )
                })
                .collect(),
        }
    }
    
    /// Returns the number of patterns in this node
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
    
    /// Returns the pattern strings for display
    pub fn pattern_strings(&self) -> Vec<String> {
        self.patterns
            .iter()
            .map(|(_pid, pat)| {
                pat.pattern
                    .iter()
                    .map(|c| c.name.clone())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect()
    }
    
    /// Returns patterns as vectors of child names for framed display
    pub fn patterns_as_children(&self) -> Vec<Vec<String>> {
        self.patterns
            .iter()
            .map(|(_pid, pat)| {
                pat.pattern
                    .iter()
                    .map(|c| c.name.clone())
                    .collect()
            })
            .collect()
    }

    /// Returns patterns with (name, vertex_index) pairs for edge connections
    pub fn patterns_with_indices(&self) -> Vec<Vec<(String, usize)>> {
        self.patterns
            .iter()
            .map(|(_pid, pat)| {
                pat.pattern
                    .iter()
                    .map(|c| (c.name.clone(), *c.child.vertex_index()))
                    .collect()
            })
            .collect()
    }
    
    fn find_selected_range(
        &self,
        pattern: &[ChildVis],
        ranges: Vec<Option<Range<usize>>>,
    ) -> Option<Range<usize>> {
        pattern
            .iter()
            .scan(0, |acc, c| {
                let prev = *acc;
                *acc += c.child.width().0;
                Some(prev)
            })
            .zip(ranges)
            .fold(None as Option<Range<usize>>, |acc, (off, r)| {
                let o = r.map(|r| (r.start + off)..(r.end + off));
                acc.as_ref()
                    .zip(o.as_ref())
                    .map(|(acc, o)| acc.start.min(o.start)..acc.end.max(o.end))
                    .or(acc)
                    .or(o)
            })
    }
    pub fn show(
        &self,
        ui: &mut Ui,
    ) -> ChildPatternsResponse {
        let response = ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(0.0);
            self.patterns
                .iter()
                .filter_map(|(pid, cpat)| {
                    let r = cpat.measure(ui);
                    let height = r.response.rect.height();
                    let ranges = cpat.show(ui, Some(height)).ranges;
                    self.find_selected_range(&cpat.pattern, ranges)
                        .map(|r| (*pid, r))
                })
                .collect::<IndexMap<_, _>>()
        });
        ChildPatternsResponse {
            response: response.response,
            ranges: response.inner,
        }
    }
}
