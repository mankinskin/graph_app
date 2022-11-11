use crate::*;
use tracing_subscriber::{
    prelude::*,
    fmt::{
        self,
        writer::MakeWriterExt,
        format::PrettyFields,
    },
    field::MakeExt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
    filter::EnvFilter,
};
use std::time::Duration;
use tracing::Level;

pub struct Logger;

#[derive(Valuable, Debug)]
pub enum Event {
    NewIndex
}

struct SleepLayer {
    duration: Duration,
}
impl SleepLayer {
    pub fn with(duration: Duration) -> Self {
        Self {
            duration,
        }
    }
}

impl<S: Subscriber> Layer<S> for SleepLayer {
    fn on_event(&self, _event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        std::thread::sleep(self.duration);
    }
}

impl Default for Logger {
    fn default() -> Self {
        let registry = tracing_subscriber::registry();

        #[cfg(feature = "log_file")]
        let registry = {
            let dir = tempfile::tempdir().expect("Failed to create tempdir");
            let file_appender = tracing_appender::rolling::hourly(dir, "hypergraph");
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
            let file_writer = non_blocking
                .with_max_level(Level::TRACE);
            let file_layer = fmt::layer()
                .with_writer(file_writer)
                .pretty();
            registry.with(file_layer)
        };

        #[cfg(feature = "log_stdout")]
        let registry = {
            let stdout_writer = std::io::stdout
                .with_max_level(Level::TRACE);
            let stdout_layer = fmt::layer()
                .with_writer(stdout_writer)
                .pretty()
                .fmt_fields(PrettyFields::new().debug_alt());
            registry.with(stdout_layer)
        };
        #[cfg(feature = "log_gui")]
        let registry = {
            registry
                .with(tracing_egui::layer())
                .with(SleepLayer::with(Duration::from_secs(1)))
        };

        let registry = registry.with(EnvFilter::new("eframe=off,[]=trace"));
        registry.init();
        Logger
    }
}