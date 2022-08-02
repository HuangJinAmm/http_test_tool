// #![warn(clippy::all, rust_2018_idioms)]

#[macro_use]
extern crate lazy_static;

mod app;
mod component;
pub mod aes_tool;
pub mod template;
pub use app::TemplateApp;
pub use template::rander_template;

// lazy_static!{
//     static ref RT:Runtime = tokio::runtime::Builder::new_multi_thread().enable_all().worker_threads(4).build().unwrap();
// }
// ----------------------------------------------------------------------------
// When compiling for web:

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};
// use tokio::runtime::Runtime;

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    eframe::start_web(canvas_id, Box::new(|cc| Box::new(TemplateApp::new(cc))))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}
