#![warn(non_snake_case)]
use crate::app::{ADD_ID_KEY, ID_COUNT_KEY};
use egui::{Id, InnerResponse, RichText, Ui};
use std::{collections::BTreeMap, sync::mpsc::SyncSender};

#[derive(Clone, PartialEq, Debug)]
pub enum Action {
    Keep,
    Delete(Vec<u64>),
    Selected((u64, String)),
}

/*
树状列表中的元素:
*/
#[derive(Clone, Debug)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize,serde::Serialize))]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ContextTree {
    id: u64,
    title: String,
    selected: bool,
    sub_items: BTreeMap<u64, ContextTree>,
}

impl ContextTree {
    //添加子项目
    pub fn addItem(&mut self, id: u64, title: &str) {
        let sub = ContextTree::new(id, title);
        self.sub_items.insert(id, sub);
    }

    pub fn deleteItem(&mut self, id: &u64) -> Vec<u64> {
        match self.sub_items.remove(id) {
            Some(contree) => {
                contree.list_all_subids().unwrap_or(Vec::new())
            },
            None => Vec::new(),
        }
    }

    fn list_all_subids(&self) -> Option<Vec<u64>> {
        let subs = self.sub_items.clone();
        if subs.len() == 0 {
            None
        } else {
            let sub_ids = subs.values()
                        // .filter(|ct| ct.list_all_subids().is_some())
                        // .flat_map(|ct|ct.list_all_subids())
                        .map(|ct| {
                            let mut all_sub_id = vec![self.id];
                            if let Some(mut subs) = ct.list_all_subids() {
                                all_sub_id.append(&mut subs);
                            }
                            return all_sub_id;
                        })
                        .flatten()
                        .collect();
            Some(sub_ids)
        }
    }

    pub fn new(id: u64, title: &str) -> Self {
        Self {
            id,
            selected: false,
            title: String::from(title),
            sub_items: BTreeMap::new(),
        }
    }

    pub fn ui_impl(
        &mut self,
        ui: &mut Ui,
        selected_str: u64,
        add_title: &mut String,
        flilter: &str,
    ) -> Action {
        let id_source = ui.make_persistent_id(self.id.to_string());
        self.selected = selected_str == self.id;
        //删除
        if ui.input().key_pressed(egui::Key::Delete) {
            let del_ids = self.deleteItem(&selected_str);
            if del_ids.len() > 0 {
                return Action::Delete(del_ids); 
            }
        }

        if self.sub_items.is_empty() {
            ui.horizontal(|ui| {
                ui.label("▼");
                //显示ID
                // ui.label(
                //     RichText::new("(🆔:".to_owned() + &self.id.to_string() + ")")
                //         .color(egui::Color32::RED),
                // );
                let select_resp = ui
                    .toggle_value(&mut self.selected, self.title.clone())
                    .context_menu(|ui| {
                        ui.add_space(5.);
                        ui.text_edit_singleline(add_title);
                        ui.add_space(5.);
                        ui.horizontal(|ui| {
                            let add_resp = ui.button("添加");
                            let rename_resp = ui.button("重命名");
                            if add_resp.clicked() {
                                let sub_id = self.add_sub_item(ui, add_title);
                                self.sender_add_info(ui, sub_id);
                                ui.close_menu();
                            }
                            if rename_resp.clicked() {
                                self.title = add_title.to_string();
                                ui.close_menu();
                            }
                        });
                    });
                if select_resp.clicked() {
                    return Action::Selected((self.id, self.title.clone()));
                } else {
                    return Action::Keep;
                }
            })
            .inner
        } else {
            let (_, headRep, bodyResp) =
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id_source,
                    true,
                )
                .show_header(ui, |ui| {
                    // ui.label(
                    //     RichText::new("(🆔:".to_owned() + &self.id.to_string() + ")")
                    //         .color(egui::Color32::RED),
                    // );
                    let select_resp = ui
                        .toggle_value(&mut self.selected, self.title.clone())
                        .context_menu(|ui| {
                            ui.add_space(5.);
                            ui.text_edit_singleline(add_title);
                            ui.add_space(5.);
                            ui.horizontal(|ui| {
                                let add_resp = ui.button("添加");
                                let rename_resp = ui.button("重命名");
                                if add_resp.clicked() {
                                    let sub_id = self.add_sub_item(ui, add_title);
                                    self.sender_add_info(ui, sub_id);
                                    ui.close_menu();
                                }
                                if rename_resp.clicked() {
                                    self.title = add_title.to_string();
                                    ui.close_menu();
                                }
                            });
                        });
                    if select_resp.clicked() {
                        return Action::Selected((self.id, self.title.clone()));
                    } else {
                        return Action::Keep;
                    }
                })
                .body(|ui| self.sub_ui(ui, selected_str, add_title, flilter));
            match (headRep.inner, bodyResp) {
                (Action::Selected(head), _) => Action::Selected(head),
                (
                    Action::Keep,
                    Some(InnerResponse {
                        inner: Action::Selected(body),
                        ..
                    }),
                ) => Action::Selected(body),
                (
                    _,
                    Some(InnerResponse {
                        inner:Action::Delete(del_ids),
                        ..
                    }),
                ) => Action::Delete(del_ids),
                _ => Action::Keep,
            }
        }
    }
    fn sender_add_info(&self, ui: &mut Ui, sub_id: u64) {
        let mut data = ui.data();
        let sender: SyncSender<(u64, u64)> = data.get_temp(Id::new(ADD_ID_KEY)).unwrap();
        sender.send((self.id, sub_id));
    }

    fn add_sub_item(&mut self, ui: &mut Ui, add_title: &mut String) -> u64 {
        let mut data = ui.data();
        let id_count: &mut u64 = data.get_persisted_mut_or_default(Id::new(ID_COUNT_KEY));
        *id_count = *id_count + 1;
        let sub_id = *id_count;
        self.addItem(sub_id, add_title.clone().as_str());
        sub_id
    }

    pub fn sub_ui(
        &mut self,
        ui: &mut Ui,
        selected_str: u64,
        add_title: &mut String,
        flilter: &str,
    ) -> Action {
        let Self { sub_items, .. } = self;

        for (_, sub) in sub_items.iter_mut() {
            if sub.title.contains(flilter) {
                let sub_resp = sub.ui_impl(ui, selected_str, add_title, flilter);
                if let Action::Keep = sub_resp {
                    continue;
                } else {
                    return sub_resp;
                }
            }
        }
        Action::Keep
        // self.sub_items = sub_items.clone().into_iter().filter_map(|mut subtree|{
        // subtree.1.ui_impl(ui, selected_str);
        // Some(subtree)
        // if subtree.1.ui_impl(ui,selected_str) == Action::Keep{
        //     Some(subtree)
        // } else {
        //     None
        // }
        // }).collect();
    }
}
