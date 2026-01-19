use super::PluginExecutor;
use crate::error::{AppError, Result};
use crate::models::Plugin;
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;

#[derive(Clone)]
pub struct PythonExecutor {
    python_path: String,
}

impl PythonExecutor {
    pub fn new(python_path: Option<String>) -> Self {
        Self {
            python_path: python_path.unwrap_or_else(|| {
                if cfg!(target_os = "windows") {
                    "python".to_string()
                } else {
                    "python3".to_string()
                }
            }),
        }
    }
}

impl Default for PythonExecutor {
    fn default() -> Self {
        Self::new(None)
    }
}

impl PluginExecutor for PythonExecutor {
    async fn execute(
        &self,
        plugin: &Plugin,
        args: Vec<String>,
        env: HashMap<String, String>,
    ) -> Result<(u32, tokio::process::Child)> {
        // Write plugin code to a temporary file
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| AppError::Execution(format!("Failed to create temp file: {}", e)))?;

        temp_file
            .write_all(plugin.code.as_bytes())
            .map_err(|e| AppError::Execution(format!("Failed to write code: {}", e)))?;

        let temp_path = temp_file
            .path()
            .to_str()
            .ok_or_else(|| AppError::Execution("Invalid temp file path".to_string()))?
            .to_string();

        // Keep the temp file until the process completes
        temp_file
            .keep()
            .map_err(|e| AppError::Execution(format!("Failed to keep temp file: {}", e)))?;

        // Build the command
        let mut cmd = tokio::process::Command::new(&self.python_path);
        cmd.arg(temp_path);

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
