use eframe::{egui, egui::Color32};

use super::ThemePalette;

pub fn dark() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    v.faint_bg_color = Color32::from_rgb(35, 35, 37);
    v.widgets.noninteractive.bg_fill = Color32::from_rgb(25, 25, 27);
    v.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(242, 242, 247);
    v.widgets.inactive.fg_stroke.color = Color32::from_rgb(242, 242, 247);
    v.widgets.hovered.bg_fill = v.widgets.active.bg_fill;
    v.widgets.active.bg_fill = ThemePalette::DARK.cyan;
    v
}

pub fn light() -> egui::Visuals {
    let mut v = egui::Visuals::light();
    v.widgets.hovered.bg_fill = v.widgets.active.bg_fill;
    v.widgets.active.bg_fill = ThemePalette::LIGHT.green;
    v
}
