//! Central panel with tabs (Graph, Inserter).

use eframe::egui::{self, Ui};
use strum::IntoEnumIterator;

use super::{App, CentralTab};
use crate::algorithm::Algorithm;

impl App {
    pub(crate) fn central_panel(
        &mut self,
        ctx: &egui::Context,
    ) {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                // Tab bar
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.selected_tab,
                        CentralTab::Graph,
                        "📊 Graph",
                    );
                    ui.selectable_value(
                        &mut self.selected_tab,
                        CentralTab::Inserter,
                        "✏ Inserter",
                    );
                });
                ui.separator();

                // Get viewport rect for constraining windows
                let viewport_rect = ui.available_rect_before_wrap();

                match self.selected_tab {
                    CentralTab::Graph => {
                        if let Some(mut vis) = self.vis_mut() {
                            vis.show(ui)
                        }
                    }
                    CentralTab::Inserter => {
                        self.show_inserter_tab(ui, viewport_rect);
                    }
                }

                egui::warn_if_debug_build(ui);
            })
            .response
            .context_menu(|ui| self.context_menu(ui));
    }

    fn show_inserter_tab(
        &mut self,
        ui: &mut Ui,
        _viewport_rect: egui::Rect,
    ) {
        ui.heading("Text Inserter");
        ui.add_space(10.0);

        // Show currently selected algorithm
        ui.horizontal(|ui| {
            ui.label("Algorithm:");
            egui::ComboBox::from_id_salt("inserter_algorithm")
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
        });

        ui.add_space(5.0);
        ui.label(self.selected_algorithm.description());

        ui.add_space(15.0);
        ui.separator();
        ui.add_space(10.0);

        ui.label("Input texts:");
        ui.add_space(5.0);

        if let Some(mut read_ctx) = self.ctx_mut() {
            let texts = &mut read_ctx.graph_mut().insert_texts;
            let mut to_remove = None;

            for (i, text) in texts.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(text).desired_width(400.0));
                    if ui.button("✖").on_hover_text("Remove").clicked() {
                        to_remove = Some(i);
                    }
                });
            }

            if let Some(idx) = to_remove {
                texts.remove(idx);
            }

            ui.add_space(5.0);
            if ui.button("+ Add Text").clicked() {
                texts.push(String::new());
            }
        }

        ui.add_space(15.0);
        ui.separator();
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            let is_running = self.read_task.is_some();

            if ui
                .add_enabled(!is_running, egui::Button::new("▶ Run"))
                .clicked()
            {
                self.start_read();
            }

            if is_running {
                if ui.button("⏹ Cancel").clicked() {
                    self.abort();
                }
                ui.spinner();
                ui.label("Processing...");
            }
        });
    }
}
