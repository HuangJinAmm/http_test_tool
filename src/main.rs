#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[macro_use]
extern crate lazy_static;

// mod json_sy;
mod app;
mod component;
pub mod aes_tool;
mod template;
use app::TemplateApp;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).

    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions{
        // icon_data: todo!(),
        initial_window_size: Some(egui::Vec2{x:1000.0,y:600.0}),
        min_window_size: Some(egui::Vec2{x:1000.0,y:600.0}),
        ..Default::default()
    };
    eframe::run_native(
        "HTTP测试",
        native_options,
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    );
}
