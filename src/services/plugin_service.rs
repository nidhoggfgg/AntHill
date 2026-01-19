use crate::error::Result;
use crate::models::{Plugin, PluginType};
use crate::repository::PluginRepository;
use chrono::Utc;
use uuid::Uuid;

#[derive(Clone)]
pub struct PluginService {
    repo: PluginRepository,
}

impl PluginService {
    pub fn new(repo: PluginRepository) -> Self {
        Self { repo }
    }

    pub async fn list_plugins(&self) -> Result<Vec<Plugin>> {
        self.repo.list().await
    }

    pub async fn get_plugin(&self, id: &str) -> Result<Plugin> {
        self.repo.get(id).await
    }

    pub async fn get_plugin_by_name(&self, name: &str) -> Result<Plugin> {
        self.repo.get_by_name(name).await
    }

    pub async fn install_plugin(
        &self,
        name: String,
        version: String,
        plugin_type: PluginType,
        description: String,
        author: String,
        code: String,
        entry_point: String,
        metadata: Option<String>,
    ) -> Result<Plugin> {
        // Check if plugin already exists
        if let Ok(_) = self.repo.get_by_name(&name).await {
            return Err(crate::error::AppError::PluginAlreadyExists(name));
        }

        let now = Utc::now();
        let plugin = Plugin {
            id: Uuid::new_v4().to_string(),
            name,
            version,
            plugin_type,
            description,
            author,
            code,
            entry_point,
            enabled: true,
            created_at: now,
            updated_at: now,
            metadata,
        };

        self.repo.create(&plugin).await?;
        Ok(plugin)
    }

    pub async fn uninstall_plugin(&self, id: &str) -> Result<()> {
        self.repo.delete(id).await
    }

    pub async fn enable_plugin(&self, id: &str) -> Result<()> {
        self.repo.update_enabled(id, true).await
    }

    pub async fn disable_plugin(&self, id: &str) -> Result<()> {
        self.repo.update_enabled(id, false).await
    }
}
