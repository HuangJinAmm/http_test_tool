use egui::{Key, Vec2};

pub fn editable_label_ui(ui: &mut egui::Ui, value: &mut String) -> egui::Response {
    let state_id = ui.id().with("editable_label");
    let mut is_edit = ui.data_mut(|d| d.get_temp::<bool>(state_id).unwrap_or(false));
    let rsp;
    if is_edit {
        rsp = ui.text_edit_multiline(value);
        if rsp.lost_focus() || ui.input(|i| i.key_pressed(Key::Enter)) {
            is_edit = false;
        }
    } else {
        let inner_rsp = ui.horizontal(|ui| {
            let resp = ui.label(value.clone());
            // if resp.double_clicked() {
            //     is_edit = !is_edit;
            // }
            let rect = resp.rect.expand2(Vec2::new(40., 10.));
            if ui.rect_contains_pointer(rect) {
                let rsp = ui.button("编辑");
                if rsp.clicked() {
                    is_edit = !is_edit;
                }
            }
            return resp;
        });
        rsp = inner_rsp.inner;
    }
    ui.data_mut(|d| d.insert_temp(state_id, is_edit));
    rsp
}

pub fn editable_label(text: &mut String) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| editable_label_ui(ui, text)
}
