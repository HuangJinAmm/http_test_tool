use std::cmp::Ordering;
use std::thread;

use eframe::egui;
use egui::text::CCursorRange;
use egui::TextStyle;
use hdrhistogram::iterators::all;
use uuid::Uuid;

use crate::component::theme::Icon;

use super::response::*;
use super::state::*;

#[derive(Debug)]
pub enum DocType {
    PlainText,
    Markdown,
    Drawing,
    Image(String),
    ImageUnsupported(String),
    Code(String),
    Unknown,
}

impl DocType {
    pub fn from_name(name: &str) -> Self {
        let ext = name.split('.').last().unwrap_or_default();
        match ext {
            "draw" => Self::Drawing,
            "md" => Self::Markdown,
            "txt" => Self::PlainText,
            "png" | "jpeg" | "jpg" | "gif" | "webp" | "bmp" | "ico" => Self::Image(ext.to_string()),
            "cr2" => Self::ImageUnsupported(ext.to_string()),
            "go" => Self::Code(ext.to_string()),
            _ => Self::Unknown,
        }
    }
    pub fn to_icon(&self) -> Icon {
        match self {
            DocType::Markdown | DocType::PlainText => Icon::DOC_TEXT,
            DocType::Drawing => Icon::DRAW,
            DocType::Image(_) => Icon::IMAGE,
            DocType::Code(_) => Icon::CODE,
            _ => Icon::DOC_UNKNOWN,
        }
    }
}

#[derive(Default,Debug)]
pub struct TreeNode {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub doc_type: Option<DocType>,
    pub children: Vec<TreeNode>,
    depth: u8,
    primary_press: Option<egui::Pos2>,
    hovering_drop: bool,
}

impl From<u8> for TreeNode {
    fn from(data: u8) -> Self {
        let depth = data;
        let doc_type = Some(DocType::PlainText);
        let name = "未命名".to_string();
        let id = Uuid::new_v4();
        Self {
            id,
            parent_id: None,
            name,
            doc_type,
            children: Vec::new(),
            depth,
            primary_press: None,
            hovering_drop: false,
        }
    }
}

impl TreeNode {

    pub fn new(name:&str,doc_type:Option<DocType>,depth:u8) -> Self {
        TreeNode { id: Uuid::new_v4(), parent_id: None, name: name.to_owned(), doc_type, children: Vec::new(), depth, primary_press: None, hovering_drop: false }
    }

    pub fn populate_from(&mut self, all_ids: &Vec<String>) {
        self.children = all_ids
            .iter()
            .filter_map(|f| {
                let mut node = TreeNode::from( self.depth + 1);
                node.name = f.clone();
                Some(node)
            })
            .collect();
        self.children.sort();
    }

    pub fn show(
        &mut self, ui: &mut egui::Ui, state: &mut TreeState,
    ) -> egui::InnerResponse<NodeResponse> {
        let (mut resp, mut node_resp) = if state.renaming.id == Some(self.id) {
            let mut node_resp = NodeResponse::default();
            let resp = ui
                .horizontal(|ui| {
                    ui.add_space(self.depth_inset() + 5.0);

                    let resp = egui::TextEdit::singleline(&mut state.renaming.tmp_name)
                        .margin(egui::vec2(6.0, 6.0))
                        .hint_text("Name...")
                        .id(egui::Id::new("rename_field"))
                        .show(ui)
                        .response;

                    if resp.lost_focus() || resp.clicked_elsewhere() {
                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            node_resp.rename_request =
                                Some((self.id, state.renaming.tmp_name.clone()));
                        }
                        state.renaming = NodeRenamingState::default();
                        ui.ctx().request_repaint();
                    } else if !resp.has_focus() {
                        resp.request_focus();
                    }
                })
                .response;

            (resp, node_resp)
        } else {
            self.draw_normal(ui, state)
        };

        // Draw any children, if expanded, and merge their responses.
        if state.expanded.contains(&self.id) {
            for node in self.children.iter_mut() {
                let child_resp = node.show(ui, state);
                node_resp = node_resp.union(child_resp.inner);
                resp = resp.union(child_resp.response);
            }
        }

