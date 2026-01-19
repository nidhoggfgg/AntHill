use crate::models::Plugin;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct InstallPluginRequest {
    pub name: String,
    pub version: String,
    pub plugin_type: String,
    pub description: String,
    pub author: String,
    pub code: String,
    pub entry_point: String,
    pub metadata: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PluginResponse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub plugin_type: String,
    pub description: String,
    pub author: String,
    pub entry_point: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Plugin> for PluginResponse {
    fn from(plugin: Plugin) -> Self {
        Self {
            id: plugin.id,
            name: plugin.name,
            version: plugin.version,
            plugin_type: format!("{:?}", plugin.plugin_type),
            description: plugin.description,
            author: plugin.author,
            entry_point: plugin.entry_point,
            enabled: plugin.enabled,
            created_at: plugin.created_at.to_rfc3339(),
            updated_at: plugin.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PluginsListResponse {
    pub data: Vec<PluginResponse>,
}
