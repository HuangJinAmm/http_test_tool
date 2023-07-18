use core::fmt::Debug;
use egui::{Align2, Id, InnerResponse, RichText, Ui, Window};
use egui_dnd::{utils::shift_vec, DragDropItem, DragDropUi, Handle};
use log::info;

#[derive(Clone, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub enum Action {
    Keep,
    ///复制 复制的节点id和标题..只能复制节点，不能复制集合
    Copy((u64, String)),
    ///粘贴 的节点id路径，根路径最后
    Parse(Vec<u64>),
    ///删除
    Delete(Vec<u64>),
    ///添加
    Add((Vec<u64>, NodeType)),
    ///重命名
    Rename(Vec<u64>),

    Selected((Vec<u64>, String)),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum NodeType {
    Collection,
    Node,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TreeUi {
    ///选中的节点
    selected: u64,
    action_tmp: Action,
    popup: bool,
    open: bool,
    rename: String,
    filter: String,
    id_count: u64,
    sub_node: TreeUiNode,
}

impl TreeUi {
    pub fn new() -> Self {
        TreeUi {
            selected: 0,
            action_tmp: Action::Keep,
            popup: false,
            open: true,
            rename: String::new(),
            filter: String::new(),
            id_count: 1,
            sub_node: TreeUiNode {
                id: 0,
                title: "ApiPost".to_owned(),
                node_type: NodeType::Collection,
                sub_items: Vec::new(),
                drag_drop_ui: DragDropUi::default(),
            },
        }
    }

    pub fn selected_id(&self) -> u64 {
        self.selected
    }

    pub fn pre_action(&self) -> Action {
        self.action_tmp.clone()
    }

    pub fn del(&mut self, mut del: Vec<u64>) -> Option<Vec<u64>> {
        //第一个节点只有一个，直接删掉
        dbg!(&del);
        let _node_id = del.pop();
        self.sub_node.delete_item(del)
    }

    pub fn rename(&mut self, mut rename: Vec<u64>) {
        //第一个节点只有一个，直接删掉
        let _node_id = rename.pop();
        let title = &self.rename;
        self.sub_node.rename(rename, title);
    }

    /// 返回新的节点id,(复制的id,粘贴的id),只能粘贴节点
    pub fn parse_node(&mut self, parse_pos: Vec<u64>) -> Option<(u64, u64)> {
        let Action::Copy((copy_id,title))  = self.action_tmp.clone() else {return None;};
        self.id_count = self.id_count + 1;
        let new_id = self.id_count;
        if self
            .sub_node
            .add_recusive(parse_pos, new_id, &title, NodeType::Node)
        {
            Some((copy_id, new_id))
        } else {
            None
        }
    }

    pub fn add(&mut self, mut add: Vec<u64>, node_type: NodeType) -> bool {
        //最后一个是0号，直接删掉
        let _node_id = add.pop();
        self.id_count = self.id_count + 1;
        let new_id = self.id_count;
        let title = &self.rename;
        self.sub_node.add_recusive(add, new_id, title, node_type)
    }

    pub fn ui_impl(&mut self, ui: &mut Ui) -> Action {
        //传递打开动作
        let mut open_action = None;

        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.filter)
                    .desired_width(180.0)
                    .hint_text("筛选条件"),
            );

            if self.open {
                if ui.small_button("📕").clicked() {
                    self.open = false;
                    open_action = Some(false);
                }
            } else {
                if ui.small_button("📖").clicked() {
                    self.open = true;
                    open_action = Some(true);
                }
            }
        });

        //弹框出路action_tmp里的事情
        let mut popup_resp = None;
        if self.popup {
            let inner_popup_resp = match self.action_tmp.clone() {
                Action::Add((adds, NodeType::Collection)) => {
                    if let Some(add_resp) = Window::new("添加集合")
                        .anchor(Align2::CENTER_CENTER, (1.0, 1.0))
                        .collapsible(false)
                        .show(ui.ctx(), |ui| {
                            ui.text_edit_singleline(&mut self.rename);
                            ui.horizontal(|ui| {
                                if ui.button("确认").clicked() {
                                    self.popup = false;
                                    if self.add(adds, NodeType::Collection) {
                                        return Action::Add((
                                            vec![self.id_count],
                                            NodeType::Collection,
                                        ));
                                    }
                                }
                                if ui.button("取消").clicked() {
                                    self.popup = false
                                }
                                Action::Keep
                            })
                            .inner
                        })
                    {
                        add_resp.inner.unwrap_or(Action::Keep)
                    } else {
                        Action::Keep
                    }
                }
                Action::Add((adds, NodeType::Node)) => {
                    if let Some(add_resp) = Window::new("添加节点")
                        .anchor(Align2::CENTER_CENTER, (1.0, 1.0))
                        .collapsible(false)
                        .show(ui.ctx(), |ui| {
                            ui.text_edit_singleline(&mut self.rename);
                            ui.horizontal(|ui| {
                                if ui.button("确认").clicked() {
                                    self.popup = false;
                                    if self.add(adds, NodeType::Node) {
                                        return Action::Add((vec![self.id_count], NodeType::Node));
                                    }
                                }
                                if ui.button("取消").clicked() {
                                    self.popup = false;
                                }
                                Action::Keep
                            })
                            .inner
                        })
                    {
                        add_resp.inner.unwrap_or(Action::Keep)
                    } else {
                        Action::Keep
                    }
                }
                Action::Rename(rename) => {
                    if let Some(rename_resp) = Window::new("重命名")
                        .anchor(Align2::CENTER_CENTER, (1.0, 1.0))
                        .collapsible(false)
                        .show(ui.ctx(), |ui| {
                            ui.text_edit_singleline(&mut self.rename);
                            ui.horizontal(|ui| {
                                if ui.button("确认").clicked() {
                                    self.rename(rename);
                                    self.popup = false;
                                }
                                if ui.button("取消").clicked() {
                                    self.popup = false;
                                }
                                Action::Keep
                            })
                            .inner
                        })
                    {
                        rename_resp.inner.unwrap_or(Action::Keep)
                    } else {
                        Action::Keep
                    }
                }
                _ => Action::Keep,
            };
            popup_resp = Some(inner_popup_resp);
        }

        let sub_resp =
            match self
                .sub_node
                .ui_impl(ui, self.selected, &self.filter, open_action, None)
            {
                Action::Delete(del) => match self.del(del) {
                    Some(dels) => {
                        if dels.is_empty() {
                            Action::Keep
                        } else {
                            Action::Delete(dels)
                        }
                    }
                    None => Action::Keep,
                },
                //内部传上来的动作，放tmp里
                Action::Add(add) => {
                    self.popup = true;
                    self.action_tmp = Action::Add(add);
                    Action::Keep
                }
                Action::Selected((id, title)) => {
                    self.selected = id.first().unwrap().to_owned();
                    Action::Selected((id, title))
                }
                //内部传上来的动作，放tmp里
                Action::Rename(rename) => {
                    self.popup = true;
                    self.action_tmp = Action::Rename(rename);
                    Action::Keep
                }
                //内部传上来的动作，放tmp里
                Action::Copy(cop) => {
                    self.action_tmp = Action::Copy(cop.clone());
                    Action::Copy(cop)
                }
                other => other,
            };

        match (popup_resp, sub_resp) {
            (None, sub) => sub,
            (Some(pop), _) => pop,
        }
    }
}

