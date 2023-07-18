// use egui::TextEdit;
// use egui_extras::{Column, TableBuilder};

use super::syntax_highlight::{highlight, CodeTheme};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SelectKeyValueItem {
    pub selected: bool,
    pub key: String,
    pub value: String,
}

impl SelectKeyValueItem {
    pub fn new(key: &str, value: &str) -> Self {
        Self {
            selected: true,
            key: key.into(),
            value: value.into(),
        }
    }
}

pub struct HeaderUi {}

impl HeaderUi {
    pub fn new() -> Self {
        Self {}
    }

    pub fn ui_grid_input(ui: &mut egui::Ui, id: &str, inputs: &mut Vec<SelectKeyValueItem>) {
        ui.group(|ui| {
            egui::Grid::new(id)
                .num_columns(3)
                .min_col_width(10.)
                .min_row_height(20.)
                .show(ui, |ui| {
                    ui.add_sized(ui.available_size(), egui::widgets::Label::new(""));
                    ui.add_sized(
                        [120., 20.],
                        egui::widgets::Label::new(egui::RichText::new("键").strong()),
                    );
                    ui.add_sized(
                        ui.available_size(),
                        egui::widgets::Label::new(egui::RichText::new("值").strong()),
                    );
                    ui.end_row();
                    for SelectKeyValueItem {
                        selected,
                        key,
                        value,
                    } in inputs
                    {
                        ui.checkbox(selected, "");

                        let theme = CodeTheme::from_memory(ui.ctx());

                        let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                            let layout_job = highlight(ui.ctx(), &theme, string, "json");
                            // layout_job.wrap.max_width = wrap_width; // no wrapping
                            ui.fonts(|f| f.layout_job(layout_job))
                        };
                        ui.add_sized(
                            ui.available_size(),
                            egui::text_edit::TextEdit::singleline(key),
                        );
                        ui.add_sized(
                            ui.available_size(),
                            egui::text_edit::TextEdit::singleline(value).layouter(&mut layouter),
                        );
                        ui.end_row();
                    }
                });
        });
    }

    pub fn ui_grid(ui: &mut egui::Ui, id: &str, inputs: &mut Vec<SelectKeyValueItem>) {
        ui.group(|ui| {
            egui::Grid::new(id)
                .num_columns(2)
                .min_col_width(80.)
                .min_row_height(20.)
                .show(ui, |ui| {
                    ui.add_sized(
                        ui.available_size(),
                        egui::widgets::Label::new(egui::RichText::new("键").strong()),
                    );
                    ui.add_sized(
                        ui.available_size(),
                        egui::widgets::Label::new(egui::RichText::new("值").strong()),
                    );
                    ui.end_row();
                    for SelectKeyValueItem { key, value, .. } in inputs {
                        ui.add_sized(ui.available_size(), egui::widgets::Label::new(key.clone()));
                        ui.add_sized(
                            ui.available_size(),
                            egui::widgets::Label::new(value.clone()),
                        );
                        ui.end_row();
                    }
                });
        });
    }

    // pub fn ui_table(&mut self, ui: &mut egui::Ui, inputs: &mut Vec<SelectKeyValueItem>) {
    //     let most = ui.available_width();
    //     TableBuilder::new(ui)
    //         .column(Column::exact(10.0))
    //         .column(Column::auto_with_initial_suggestion(300.0))
    //         .column(Column::remainder())
    //         .header(20.0, |mut header| {
    //             header.col(|ui| {
    //                 ui.label("");
    //             });
    //             header.col(|ui| {
    //                 ui.label("键");
    //             });
    //             header.col(|ui| {
    //                 ui.label("值");
    //             });
    //         })
    //         .body(|mut body| {
    //             body.row(30.0, |mut row| {
    //                 for SelectKeyValueItem {
    //                     selected,
    //                     key,
    //                     value,
    //                 } in inputs
    //                 {
    //                     row.col(|ui| {
    //                         ui.checkbox(selected, "");
    //                     });
    //                     row.col(|ui| {
    //                         TextEdit::singleline(key).desired_width(most / 2.0).show(ui);
    //                         // ui.text_edit_singleline(key);
    //                     });
    //                     row.col(|ui| {
    //                         TextEdit::singleline(value)
    //                             .desired_width(ui.available_width())
    //                             .show(ui);
    //                         // ui.text_edit_singleline(value);
    //                     });
    //                 }
    //             });
    //         });
    // }
}
