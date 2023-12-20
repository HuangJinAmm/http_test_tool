mod icons;
mod palette;
mod visuals;

pub use icons::Icon;
pub use palette::{DrawingPalette, ThemePalette};

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use eframe::egui;

pub fn init(initial_mode: dark_light::Mode, ctx: &egui::Context) {

    ctx.set_visuals(egui_visuals(initial_mode));

    let mut style = (*ctx.style()).clone();
    style.spacing.button_padding = egui::vec2(7.0, 7.0);
    style.spacing.menu_margin = egui::Margin::same(10.0);

    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::new(17.0, egui::FontFamily::Proportional));
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::new(17.0, egui::FontFamily::Monospace));
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::new(17.0, egui::FontFamily::Proportional));
    ctx.set_style(style);

    // poll_system_theme(ctx, initial_mode);
}

pub fn apply_settings(mode:dark_light::Mode, ctx: &egui::Context) {
    ctx.set_visuals(egui_visuals(mode));
}

// fn poll_system_theme(
//     ctx: &egui::Context, initial_mode: dark_light::Mode,
// ) {
//     let ctx = ctx.clone();

//     let mut mode = initial_mode;

//     thread::spawn(move || loop {
//         let m = dark_light::detect();
//         if mode != m {
//             mode = m;
//             ctx.set_visuals(egui_visuals(m));
//             ctx.request_repaint();
//         }
//         thread::sleep(Duration::from_secs(2));
//     });
// }

pub fn egui_visuals(m: dark_light::Mode) -> egui::Visuals {
    match m {
        dark_light::Mode::Default | dark_light::Mode::Dark => visuals::dark(),
        dark_light::Mode::Light => visuals::light(),
    }
}

pub fn register_fonts(fonts: &mut egui::FontDefinitions,font_data:&'static [u8]) {
    let mut font = egui::FontData::from_static(font_data);
    font.tweak.y_offset_factor = -0.1;

    fonts.font_data.insert("material_icons".to_owned(), font);

    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("material_icons".to_owned());
}
