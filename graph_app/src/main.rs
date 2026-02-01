#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]
#![allow(clippy::obfuscated_if_else)]

use graph_app::App;

#[derive(Debug, clap::Parser)]
#[clap(author, version, about)]
pub struct Args {
    #[command(flatten)]
    rerun: rerun::clap::RerunArgs,
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser as _;

    let _args = Args::parse();

    // let (rec, _serve_guard) = args.rerun.init("context_graph_app")?;
    println!("Main thread {:?}", std::thread::current().id());
    eframe::run_native(
        "Graph App",
        eframe::NativeOptions::default(),
        Box::new(|creation_context| {
            use eframe::egui::ThemePreference;

            creation_context.egui_ctx.set_theme(ThemePreference::Dark);
            Ok(Box::new(App::new()))
        }),
    )
    .map_err(|e| e.into())
}
