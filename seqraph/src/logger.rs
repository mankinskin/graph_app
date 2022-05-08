use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt::*, layer::SubscriberExt,  EnvFilter};


pub struct Logger(WorkerGuard);


impl Default for Logger {
    fn default() -> Self {
        let dir = tempfile::tempdir().expect("Failed to create tempdir");
        let file_appender = tracing_appender::rolling::hourly(dir, "hypergraph");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let collector = tracing_subscriber::registry()
            .with(EnvFilter::from_default_env().add_directive(tracing::Level::TRACE.into()))
            .with(Layer::new().with_writer(std::io::stdout))
            .with(Layer::new().with_writer(non_blocking));
        tracing::subscriber::set_global_default(collector).expect("Unable to set a global collector");
        Logger(guard)
    }
}