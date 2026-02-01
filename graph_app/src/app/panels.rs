//! Panel rendering (top, left, right, bottom).

use eframe::egui;
use strum::IntoEnumIterator;

use super::App;
use crate::algorithm::Algorithm;

impl App {
    pub(crate) fn top_panel(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        egui::TopBottomPanel::top("top_menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // File menu
                ui.menu_button("File", |ui| {
                    if ui.button("Open...").clicked() {
                        self.open_file_dialog();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // Edit menu
                ui.menu_button("Edit", |ui| {
                    ui.label("(No edit actions)");
                });

                // View menu
                ui.menu_button("View", |ui| {
                    if ui
                        .checkbox(&mut self.left_panel_open, "Left Panel")
                        .clicked()
                    {
                        ui.close();
                    }
                    if ui
                        .checkbox(&mut self.right_panel_open, "Right Panel")
                        .clicked()
                    {
                        ui.close();
                    }
                    if ui
                        .checkbox(&mut self.bottom_panel_open, "Bottom Panel")
                        .clicked()
                    {
                        ui.close();
                    }
                    ui.separator();
                    if ui
                        .checkbox(&mut self.inserter_open, "Inserter Window")
                        .clicked()
                    {
                        ui.close();
                    }
                    if ui
                        .checkbox(&mut self.settings_open, "Settings Window")
                        .clicked()
                    {
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("New Tab").clicked() {
                        self.create_new_tab();
                        ui.close();
                    }
                });

                // Right-aligned items
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("⚙").on_hover_text("Settings").clicked()
                        {
                            self.settings_open = !self.settings_open;
                        }
                    },
                );
            });
        });
    }

    pub(crate) fn left_panel(
        &mut self,
        ctx: &egui::Context,
    ) {
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(200.0)
            .min_width(150.0)
            .show_animated(ctx, self.left_panel_open, |ui| {
                ui.heading("Tools");
                ui.separator();

                // Algorithm selection
                ui.label("Algorithm:");
                egui::ComboBox::from_id_salt("left_panel_algorithm")
                    .selected_text(self.selected_algorithm.to_string())
                    .show_ui(ui, |ui| {
                        for algorithm in Algorithm::iter() {
                            ui.selectable_value(
                                &mut self.selected_algorithm,
                                algorithm,
                                algorithm.to_string(),
                            );
                        }
                    });

                ui.add_space(10.0);
                ui.label(self.selected_algorithm.description());

                ui.add_space(20.0);
                ui.separator();

                // Insert controls
                ui.heading("Insert");
                if let Some(mut read_ctx) = self.ctx_mut() {
                    for text in &mut read_ctx.graph_mut().insert_texts {
                        ui.text_edit_singleline(text);
                    }
                    if ui.button("+ Add Text").clicked() {
                        read_ctx.graph_mut().insert_texts.push(String::new());
                    }
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("▶ Run").clicked() && self.read_task.is_none()
                    {
                        self.start_read();
                    }
                    if self.read_task.is_some()
                        && ui.button("⏹ Cancel").clicked()
                    {
                        self.abort();
                    }
                });
            });
    }

    pub(crate) fn right_panel(
        &mut self,
        ctx: &egui::Context,
    ) {
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(200.0)
            .min_width(150.0)
            .show_animated(ctx, self.right_panel_open, |ui| {
                ui.heading("Properties");
                ui.separator();

                // Show graph info
                if let Some(read_ctx) = self.ctx() {
                    let graph = read_ctx.graph();
                    if let Some(graph_ref) = graph.try_read() {
                        ui.label(format!(
                            "Vertices: {}",
                            graph_ref.vertex_count()
                        ));
                    }
                }

                ui.add_space(20.0);
                ui.separator();
                ui.heading("Selection");
                ui.label("No selection");
            });
    }

    pub(crate) fn bottom_panel(
        &mut self,
        ctx: &egui::Context,
    ) {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .exact_height(28.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Show task status
                    if self.read_task.is_some() {
                        ui.spinner();
                        ui.label("Processing...");

                        // Show progress if available
                        if let Some(read_ctx) = self.ctx() {
                            if let Some(status) = read_ctx.status() {
                                let status = status.read().unwrap();
                                ui.separator();
                                ui.label(format!("Pass: {:?}", status.pass()));
                                let progress = *status.steps() as f32
                                    / *status.steps_total() as f32;
                                ui.add(
                                    egui::ProgressBar::new(progress)
                                        .desired_width(150.0)
                                        .show_percentage(),
                                );
                            }
                        }
                    } else {
                        ui.label("Ready");
                    }

                    // Debug build warning on the right
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            egui::warn_if_debug_build(ui);
                        },
                    );
                });
            });
    }
}
