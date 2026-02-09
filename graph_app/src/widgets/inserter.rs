//! Inserter widget for text input and graph operations.

use eframe::egui::{
    self,
    Ui,
};
use strum::IntoEnumIterator;

use crate::algorithm::Algorithm;

/// Response from the inserter widget.
#[derive(Default)]
pub(crate) struct InserterResponse {
    /// User clicked the run button.
    pub(crate) run_clicked: bool,
    /// User clicked the cancel button.
    pub(crate) cancel_clicked: bool,
    /// User clicked the clear graph button.
    pub(crate) clear_clicked: bool,
    /// User clicked the test async button (wasm only).
    #[cfg(target_arch = "wasm32")]
    pub(crate) test_async_clicked: bool,
}

/// Widget for inserting text and managing graph operations.
pub(crate) struct Inserter<'a> {
    /// Currently selected algorithm.
    selected_algorithm: &'a mut Algorithm,
    /// Input texts to insert.
    texts: &'a mut Vec<String>,
    /// Whether a task is currently running.
    is_running: bool,
}

impl<'a> Inserter<'a> {
    pub(crate) fn new(
        selected_algorithm: &'a mut Algorithm,
        texts: &'a mut Vec<String>,
        is_running: bool,
    ) -> Self {
        Self {
            selected_algorithm,
            texts,
            is_running,
        }
    }

    pub(crate) fn show(
        self,
        ui: &mut Ui,
    ) -> InserterResponse {
        let mut response = InserterResponse::default();

        // Algorithm selection
        ui.horizontal(|ui| {
            ui.label("Algorithm:");
            egui::ComboBox::from_id_salt("inserter_algorithm")
                .selected_text(self.selected_algorithm.to_string())
                .show_ui(ui, |ui| {
                    for algorithm in Algorithm::iter() {
                        ui.selectable_value(
                            self.selected_algorithm,
                            algorithm,
                            algorithm.to_string(),
                        );
                    }
                });
        });

        ui.add_space(5.0);
        ui.label(self.selected_algorithm.description());

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(5.0);

        // Input texts
        ui.label("Input texts:");
        ui.add_space(5.0);

        let mut to_remove = None;
        for (i, text) in self.texts.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(text).desired_width(280.0));
                if ui.button("✖").on_hover_text("Remove").clicked() {
                    to_remove = Some(i);
                }
            });
        }

        if let Some(idx) = to_remove {
            self.texts.remove(idx);
        }

        ui.add_space(5.0);
        if ui.button("+ Add Text").clicked() {
            self.texts.push(String::new());
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(5.0);

        // Run/cancel buttons
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!self.is_running, egui::Button::new("▶ Run"))
                .clicked()
            {
                response.run_clicked = true;
            }

            #[cfg(target_arch = "wasm32")]
            if ui
                .add_enabled(!self.is_running, egui::Button::new("🧪 Test 10s"))
                .on_hover_text("Run a 10-second async test to verify tasks work")
                .clicked()
            {
                response.test_async_clicked = true;
            }

            if self.is_running {
                if ui.button("⏹ Cancel").clicked() {
                    response.cancel_clicked = true;
                }
                ui.spinner();
                ui.label("Processing...");
            }
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(5.0);

        // Clear graph button
        if ui
            .button("🗑 Clear Graph")
            .on_hover_text("Remove all nodes and edges from the graph")
            .clicked()
        {
            response.clear_clicked = true;
        }

        response
    }
}