/*
树状列表中的元素:
*/
#[derive(Clone)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize,serde::Serialize))]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct TreeUiNode {
    id: u64,
    title: String,
    node_type: NodeType,
    // sub_items: BTreeMap<u64, TreeUiNode>,
    sub_items: Vec<TreeUiNode>,
    #[serde(skip)]
    drag_drop_ui: DragDropUi,
}

impl Debug for TreeUiNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeUiNode")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("node_type", &self.node_type)
            .field("sub_items", &self.sub_items)
            .finish()
    }
}

impl TreeUiNode {
    //添加子项目
    pub fn add_item(&mut self, id: u64, title: &str, node_type: NodeType) -> bool {
        let sub = TreeUiNode::new(id, title, node_type);
        // match self.sub_items.insert(id, sub) {
        //     Some(_) => true,
        //     None => false,
        // }
        self.sub_items.push(sub);
        true
    }

    pub fn find_node(&mut self, id: u64) -> Option<&mut TreeUiNode> {
        self.sub_items.iter_mut().find(|node| node.id == id)
    }

    pub fn add_recusive(
        &mut self,
        mut ids: Vec<u64>,
        id: u64,
        title: &str,
        node_type: NodeType,
    ) -> bool {
        if ids.len() > 0 {
            let sub_id = ids.pop().unwrap();
            // match self.sub_items.get_mut(&sub_id) {
            match self.find_node(sub_id) {
                Some(sub_node) => sub_node.add_recusive(ids, id, title, node_type),
                None => false,
            }
        } else {
            self.add_item(id, title, node_type)
        }
    }

