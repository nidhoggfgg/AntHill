use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub plugin_type: PluginType,
    pub description: String,
    pub author: String,
    pub code: String,
    pub entry_point: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[repr(i32)]
pub enum PluginType {
    Python = 0,
    JavaScript = 1,
}
