use derive_more::derive::{
    Deref,
    DerefMut,
};
use eframe::egui::{
    self,
    Ui,
};
use ngrams::graph::vocabulary::ProcessStatus;
use strum::IntoEnumIterator;

#[derive(Deref, DerefMut, Debug)]
pub struct ShowStatus<'a>(pub &'a ngrams::graph::Status);

impl ShowStatus<'_> {
    pub fn show(
        &self,
        ctx: &egui::Context,
    ) {
        egui::Window::new("Status").show(ctx, |ui| {
            ui.label(format!("Texts: \"{:#?}\"", self.insert_texts));
            ProcessStatus::iter()
                .skip(1)
                .for_each(|pass| self.show_pass(ui, pass))
        });
    }
    fn show_pass(
        &self,
        ui: &mut Ui,
        pass: ProcessStatus,
    ) {
        let checked = *self.pass() > pass
            || (pass == ProcessStatus::Finished
                && *self.pass() == ProcessStatus::Finished);
        let percent =
            (*self.steps() as f32 / *self.steps_total() as f32 * 100.0) as u32;
        let text = format!(
            "{:?}{}",
            pass,
            (pass == ProcessStatus::Finished)
                .then(|| String::new())
                .unwrap_or_else(|| checked
                    .then_some(100)
                    .or_else(|| (pass
                        == ProcessStatus::iter()
                            .skip_while(|i| *i < *self.pass())
                            .next()
                            .unwrap())
                    // is next
                    .then_some(percent))
                    .map(|p| format!(" Pass: {}%", p))
                    .unwrap_or(String::from(" Pass")))
        );
        ui.checkbox(&mut (checked.clone()), text);
    }
}