    ///重命名节点
    pub fn rename(&mut self, mut ids: Vec<u64>, title: &str) {
        if ids.len() > 0 {
            let sub_id = ids.pop().unwrap();
            // match self.sub_items.get_mut(&sub_id) {
            match self.find_node(sub_id) {
                Some(sub_node) => sub_node.rename(ids, title),
                None => {}
            }
        } else {
            self.title = title.to_owned();
        }
    }

    ///返回删除的节点id
    pub fn delete_item(&mut self, mut id: Vec<u64>) -> Option<Vec<u64>> {
        if id.len() > 1 {
            let sub_id = id.pop().unwrap();
            // let Some(sub_node) = self.sub_items.get_mut(&sub_id) else {
            let Some(sub_node) = self.find_node(sub_id) else {
                return None;
            };
            sub_node.delete_item(id)
        } else {
            let sub_id = id.pop().unwrap();
            if let Some((i, _)) = self.sub_items.iter().enumerate().find(|n| n.1.id == sub_id) {
                let node = self.sub_items.remove(i);
                return Some(node.list_all_subids().unwrap_or(Vec::new()));
            }
            None
        }
    }

    fn list_all_subids(&self) -> Option<Vec<u64>> {
        let subs = self.sub_items.clone();
        let mut sub_ids = vec![self.id];
        if subs.len() > 0 {
            for sub in subs {
                if let Some(mut each_sub_ids) = sub.list_all_subids() {
                    sub_ids.append(&mut each_sub_ids);
                }
            }
        }
        Some(sub_ids)
    }

    pub fn new(id: u64, title: &str, node_type: NodeType) -> Self {
        Self {
            id,
            title: String::from(title),
            sub_items: Vec::new(),
            node_type,
            drag_drop_ui: DragDropUi::default(),
        }
    }

