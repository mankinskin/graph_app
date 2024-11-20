use eframe::{egui::{
    self, ThemePreference, Ui
}, CreationContext};
use ngrams::graph::{vocabulary::ProcessStatus, Status, StatusHandle};
use seqraph::graph::HypergraphRef;
#[cfg(feature = "persistence")]
use serde::*;
use tokio::task::JoinHandle;

use crate::examples::{build_graph1, build_graph2, build_graph3};
#[allow(unused)]
use crate::graph::*;
use strum::IntoEnumIterator;
use std::{future::Future, sync::{Arc, RwLock}};
use derive_more::{Deref, DerefMut};

#[derive(Deref, DerefMut, Debug)]
struct ShowStatus<'a>(&'a ngrams::graph::Status);
impl ShowStatus<'_>  {
    fn show(&self, ctx: &egui::Context) {
        egui::Window::new("Status").show(ctx, |ui| {
            ui.label(format!("Text: \"{}\"", self.insert_text));
            ProcessStatus::iter().skip(1)
                .for_each(|pass| self.show_pass(ui, pass))
        });
    }
    fn show_pass(&self, ui: &mut Ui, pass: ProcessStatus) {
        let checked = *self.pass() > pass || (
            pass == ProcessStatus::Finished && *self.pass() == ProcessStatus::Finished
        );
        let percent = (*self.steps() as f32 / *self.steps_total() as f32 * 100.0) as u32;
        let text = format!(
            "{:?}{}",
            pass,
            (pass == ProcessStatus::Finished)
                .then(|| String::new())
                .unwrap_or_else(||
                    checked.then_some(100)
                        .or_else(||
                            (pass == ProcessStatus::iter().skip_while(|i| *i < *self.pass()).next().unwrap())
                                // is next
                                .then_some(percent)
                        )
                        .map(|p| format!(" Pass: {}%", p))
                        .unwrap_or(String::from(" Pass"))
                )
        );
        ui.checkbox(&mut (checked.clone()), text);
    }
}


#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App
{
    #[allow(unused)]
    graph_file: Option<std::path::PathBuf>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    graph: Graph,
    inserter: bool,
    status: Option<ngrams::graph::StatusHandle>,
    read_task: Option<JoinHandle<()>>,
}

impl Default for App {
    fn default() -> Self
    {
        Self {
            graph_file: None,
            graph: Graph::default(),
            inserter: true,
            read_task: None,
            status: None,
        }
    }
}
impl App
{
    pub fn new(creation_context: &CreationContext<'_>) -> Self {
        creation_context.egui_ctx.set_theme(ThemePreference::Dark);
        Self
        {
            ..Default::default()
        }
    }
    #[allow(unused)]
    pub fn from_graph_ref(graph: HypergraphRef) -> Self
    {
        Self
        {
            graph_file: None,
            graph: Graph::new_from_graph_ref(graph),
            inserter: true,
            read_task: None,
            status: None,
        }
    }
    pub fn context_menu(
        &mut self,
        ui: &mut Ui,
    )
    {
        ui.horizontal(|ui| {
            ui.label("Quick Insert:");
            ui.text_edit_singleline(&mut self.graph.insert_text);
            if ui.button("Go").clicked()
            {
                let insert_text = self.graph.insert_text.clone();
                self.graph.read_text(insert_text, ui.ctx());
                self.graph.insert_text = String::new();
                ui.close_menu();
            }
        });
        if ui.button("Open Inserter").clicked()
        {
            self.inserter = true;
            ui.close_menu();
        }
        {
            let mut vis = self.graph.vis_mut();
            ui.menu_button("Layout", |ui| {
                ui.radio_value(&mut vis.layout, Layout::Graph, "Graph")
                    .clicked();
                ui.radio_value(&mut vis.layout, Layout::Nested, "Nested")
                    .clicked();
            });
        }
        ui.menu_button("Load preset...", |ui| {
            if ui.button("Graph 1").clicked()
            {
                self.graph.set_graph(build_graph1());
                ui.close_menu();
            }
            if ui.button("Graph 2").clicked()
            {
                self.graph.set_graph(build_graph2());
                ui.close_menu();
            }
            if ui.button("Graph 3").clicked()
            {
                self.graph.set_graph(build_graph3());
                ui.close_menu();
            }
        });
        if ui.button("Clear").clicked()
        {
            self.graph.clear();
            ui.close_menu();
        }
    }
    #[allow(unused)]
    async fn read_graph_file(
        graph: Arc<RwLock<Option<String>>>,
        file: &rfd::FileHandle,
    )
    {
        let content = file.read().await;
        match std::str::from_utf8(&content[..])
        {
            Ok(content) =>
            {}
            Err(err) =>
            {
                rfd::AsyncMessageDialog::default()
                    .set_description(format!("{}", err))
                    .show()
                    .await;
            }
        }
    }
    #[allow(unused)]
    fn open_file_dialog(&mut self)
    {
        // open graph file
        let mut dialog = rfd::AsyncFileDialog::default();
        let res = std::env::current_dir();
        if let Ok(current_dir) = &res
        {
            dialog = dialog.set_directory(current_dir);
        }
    }
    fn top_panel(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    )
    {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Actions...", |ui| self.context_menu(ui));
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("Quit").clicked()
                        {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close)
                        }
                    },
                )
            })
        });
    }
    fn central_panel(
        &mut self,
        ctx: &egui::Context,
    )
    {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                self.graph.show(ui);
                //tracing_egui::show(ui.ctx(), &mut true);
                egui::warn_if_debug_build(ui);
            })
            .response
            .context_menu(|ui| self.context_menu(ui));
    }
}
impl eframe::App for App
{
    //fn max_size_points(&self) -> egui::Vec2
    //{
    //    (f32::INFINITY, f32::INFINITY).into()
    //}
    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(
        &mut self,
        storage: &mut dyn epi::Storage,
    )
    {
        epi::set_value(storage, epi::APP_KEY, self);
    }
    fn update(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
    )
    {
        self.top_panel(ctx, frame);
        self.central_panel(ctx);
        if self.inserter
        {
            egui::Window::new("Inserter").show(ctx, |ui| {
                ui.text_edit_multiline(&mut self.graph.insert_text);
                if ui.button("Insert").clicked()
                {
                    let insert_text = std::mem::take(&mut self.graph.insert_text);
                    let graph = self.graph.graph.clone();
                    let labels = self.graph.labels.clone();
                    let status = StatusHandle::from(Status::new(insert_text.clone()));
                    self.status = Some(status.clone());
                    std::thread::spawn(move || {
                        let res = ngrams::graph::parse_corpus(ngrams::graph::Corpus::new("", [insert_text]), status);

                        *graph.write().unwrap() = res.graph;
                        *labels.write().unwrap() = res.labels;
                    });
                }
            });
        }
        if let Some(status) = self.status.as_ref()
        {
            ShowStatus(&*status.read().unwrap()).show(ctx);
        }
    }
    fn on_exit(&mut self, _ctx: Option<&eframe::glow::Context>)
    {
        if let Some(handle) = self.read_task.take()
        {
            println!("abort");
            handle.abort();
        }
    }
}
#[allow(unused)]
#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F)
{
    tokio::spawn(f);
}
#[allow(unused)]
#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F)
{
    wasm_bindgen_futures::spawn_local(f);
}
