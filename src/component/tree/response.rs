use std::{collections::HashSet, path::PathBuf};

use anyhow::Error;
use uuid::Uuid;

#[derive(Default,Debug)]
pub struct NodeResponse {
    pub open_requests: HashSet<Uuid>,
    pub new_file: Option<Uuid>,
    pub new_drawing: Option<bool>,
    // pub export_file: Option<Result<(Uuid, PathBuf),Error>>,
    pub new_folder_modal: Option<Uuid>,
    // pub create_share_modal: Option<lb::File>,
    pub rename_request: Option<(Uuid, String)>,
    pub delete_request: bool,
    pub dropped_on: Option<Uuid>,
}

impl NodeResponse {
    pub fn union(self, other: Self) -> Self {
        let mut this = self;
        this.new_file = this.new_file.or(other.new_file);
        this.new_drawing = this.new_drawing.or(other.new_drawing);
        this.new_folder_modal = this.new_folder_modal.or(other.new_folder_modal);
        // this.create_share_modal = this.create_share_modal.or(other.create_share_modal);
        // this.export_file = this.export_file.or(other.export_file);
        this.open_requests.extend(other.open_requests);
        this.rename_request = this.rename_request.or(other.rename_request);
        this.delete_request = this.delete_request || other.delete_request;
        this.dropped_on = this.dropped_on.or(other.dropped_on);
        this
    }
}
