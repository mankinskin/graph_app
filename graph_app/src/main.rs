#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]
#![allow(clippy::obfuscated_if_else)]

use graph_app::App;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, clap::Parser)]
#[clap(author, version, about)]
#[cfg(not(target_arch = "wasm32"))]
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

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    eframe::run_native(
        "Graph App",
        native_options,
        Box::new(|creation_context| {
            use eframe::egui::ThemePreference;

            creation_context.egui_ctx.set_theme(ThemePreference::Dark);
            Ok(Box::new(App::new()))
        }),
    )
    .map_err(|e| e.into())
}

// WebAssembly entry point
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect tracing to console.log
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        // Get the canvas element
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("graph_app_canvas")
            .expect("No canvas element with id 'graph_app_canvas'")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Element is not a canvas");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|creation_context| {
                    use eframe::egui::ThemePreference;
                    creation_context.egui_ctx.set_theme(ThemePreference::Dark);
                    Ok(Box::new(App::new()))
                }),
            )
            .await;

        // Remove loading text and show canvas
        let loading_text = document.get_element_by_id("loading_text");
        if let Some(loading_text) = loading_text {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                },
                Err(e) => {
                    loading_text.set_inner_html(&format!(
                        "<p>The app has crashed. Error: {:?}</p>",
                        e
                    ));
                },
            }
        }
    });
}
