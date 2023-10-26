#![warn(clippy::all, rust_2018_idioms)]

pub mod api_context;
mod app;
pub mod component;
pub mod request_data;
pub mod ui;
pub mod utils;
pub mod history_db;
pub use app::TemplateApp;
