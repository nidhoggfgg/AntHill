use super::PluginExecutor;
use crate::error::{AppError, Result};
use crate::models::Plugin;
use std::collections::HashMap;
use std::path::Path;

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
        work_dir: &Path,
    ) -> Result<(u32, tokio::process::Child)> {
        let script_path = Path::new(&plugin.plugin_path).join(&plugin.entry_point);
        if !script_path.is_file() {
            return Err(AppError::Execution(format!(
                "Entry point not found: {}",
                script_path.display()
            )));
        }

        // Build the command
        let mut cmd = tokio::process::Command::new(&self.node_path);
        cmd.arg(&script_path);
        cmd.current_dir(work_dir);

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
