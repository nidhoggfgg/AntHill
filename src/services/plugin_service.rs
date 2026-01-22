use crate::error::Result;
use crate::models::{Plugin, PluginParameter, PluginType};
use crate::repository::PluginRepository;
use crate::paths;
use chrono::Utc;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
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

    #[allow(unused)]
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
        package_url: String,
        entry_point: String,
        metadata: Option<String>,
        parameters: Option<Vec<PluginParameter>>,
    ) -> Result<Plugin> {
        // Check if plugin already exists
        if self.repo.get_by_name(&name).await.is_ok() {
            return Err(crate::error::AppError::PluginAlreadyExists(name));
        }

        if entry_point.trim().is_empty() {
            return Err(crate::error::AppError::Execution(
                "Entry point cannot be empty".to_string(),
            ));
        }
        Self::validate_entry_point(&entry_point)?;

        let plugin_id = Uuid::new_v4().to_string();
        let plugin_dir = Self::plugin_dir_for(&plugin_id)?;

        fs::create_dir_all(&plugin_dir)?;

        let parameters_json = Self::validate_parameters(parameters)?;

        if let Err(err) = self.download_and_extract(&package_url, &plugin_dir).await {
            let _ = fs::remove_dir_all(&plugin_dir);
            return Err(err);
        }

        let entry_path = plugin_dir.join(&entry_point);
        if !entry_path.is_file() {
            let _ = fs::remove_dir_all(&plugin_dir);
            return Err(crate::error::AppError::Execution(format!(
                "Entry point not found: {}",
                entry_path.display()
            )));
        }

        let now = Utc::now();
        let plugin = Plugin {
            id: plugin_id,
            name,
            version,
            plugin_type,
            description,
            author,
            plugin_path: plugin_dir.to_string_lossy().to_string(),
            entry_point,
            enabled: true,
            created_at: now,
            updated_at: now,
            metadata,
            parameters: parameters_json,
        };

        self.repo.create(&plugin).await?;
        Ok(plugin)
    }

    pub async fn uninstall_plugin(&self, id: &str) -> Result<()> {
        let plugin = self.repo.get(id).await?;
        if !plugin.plugin_path.is_empty() {
            match fs::remove_dir_all(&plugin.plugin_path) {
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => return Err(err.into()),
            }
        }
        self.repo.delete(id).await
    }

    pub async fn enable_plugin(&self, id: &str) -> Result<()> {
        self.repo.update_enabled(id, true).await
    }

    pub async fn disable_plugin(&self, id: &str) -> Result<()> {
        self.repo.update_enabled(id, false).await
    }

    fn plugin_dir_for(plugin_id: &str) -> Result<PathBuf> {
        let base_dir = paths::plugins_dir()?;
        Ok(base_dir.join(plugin_id))
    }

    async fn download_and_extract(&self, url: &str, target_dir: &Path) -> Result<()> {
        if let Some(path) = Self::local_path_from_url(url) {
            let bytes = fs::read(&path).map_err(|e| {
                crate::error::AppError::Execution(format!(
                    "Failed to read local package {}: {}",
                    path.display(),
                    e
                ))
            })?;
            return Self::extract_zip(&bytes, target_dir);
        }

        let response = reqwest::get(url).await.map_err(|e| {
            crate::error::AppError::Execution(format!("Failed to download package: {}", e))
        })?;
        let response = response.error_for_status().map_err(|e| {
            crate::error::AppError::Execution(format!("Failed to download package: {}", e))
        })?;

        let bytes = response.bytes().await.map_err(|e| {
            crate::error::AppError::Execution(format!("Failed to read package bytes: {}", e))
        })?;

        Self::extract_zip(&bytes, target_dir)
    }

    fn extract_zip(bytes: &[u8], target_dir: &Path) -> Result<()> {
        let reader = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader).map_err(|e| {
            crate::error::AppError::Execution(format!("Invalid zip archive: {}", e))
        })?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                crate::error::AppError::Execution(format!("Failed to read archive: {}", e))
            })?;

            let Some(relative_path) = file.enclosed_name().as_deref().map(Path::to_path_buf) else {
                return Err(crate::error::AppError::Execution(
                    "Invalid file path in archive".to_string(),
                ));
            };

            let out_path = target_dir.join(relative_path);
            if file.name().ends_with('/') {
                fs::create_dir_all(&out_path)?;
                continue;
            }

            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut outfile = fs::File::create(&out_path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            outfile.write_all(&buffer)?;
        }

        Ok(())
    }

    fn local_path_from_url(url: &str) -> Option<PathBuf> {
        if let Some(path) = url.strip_prefix("file://") {
            let path = path.strip_prefix("localhost/").unwrap_or(path);
            return Some(PathBuf::from(path));
        }
        None
    }

    fn validate_entry_point(entry_point: &str) -> Result<()> {
        let path = Path::new(entry_point);
        if path.is_absolute() {
            return Err(crate::error::AppError::Execution(
                "Entry point must be a relative path".to_string(),
            ));
        }
        if path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(crate::error::AppError::Execution(
                "Entry point cannot contain '..'".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_parameters(
        parameters: Option<Vec<PluginParameter>>,
    ) -> Result<Option<String>> {
        let Some(parameters) = parameters else {
            return Ok(None);
        };

        let mut seen = std::collections::HashSet::new();
        for param in &parameters {
            let name = param.name.trim();
            if name.is_empty() {
                return Err(crate::error::AppError::Execution(
                    "Parameter name cannot be empty".to_string(),
                ));
            }
            if name != param.name {
                return Err(crate::error::AppError::Execution(format!(
                    "Parameter name has leading/trailing whitespace: {}",
                    param.name
                )));
            }
            if !seen.insert(name.to_string()) {
                return Err(crate::error::AppError::Execution(format!(
                    "Duplicate parameter name: {}",
                    name
                )));
            }
            if let Some(default) = &param.default {
                if !param.param_type.matches(default) {
                    return Err(crate::error::AppError::Execution(format!(
                        "Default value for parameter '{}' does not match type {:?}",
                        name, param.param_type
                    )));
                }
            }
        }

        let json = serde_json::to_string(&parameters).map_err(|e| {
            crate::error::AppError::Execution(format!(
                "Failed to serialize parameters: {}",
                e
            ))
        })?;
        Ok(Some(json))
    }
}
