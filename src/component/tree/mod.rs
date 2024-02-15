mod node;
mod response;
mod state;

pub use self::node::TreeNode;

use eframe::egui;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub use self::node::DocType;

use self::response::NodeResponse;
use self::state::*;

#[derive(Debug,Serialize,Deserialize)]
pub struct TreeView {
    pub root: TreeNode,
    pub state: TreeState,
}

impl Default for TreeView {
    fn default() -> Self {
        TreeView::new()
    }
}

impl TreeView {
    pub fn new() -> Self {
        let root = TreeNode::new("根路径", None, 0);

        let state = TreeState::default();

        Self { root, state }
    }

    pub fn move_selected(&mut self,id: &Uuid) {
        for sid in self.state.selected.iter() {
            if let Some(node) = self.root.remove_rec(sid.to_owned()) {
                if let Some(snode) = self.root.find_mut(id.to_owned()) {
                    snode.insert_node(node) 
                } 
            } 
        }
    }

    pub fn expand_to(&mut self, id: Uuid) {
        if let Some(node) = self.root.find(id) {
            // Select only the target file.
            self.state.selected.clear();
            self.state.selected.insert(id);

            // Expand all target file parents.
            let mut id = node.parent_id.unwrap();
            while let Some(node) = self.root.find(id) {
                self.state.expanded.insert(id);
                if node.id == self.root.id {
                    break;
                }
                id = node.parent_id.unwrap();
            }
        } else {
            eprintln!("couldn't find node with id {}", id);
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> NodeResponse {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
        let mut is_hovered = false;
        let mut r = egui::Frame::none().show(ui, |ui| {
            let result = self.root.show(ui, &mut self.state);
            is_hovered = result.response.hovered();
            result.inner
        });

        let empty_space_res = ui.interact(
            ui.available_rect_before_wrap(),
            egui::Id::from("tree-empty-space"),
            egui::Sense::click(),
        );

        empty_space_res.context_menu(|ui| {
            ui.spacing_mut().button_padding = egui::vec2(4.0, 4.0);

            if ui.button("新建文件").clicked() {
                r.inner.new_file = Some(self.root.id.clone());
                ui.close_menu();
            }
            // if ui.button("New Drawing").clicked() {
            //     r.inner.new_drawing = Some(true);
            //     ui.close_menu();
            // }
            if ui.button("新建文件夹").clicked() {
                r.inner.new_folder_modal = Some(self.root.id.clone());
                ui.close_menu();
            }
        });

        if self.state.is_dragging() {
            if ui.input(|i| i.pointer.any_released()) {
                let maybe_pos = ui.ctx().pointer_interact_pos();
                self.state.dropped(maybe_pos);
            } else {
                self.draw_drag_info_by_cursor(ui);
            }
        } else if is_hovered && ui.input(|i| i.pointer.primary_down()) {
            // todo(steve): prep drag only if a file is clicked
            self.state.dnd.is_primary_down = true;
            if ui.input(|i| i.pointer.is_moving()) {
                self.state.dnd.has_moved = true;
            }
        }
        ui.expand_to_include_rect(ui.available_rect_before_wrap());

        // while let Ok(update) = self.state.update_rx.try_recv() {
        //     match update {
        //         TreeUpdate::RevealFileDone((expanded_files, selected)) => {
        //             self.state.request_scroll = true;

        //             expanded_files.iter().for_each(|f| {
        //                 self.state.expanded.insert(*f);
        //             });
        //             self.state.selected.clear();
        //             self.state.selected.insert(selected);
        //         }
        //         TreeUpdate::ExportFile((exported_file, dest)) => {
        //             dbg!("export File");
        //         }
        //     }
        // }
        r.inner
    }

    fn draw_drag_info_by_cursor(&mut self, ui: &mut egui::Ui) {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);

        // Paint a caption under the cursor in a layer above.
        let layer_id = egui::LayerId::new(egui::Order::Tooltip, self.state.id);

        let hover_pos = ui.input(|i| i.pointer.hover_pos().unwrap());
        let mut end = hover_pos;
        end.x += 70.0;
        end.y += 50.0;

        let response = ui
            .allocate_ui_at_rect(egui::Rect::from_two_pos(hover_pos, end), |ui| {
                ui.with_layer_id(layer_id, |ui| {
                    egui::Frame::none()
                        .rounding(3.0)
                        .inner_margin(1.0)
                        .fill(ui.visuals().widgets.active.fg_stroke.color)
                        .show(ui, |ui| {
                            egui::Frame::none()
                                .rounding(3.0)
                                .inner_margin(egui::style::Margin::symmetric(12.0, 7.0))
                                .fill(ui.visuals().window_fill)
                                .show(ui, |ui| {
                                    ui.label(self.state.drag_caption());
                                });
                        });
                })
            })
            .response;

        if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
            // todo: make sure dragging doesn't expand scroll area to infinity and beyond. respect the initial max width and height;

            if pointer_pos.y < 30.0 {
                ui.scroll_with_delta(egui::vec2(0., 30.0));
            }
            if pointer_pos.y < 100.0 {
                ui.scroll_with_delta(egui::vec2(0., 10.0));
            }
            ui.scroll_to_rect(response.rect, None);
        }
    }

    pub fn remove(&mut self, f: Uuid) {
        if let Some(node) = self.root.find_mut(f) {
            if let Some(mut removed) = node.remove(f) {
                clear_children(&mut self.state, &mut removed);
            }
        }
    }


    pub fn add(&mut self, new_node: TreeNode, parent: Option<Uuid>) {
        if let Some(parent_id) = parent {
            if let Some(node) = self.root.find_mut(parent_id) {
                node.add_child(new_node);
            }
        } else {
            self.root.add_child(new_node);
        }
    }

    // pub fn get_selected_files(&self) -> Vec<lb::File> {
    //     self.state
    //         .selected
    //         .iter()
    //         .map(|id| self.root.find(*id).unwrap().file.clone())
    //         .collect()
    // }

}


fn clear_children(state: &mut TreeState, node: &mut TreeNode) {
    state.selected.remove(&node.id);
    for child in &mut node.children {
        clear_children(state, child);
    }
}
