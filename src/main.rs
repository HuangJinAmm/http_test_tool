#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    // tracing_subscriber::fmt::init();

    use eframe::IconData;
    egui_logger::init().unwrap();
    log::set_max_level(log::LevelFilter::Debug);
    let mut native_options = eframe::NativeOptions::default();
    let icon = IconData::try_from_png_bytes(include_bytes!("../http.png")).unwrap();
    native_options.icon_data = Some(icon);
    eframe::run_native(
        "Http测试工具",
        native_options,
        Box::new(|cc| Box::new(http_test_tool::TemplateApp::new(cc))),
    )
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();
    egui_logger::init().unwrap();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(eframe_template::TemplateApp::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
