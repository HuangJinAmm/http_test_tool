use std::{
    collections::HashSet,
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
};

use eframe::egui;
use uuid::Uuid;

pub struct TreeState {
    pub id: egui::Id,
    pub max_node_width: f32,
    pub selected: HashSet<Uuid>,
    pub expanded: HashSet<Uuid>,
    pub renaming: NodeRenamingState,
    pub request_scroll: bool,
    pub dnd: TreeDragAndDropState,
    pub update_tx: Sender<TreeUpdate>,
    pub update_rx: Receiver<TreeUpdate>,
}

impl Default for TreeState {
    fn default() -> Self {
        let (update_tx, update_rx) = mpsc::channel();
        Self {
            id: egui::Id::new("filetree"),
            max_node_width: 0.0,
            selected: HashSet::new(),
            expanded: HashSet::new(),
            dnd: TreeDragAndDropState::default(),
            renaming: NodeRenamingState::default(),
            request_scroll: false,
            update_tx,
            update_rx,
        }
    }
}

pub enum TreeUpdate {
    RevealFileDone((Vec<Uuid>, Uuid)),
    ExportFile((Uuid, PathBuf)),
}

impl TreeState {
    pub fn toggle_selected(&mut self, id: Uuid) {
        if !self.selected.remove(&id) {
            self.selected.insert(id);
        }
    }

    pub fn is_dragging(&self) -> bool {
        self.dnd.is_primary_down && self.dnd.has_moved
    }

    pub fn dropped(&mut self, pos: Option<egui::Pos2>) {
        self.dnd.is_primary_down = false;
        self.dnd.has_moved = false;
        self.dnd.dropped = pos;
    }

    pub fn drag_caption(&self) -> String {
        let n = self.selected.len();
        format!("{} file{}", n, if n > 1 { "s" } else { "" })
    }
}

#[derive(Default)]
pub struct TreeDragAndDropState {
    pub is_primary_down: bool,
    pub has_moved: bool,
    pub dropped: Option<egui::Pos2>,
}

#[derive(Default)]
pub struct NodeRenamingState {
    pub id: Option<Uuid>,
    pub tmp_name: String,
}

impl NodeRenamingState {
    pub fn new(id:Uuid,name:&str) -> Self {
        Self { id: Some(id), tmp_name: name.to_owned() }
    }
}