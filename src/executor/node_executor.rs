use super::PluginExecutor;
use crate::error::{AppError, Result};
use crate::models::Plugin;
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;

#[derive(Clone)]
pub struct NodeExecutor {
    node_path: String,
}

impl NodeExecutor {
    pub fn new(node_path: Option<String>) -> Self {
        Self {
            node_path: node_path.unwrap_or_else(|| {
                if cfg!(target_os = "windows") {
                    "node".to_string()
                } else {
                    "node".to_string()
                }
            }),
        }
    }
}

impl Default for NodeExecutor {
    fn default() -> Self {
        Self::new(None)
    }
}

impl PluginExecutor for NodeExecutor {
    async fn execute(
        &self,
        plugin: &Plugin,
        args: Vec<String>,
        env: HashMap<String, String>,
    ) -> Result<(u32, tokio::process::Child)> {
        // Write plugin code to a temporary file
        let temp_file = NamedTempFile::new()
            .map_err(|e| AppError::Execution(format!("Failed to create temp file: {}", e)))?;

        let extension = if plugin.entry_point.ends_with(".mjs") {
            ".mjs"
        } else {
            ".js"
        };

        // Create a new temp file with the correct extension
        let temp_path = temp_file.into_temp_path();
        let file_path = temp_path.with_extension(extension);

        let mut file = std::fs::File::create(&file_path)
            .map_err(|e| AppError::Execution(format!("Failed to create temp file: {}", e)))?;

        file.write_all(plugin.code.as_bytes())
            .map_err(|e| AppError::Execution(format!("Failed to write code: {}", e)))?;

        let file_str = file_path
            .to_str()
            .ok_or_else(|| AppError::Execution("Invalid temp file path".to_string()))?;

        // Build the command
        let mut cmd = tokio::process::Command::new(&self.node_path);
        cmd.arg(file_str);

        for arg in args {
            cmd.arg(arg);
        }

        // Set environment variables
        for (key, value) in env {
            cmd.env(key, value);
        }

        // Capture stdout and stderr
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let child = cmd.spawn()?;

        let pid = child
            .id()
            .ok_or_else(|| AppError::Execution("Failed to get process ID".to_string()))?;

        Ok((pid, child))
    }
}
