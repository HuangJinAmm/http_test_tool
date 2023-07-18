use egui::{
    color_picker::{color_edit_button_srgba, Alpha},
    Slider, Ui, WidgetText, Rect, Layout,
};

use egui_dock::{Style, TabViewer, Tree};
use std::collections::HashMap;

use crate::component::toggle::toggle;

use super::{password::password, editeable_label::editable_label};

// #[derive(serde::Deserialize, serde::Serialize)]
// pub struct DockUi<T> {
//     #[serde(skip)]
//     pub tabs: DockContext<T>,
//     pub tree: Tree<String>,
// }

pub trait TabUi {
    fn ui(&mut self, ui: &mut Ui);
    fn context_menu(&mut self, ui: &mut Ui);
}

// pub trait TabUi {
//     type T;
//     fn ui(&mut self, ui: &mut Ui,date:&mut Self::T);
//     fn context_menu(&mut self, ui: &mut Ui,data:&mut Self::T);
// }

// pub struct DockContext<K> {
//     tabs: HashMap<String, Box<dyn TabUi<T=K>>>,
// }

// impl Default for DockContext<K> {
//     fn default() -> Self {
//         DockContext::new()
//     }
// }
// impl DockContext<K> {
//     pub fn new() -> Self {
//         DockContext {
//             tabs: HashMap::new(),
//         }
//     }
// }

// impl DockUi<K> {
//     pub fn new() -> Self {
//         DockUi {
//             tabs: DockContext::new(),
//             tree: Tree::new(vec![]),
//         }
//     }
//     pub fn load_ui(&mut self,title: String,tab_ui: Box<dyn TabUi>) {
//         self.tabs.tabs.insert(title, tab_ui);
//     }

//     pub fn register_ui(&mut self, title: String, tab_ui: Box<dyn TabUi>) {
//         let _ = self.tabs.tabs.insert(title.clone(), tab_ui);
//         self.tree.push_to_focused_leaf(title);
//     }

//     pub fn trigger_tab(&mut self, tab: &String) {
//         if let Some(index) = self.tree.find_tab(tab) {
//             self.tree.remove_tab(index);
//         } else {
//             self.tree.push_to_focused_leaf(tab.clone());
//         }
//     }

//     pub fn open_tab(&mut self, tab: &String) {
//         if self.tabs.tabs.contains_key(tab) {
//             self.tree.push_to_focused_leaf(tab.clone());
//         }
//     }

//     pub fn get_all_tabs(&self) -> Vec<String> {
//         let tabs = self.tabs.tabs.keys().map(|k| k.to_owned()).collect();
//         tabs
//     }

//     pub fn is_open(&self, title: &String) -> bool {
//         if let Some(_index) = self.tree.find_tab(title) {
//             true
//         } else {
//             false
//         }
//     }

//     pub fn close_tab(&mut self, title: &String) -> bool {
//         if let Some(index) = self.tree.find_tab(title) {
//             let _rm = self.tree.remove_tab(index);
//         }
//         true
//     }
// }

// impl TabViewer for DockContext {
//     type Tab = String;

//     fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
//         if let Some(tab_ui) = self.tabs.get_mut(tab) {
//             tab_ui.ui(ui);
//         }
//     }

//     fn context_menu(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
//         if let Some(tab_ui) = self.tabs.get_mut(tab) {
//             tab_ui.context_menu(ui);
//         }
//     }

//     fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
//         tab.as_str().into()
//     }

//     fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
//         // if let Some(index )= self.tree.find_tab(tab) {
//         //     self.tree.remove_tab(index);
//         // }
//         true
//     }
// }

// pub struct SimpleDemo {
//     lable: String,
//     toggle:bool,
// }

// impl SimpleDemo {
//     pub fn new(title: &str) -> Self {
//         SimpleDemo {
//             lable: title.to_owned(),
//             toggle:false
//         }
//     }
// }

// impl TabUi for SimpleDemo {
//     fn ui(&mut self, ui: &mut Ui) {
//         ui.heading(self.lable.clone());
//         ui.add(toggle(&mut self.toggle));
//         ui.add(password(&mut self.lable));
//         ui.add(editable_label(&mut self.lable));
//     }

//     fn context_menu(&mut self, ui: &mut Ui) {
//         ui.menu_button("Sub menu", |ui| {
//             ui.label("hello :)");
//         });
//     }
// }

// struct StyleEditor {
//     style:Style
// }

// impl DockContext {

//     fn style_editor(&mut self, ui: &mut Ui) {
//         ui.heading("Style Editor");

//         let style = self.style.as_mut().unwrap();

//         ui.collapsing("Border", |ui| {
//             egui::Grid::new("border").show(ui, |ui| {
//                 ui.label("Width:");
//                 ui.add(Slider::new(&mut style.border_width, 1.0..=50.0));
//                 ui.end_row();

//                 ui.label("Color:");
//                 color_edit_button_srgba(ui, &mut style.border_color, Alpha::OnlyBlend);
//                 ui.end_row();
//             });
//         });

//         ui.collapsing("Selection", |ui| {
//             egui::Grid::new("selection").show(ui, |ui| {
//                 ui.label("Color:");
//                 color_edit_button_srgba(ui, &mut style.selection_color, Alpha::OnlyBlend);
//                 ui.end_row();
//             });
//         });