    pub fn ui_impl(
        &mut self,
        ui: &mut Ui,
        selected_str: u64,
        flilter: &str,
        open: Option<bool>,
        handler: Option<Handle<'_>>,
    ) -> Action {
        let id_source = ui.make_persistent_id(self.id.to_string());
        let mut selected = selected_str == self.id;
        // delete删除
        // if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
        //     if let Some(del_ids) = self.delete_item(vec![selected_str]) {
        //         if del_ids.len() > 0 {
        //             return Action::Delete(del_ids);
        //         }
        //     }
        // }

        match self.node_type {
            NodeType::Node => {
                if !self.title.contains(flilter) {
                    return Action::Keep;
                }
                ui.horizontal(|ui| {
                    // ui.label("▼");
                    if let Some(h) = handler {
                        h.ui(ui, self, |ui| {
                            ui.label("🖥");
                        });
                    }

                    ui.label(RichText::new(format!("(🆔:{})", self.id)).color(egui::Color32::RED));
                    let mut context_resp = Option::None;
                    let select_resp = ui
                        .toggle_value(&mut selected, self.title.clone())
                        .context_menu(|ui| {
                            // if ui.button("添加集合").clicked() {
                            //     ui.close_menu();
                            //     context_resp =
                            //         Some(Action::Add((vec![self.id], NodeType::Collection)));
                            // }
                            if ui.button("复制节点").clicked() {
                                ui.close_menu();
                                context_resp = Some(Action::Copy((self.id, self.title.clone())));
                            }
                            if ui.button("重命名").clicked() {
                                ui.close_menu();
                                context_resp = Some(Action::Rename(vec![self.id]));
                            }
                            if ui.button("删除").clicked() {
                                ui.close_menu();
                                context_resp = Some(Action::Delete(vec![self.id]));
                            }
                        });
                    if context_resp.is_some() {
                        return context_resp.unwrap();
                    }
                    if select_resp.clicked() {
                        return Action::Selected((vec![self.id], self.title.clone()));
                    } else {
                        return Action::Keep;
                    }
                })
                .inner
            }
            NodeType::Collection => {
                let mut head = egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id_source,
                    true,
                );
                if let Some(flag) = open {
                    head.set_open(flag);
                }
                let (_, head_rep, body_resp) = head
                    .show_header(ui, |ui| {
                        if let Some(h) = handler {
                            h.ui(ui, self, |ui| {
                                ui.label("🗁");
                            });
                        }
                        // ui.label(
                        //     RichText::new("(🆔:".to_owned() + &self.id.to_string() + ")")
                        //         .color(egui::Color32::RED),
                        // );
                        let mut context_resp = Option::None;
                        let select_resp = ui
                            .toggle_value(&mut selected, self.title.clone())
                            .context_menu(|ui| {
                                if ui.button("添加集合").clicked() {
                                    ui.close_menu();
                                    context_resp =
                                        Some(Action::Add((vec![self.id], NodeType::Collection)));
                                }
                                if ui.button("添加节点").clicked() {
                                    ui.close_menu();
                                    context_resp =
                                        Some(Action::Add((vec![self.id], NodeType::Node)));
                                }
                                if ui.button("粘贴节点").clicked() {
                                    ui.close_menu();
                                    context_resp = Some(Action::Parse(vec![self.id]));
                                }
                                if ui.button("重命名").clicked() {
                                    ui.close_menu();
                                    context_resp = Some(Action::Rename(vec![self.id]));
                                }
                                if self.id != 0 && ui.button("删除").clicked() {
                                    ui.close_menu();
                                    context_resp = Some(Action::Delete(vec![self.id]));
                                }
                            });
                        if context_resp.is_some() {
                            return context_resp.unwrap();
                        }
                        if select_resp.clicked() {
                            return Action::Selected((vec![self.id], self.title.clone()));
                        } else {
                            return Action::Keep;
                        }
                    })
                    .body(|ui| self.sub_ui(ui, selected_str, flilter, open));
                match (head_rep.inner, body_resp) {
                    (Action::Keep, Some(InnerResponse { inner, .. })) => inner,
                    (head, _) => head,
                    // (
                    //     _,
                    //     Some(InnerResponse {
                    //         inner: Action::Delete(del_ids),
                    //         ..
                    //     }),
                    // ) => Action::Delete(del_ids),
                    // _ => Action::Keep,
                }
            }
        }
    }

    pub fn sub_ui(
        &mut self,
        ui: &mut Ui,
        selected_str: u64,
        flilter: &str,
        open: Option<bool>,
    ) -> Action {
        let mut sub_resp = Action::Keep;
        let drag_resp = self
            .drag_drop_ui
            .ui(ui, self.sub_items.iter_mut(), |item, ui, handler| {
                // if item.title.contains(flilter) {
                match item.ui_impl(ui, selected_str, flilter, open, Some(handler)) {
                    Action::Delete(mut d) => {
                        d.push(self.id);
                        sub_resp = Action::Delete(d);
                    }
                    Action::Add((mut a, t)) => {
                        a.push(self.id);
                        sub_resp = Action::Add((a, t));
                    }
                    Action::Rename(mut r) => {
                        r.push(self.id);
                        sub_resp = Action::Rename(r);
                    }
                    Action::Selected((mut ids, x)) => {
                        ids.push(self.id);
                        sub_resp = Action::Selected((ids, x));
                    }
                    Action::Copy(cop) => {
                        sub_resp = Action::Copy(cop);
                    }
                    Action::Parse(mut parse) => {
                        parse.push(self.id);
                        sub_resp = Action::Parse(parse);
                    }
                    _ => {}
                }
                // }
            });

        if let Some(response) = drag_resp.completed {
            info!("{}-{}", response.from, response.to);
            shift_vec(response.from, response.to, &mut self.sub_items);
        }

        sub_resp
        // for sub in self.sub_items.iter_mut() {
        //     if sub.title.contains(flilter) {
        //         match sub.ui_impl(ui, selected_str, flilter) {
        //             Action::Keep => continue,
        //             Action::Delete(mut d) => {
        //                 d.push(self.id);
        //                 return Action::Delete(d);
        //             }
        //             Action::Add((mut a, t)) => {
        //                 a.push(self.id);
        //                 return Action::Add((a, t));
        //             }
        //             Action::Rename(mut r) => {
        //                 r.push(self.id);
        //                 return Action::Rename(r);
        //             }
        //             Action::Selected(x) => return Action::Selected(x),
        //         }
        //     }
        // }
        // Action::Keep
    }
}

impl DragDropItem for TreeUiNode {
    fn id(&self) -> Id {
        Id::new(self.id)
    }
}
