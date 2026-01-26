use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UpdateRequest {
    pub package_url: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateResponse {
    pub status: String,
    pub restart_required: bool,
    pub current_version: String,
    pub package_version: String,
}
