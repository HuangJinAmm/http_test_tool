use egui::{Ui, Layout};

use crate::component::{code_editor::TextEdit, dock_tab::TabUi, header_ui::{HeaderUi, SelectKeyValueItem}};

pub struct EditorDockTabUi<'a> {
    editor:TextEdit,
    text: Option<&'a mut String>
}

impl<'a> EditorDockTabUi<'a> {
    pub fn new(text:Option<&'a mut String>) -> Self {
        EditorDockTabUi {
            editor:TextEdit::default(),
            text
        }
    }
}

impl TabUi for EditorDockTabUi<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        if let Some(text) = self.text {
            self.editor.ui(ui,text); 
        }
    }

    fn context_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Sub menu", |ui| {
            ui.label("hello :)");
        });
    }
}

pub struct Header<'a> {
    id: String,
    header:HeaderUi,
    inputs:&'a mut Vec<SelectKeyValueItem>
}

impl<'a> Header<'a> {
    pub fn new(id:&str,inputs:&'a mut Vec<SelectKeyValueItem>) -> Self {
        Self { id:id.to_owned() , header: HeaderUi::new(),inputs }
    }
}

impl <'a> TabUi for Header<'a> {
    fn ui(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::top_down(egui::Align::Center), |ui|{
            self.header.ui_grid_input(ui,&self.id,self.inputs);
            self.header.ui_grid(ui, "ui_grid",self.inputs);
            self.header.ui_table(ui,self.inputs);
        });
    }

    fn context_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Sub menu", |ui| {
            ui.label("hello :)");
        });
    }
}