//         ui.collapsing("Separator", |ui| {
//             egui::Grid::new("separator").show(ui, |ui| {
//                 ui.label("Width:");
//                 ui.add(Slider::new(&mut style.separator_width, 1.0..=50.0));
//                 ui.end_row();

//                 ui.label("Offset limit:");
//                 ui.add(Slider::new(&mut style.separator_extra, 1.0..=300.0));
//                 ui.end_row();

//                 ui.label("Idle color:");
//                 color_edit_button_srgba(ui, &mut style.separator_color_idle, Alpha::OnlyBlend);
//                 ui.end_row();

//                 ui.label("Hovered color:");
//                 color_edit_button_srgba(ui, &mut style.separator_color_hovered, Alpha::OnlyBlend);
//                 ui.end_row();

//                 ui.label("Dragged color:");
//                 color_edit_button_srgba(ui, &mut style.separator_color_dragged, Alpha::OnlyBlend);
//                 ui.end_row();
//             });
//         });

//         ui.collapsing("Tabs", |ui| {
//             ui.separator();

//             ui.checkbox(
//                 &mut style.tab_hover_name,
//                 "Show tab name when hovered over them",
//             );
//             ui.checkbox(&mut style.tabs_are_draggable, "Tabs are draggable");
//             ui.checkbox(&mut style.expand_tabs, "Expand tabs");
//             ui.checkbox(&mut style.show_context_menu, "Show context menu");
//             ui.checkbox(&mut style.show_add_buttons, "Show add buttons");
//             ui.checkbox(
//                 &mut style.tab_include_scrollarea,
//                 "Include ScrollArea inside of tabs",
//             );

//             ui.checkbox(
//                 &mut style.hline_below_active_tab_name,
//                 "Show a line below the active tab name",
//             );

//             ui.separator();

//             ui.horizontal(|ui| {
//                 ui.add(Slider::new(&mut style.tab_bar_height, 20.0..=50.0));
//                 ui.label("Tab bar height");
//             });

//             ui.separator();

//             ui.label("Rounding");
//             ui.horizontal(|ui| {
//                 ui.add(Slider::new(&mut style.tab_rounding.nw, 0.0..=15.0));
//                 ui.label("North-West");
//             });
//             ui.horizontal(|ui| {
//                 ui.add(Slider::new(&mut style.tab_rounding.ne, 0.0..=15.0));
//                 ui.label("North-East");
//             });
//             ui.horizontal(|ui| {
//                 ui.add(Slider::new(&mut style.tab_rounding.sw, 0.0..=15.0));
//                 ui.label("South-West");
//             });
//             ui.horizontal(|ui| {
//                 ui.add(Slider::new(&mut style.tab_rounding.se, 0.0..=15.0));
//                 ui.label("South-East");
//             });

//             ui.separator();

//             ui.checkbox(&mut style.show_close_buttons, "Allow closing tabs");

//             ui.separator();

//             egui::Grid::new("tabs_colors").show(ui, |ui| {
//                 ui.label("Title text color, inactive and unfocused:");
//                 color_edit_button_srgba(ui, &mut style.tab_text_color_unfocused, Alpha::OnlyBlend);
//                 ui.end_row();

//                 ui.label("Title text color, inactive and focused:");
//                 color_edit_button_srgba(ui, &mut style.tab_text_color_focused, Alpha::OnlyBlend);
//                 ui.end_row();

//                 ui.label("Title text color, active and unfocused:");
//                 color_edit_button_srgba(
//                     ui,
//                     &mut style.tab_text_color_active_unfocused,
//                     Alpha::OnlyBlend,
//                 );
//                 ui.end_row();

//                 ui.label("Title text color, active and focused:");
//                 color_edit_button_srgba(
//                     ui,
//                     &mut style.tab_text_color_active_focused,
//                     Alpha::OnlyBlend,
//                 );
//                 ui.end_row();

//                 ui.label("Close button color unfocused:");
//                 color_edit_button_srgba(ui, &mut style.close_tab_color, Alpha::OnlyBlend);
//                 ui.end_row();

//                 ui.label("Close button color focused:");
//                 color_edit_button_srgba(ui, &mut style.close_tab_active_color, Alpha::OnlyBlend);
//                 ui.end_row();

//                 ui.label("Close button background color:");
//                 color_edit_button_srgba(
//                     ui,
//                     &mut style.close_tab_background_color,
//                     Alpha::OnlyBlend,
//                 );
//                 ui.end_row();

//                 ui.label("Bar background color:");
//                 color_edit_button_srgba(ui, &mut style.tab_bar_background_color, Alpha::OnlyBlend);
//                 ui.end_row();

//                 ui.label("Outline color:")
//                     .on_hover_text("The outline around the active tab name.");
//                 color_edit_button_srgba(ui, &mut style.tab_outline_color, Alpha::OnlyBlend);
//                 ui.end_row();

//                 ui.label("Horizontal line color:").on_hover_text(
//                     "The line separating the tab name area from the tab content area",
//                 );
//                 color_edit_button_srgba(ui, &mut style.hline_color, Alpha::OnlyBlend);
//                 ui.end_row();

//                 ui.label("Background color:");
//                 color_edit_button_srgba(ui, &mut style.tab_background_color, Alpha::OnlyBlend);
//                 ui.end_row();
//             });
//         });
//     }
// }