        egui::InnerResponse::new(node_resp, resp)
    }

    fn draw_normal(
        &mut self, ui: &mut egui::Ui, state: &mut TreeState,
    ) -> (egui::Response, NodeResponse) {
        let mut node_resp = NodeResponse::default();

        let mut resp = self.draw_icon_and_text(ui, state);

        if state.selected.contains(&self.id) && state.request_scroll {
            ui.scroll_to_rect(resp.rect, None);
            state.request_scroll = false;
        }

        if resp.hovered()
            && ui.input(|i| i.pointer.any_pressed())
            && ui.input(|i| i.pointer.primary_down())
        {
            self.primary_press = ui.input(|i| i.pointer.press_origin());

            if ui.input(|i| i.modifiers.ctrl) {
                state.toggle_selected(self.id);
            } else if !state.selected.contains(&self.id) {
                state.selected.clear();
                state.selected.insert(self.id);
            }
        } else if ui.input(|i| i.pointer.any_released()) {
            if let Some(pos) = self.primary_press {
                // Mouse was released over an item on which it was originally pressed.
                if resp.hovered()
                    && resp.rect.contains(pos)
                    && state.selected.len() > 1
                    && state.selected.contains(&self.id)
                    && !ui.input(|i| i.modifiers.ctrl)
                {
                    state.selected.retain(|id| *id == self.id);
                }
            }
            self.primary_press = None;
        }

        resp = ui.interact(resp.rect, resp.id, egui::Sense::click());
        if resp.double_clicked() {
            if self.doc_type.is_none() {
                if !state.expanded.remove(&self.id) {
                    state.expanded.insert(self.id);
                }
            } else {
                node_resp.open_requests.insert(self.id); // Signal that this document was opened this frame.
            }
        }

        if let Some(pos) = state.dnd.dropped {
            if resp.rect.contains(pos) {
                node_resp.dropped_on =
                    Some(if self.doc_type.is_none() { self.id } else { self.parent_id.unwrap() });
                if self.doc_type.is_none() {
                    state.expanded.insert(self.id);
                }
                state.dnd.dropped = None;
            }
        }

        if let Some(r) = resp.context_menu(|ui| self.context_menu(ui, &mut node_resp, state)) {
            resp = r.response;
        };

        (resp, node_resp)
    }

    fn draw_icon_and_text(&mut self, ui: &mut egui::Ui, state: &mut TreeState) -> egui::Response {
        let text_height = ui.text_style_height(&egui::TextStyle::Body);
        let padding = ui.spacing().button_padding;

        let depth_inset = self.depth_inset() + 5.0;
        let wrap_width = ui.available_width();

        
        let icon = if self.doc_type.is_none() && state.expanded.contains(&self.id){
            let wt: egui::WidgetText = (&Icon::FOLDER_OPEN).into();
            wt.color(ui.visuals().hyperlink_color)
        } else if self.doc_type.is_none() {
            let wt: egui::WidgetText = (&Icon::FOLDER).into();
            wt.color(ui.visuals().hyperlink_color)
        } else {
            let wt: egui::WidgetText = (&self.icon()).into();
            wt.color(ui.visuals().text_color().gamma_multiply(0.6))
        };


        let icon = icon.into_galley(ui, Some(false), wrap_width, egui::TextStyle::Body);

        let text: egui::WidgetText = (&self.name).into();
        let text = text.into_galley(ui, Some(false), wrap_width, egui::TextStyle::Body);

        let width = (depth_inset + padding.x * 2.0 + icon.size().x + 5.0 + text.size().x)
            .max(ui.available_size_before_wrap().x);
        if width > state.max_node_width {
            state.max_node_width = width;
            ui.ctx().request_repaint();
        }

        let desired_size = egui::vec2(state.max_node_width, text_height + padding.y * 4.0);

        let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());
        if ui.is_rect_visible(rect) {
            let bg = if state.selected.contains(&self.id) {
                ui.visuals().code_bg_color.gamma_multiply(0.6)
            } else if resp.hovered() {
                ui.visuals().code_bg_color.gamma_multiply(0.3)
            } else {
                ui.visuals().panel_fill
            };

            ui.painter().rect(rect, 0.0, bg, egui::Stroke::NONE);

            let icon_pos =
                egui::pos2(rect.min.x + depth_inset, rect.center().y - icon.size().y / 2.0 );

            let text_pos = egui::pos2(
                rect.min.x + depth_inset + padding.x + icon.size().x,
                rect.center().y - 0.5 * text.size().y,
            );

            let visuals = ui.style().interact(&resp);

            ui.painter().galley(icon_pos, icon, visuals.text_color());
            ui.painter().galley(text_pos,text, visuals.text_color());
        }

        let is_drop_target = self.doc_type.is_none()
            && resp.hovered()
            && state.is_dragging()
            && !state.selected.contains(&self.id);

        if !self.hovering_drop && is_drop_target {
            self.hovering_drop = true;
            ui.ctx().request_repaint();
        } else if self.hovering_drop && !is_drop_target {
            self.hovering_drop = false;
            ui.ctx().request_repaint();
        }

        resp
    }

    fn depth_inset(&self) -> f32 {
        (self.depth as f32) * 20.0
    }

    fn icon(&self) -> Icon {
        if self.hovering_drop {
            Icon::ARROW_CIRCLE_DOWN
        } else if let Some(typ) = &self.doc_type {
            typ.to_icon()
        } else {
            Icon::FOLDER
        }
    }

    fn context_menu(
        &mut self, ui: &mut egui::Ui, node_resp: &mut NodeResponse, state: &mut TreeState,
    ) {
        state.selected.clear();
        state.selected.insert(self.id);

        if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            ui.close_menu();
        }

        ui.spacing_mut().button_padding = egui::vec2(4.0, 4.0);

        if ui.button("New Document").clicked() {
            node_resp.new_file = Some(self.id.clone());
            ui.close_menu();
        }

        // if ui.button("New Drawing").clicked() {
        //     node_resp.new_drawing = Some(true);
        //     ui.close_menu();
        // }

        if ui.button("New Folder").clicked() {
            node_resp.new_folder_modal = Some(self.id.clone());
            ui.close_menu();
        }

        ui.separator();

        if ui.button("Rename").clicked() {
            state.renaming = NodeRenamingState::new(self.id,&self.name);

            let name = &state.renaming.tmp_name;
            let end_pos = name.rfind('.').unwrap_or(name.len());

            let mut rename_edit_state = egui::text_edit::TextEditState::default();
            rename_edit_state.set_ccursor_range(Some(CCursorRange {
                primary: egui::text::CCursor::new(end_pos),
                secondary: egui::text::CCursor::new(0),
            }));
            egui::TextEdit::store_state(ui.ctx(), egui::Id::new("rename_field"), rename_edit_state);

            ui.close_menu();
        }

        if ui.button("Delete").clicked() {
            node_resp.delete_request = true;
            ui.close_menu();
        }

        ui.separator();

        // if ui.button("Export").clicked() {
        //     let update_tx = state.update_tx.clone();
        //     let exported_file = self.id.clone();
        //     thread::spawn(move || {
        //         if let Some(folder) = FileDialog::new().pick_folder() {
        //             update_tx
        //                 .send(TreeUpdate::ExportFile((exported_file, folder)))
        //                 .unwrap();
        //         }
        //     });
        //     ui.close_menu();
        // }

        // let share = ui.add(egui::Button::new(
        //     egui::RichText::new("Share").color(ui.style().visuals.hyperlink_color),
        // ));

        // if share.clicked() {
            // node_resp.create_share_modal = Some(self.id.clone());
            // ui.close_menu()
        // }
    }

    // pub fn insert(&mut self, data: TreeNode) -> bool {
    //     if let Some(parent) = self.find_mut(data.parent_id) {
    //         for (i, child) in parent.children.iter().enumerate() {
    //             if data < *child {
    //                 parent.children.insert(i, data);
    //                 return true;
    //             }
    //         }
    //         parent.children.push(data);
    //         return true;
    //     }
    //     false
    // }

    pub fn insert_node(&mut self, node: Self) {
        let mut node = node;
        node.parent_id = Some(self.id);
        node.depth = self.depth + 1;

        for (i, child) in self.children.iter().enumerate() {
            if node < *child {
                self.children.insert(i, node);
                return;
            }
        }

        self.children.push(node);
    }

    pub fn remove(&mut self, id: Uuid) -> Option<TreeNode> {
        for (i, node) in self.children.iter().enumerate() {
            if node.id == id {
                return Some(self.children.remove(i));
            }
        }
        None
    }

    pub fn find(&self, id: Uuid) -> Option<&TreeNode> {
        if self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(node) = child.find(id) {
                return Some(node);
            }
        }
        None
    }

    pub fn find_mut(&mut self, id: Uuid) -> Option<&mut TreeNode> {
        if self.id == id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(node) = child.find_mut(id) {
                return Some(node);
            }
        }
        None
    }

    pub fn add_child(&mut self, mut node: TreeNode) {
        node.parent_id = Some(self.id);
        node.depth = self.depth + 1;
        self.children.push(node);
    }
}

impl PartialEq for TreeNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TreeNode {}

impl Ord for TreeNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.doc_type.is_none() && other.doc_type.is_some() {
            Ordering::Less
        } else if other.doc_type.is_none() && self.doc_type.is_some() {
            Ordering::Greater
        } else {
            self.name.cmp(&other.name)
        }
    }
}

impl PartialOrd for TreeNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
