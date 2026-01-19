use crate::error::{AppError, Result};
use crate::executor::{NodeExecutor, PluginExecutor, PythonExecutor};
use crate::models::{Execution, ExecutionStatus};
use crate::repository::{ExecutionRepository, PluginRepository};
use std::collections::HashMap;

#[derive(Clone)]
pub struct ExecutionService {
    exec_repo: ExecutionRepository,
    plugin_repo: PluginRepository,
    python_executor: PythonExecutor,
    node_executor: NodeExecutor,
}

impl ExecutionService {
    pub fn new(exec_repo: ExecutionRepository, plugin_repo: PluginRepository) -> Self {
        Self {
            exec_repo,
            plugin_repo,
            python_executor: PythonExecutor::default(),
            node_executor: NodeExecutor::default(),
        }
    }

    pub async fn execute_plugin(
        &self,
        plugin_id: &str,
        args: Vec<String>,
        env: HashMap<String, String>,
    ) -> Result<Execution> {
        // Get plugin
        let plugin = self.plugin_repo.get(plugin_id).await?;

        if !plugin.enabled {
            return Err(AppError::PluginDisabled);
        }

        // Create execution record
        let execution = self.exec_repo.create(plugin_id).await?;

        // Select executor based on plugin type
        let (pid, mut child) = match plugin.plugin_type {
            crate::models::PluginType::Python => {
                self.python_executor.execute(&plugin, args, env).await?
            }
            crate::models::PluginType::JavaScript => {
                self.node_executor.execute(&plugin, args, env).await?
            }
        };

        // Update execution with pid
        self.exec_repo.update_pid(&execution.id, pid).await?;

        // Spawn background task to monitor execution
        let exec_id = execution.id.clone();
        let exec_repo_clone = self.exec_repo.clone();

        tokio::spawn(async move {
            // Wait for process to complete
            let status_result = child.wait().await;

            match status_result {
                Ok(status) => {
                    let exit_code = status.code();

                    // Read stdout and stderr
                    // TODO: Implement proper output capture
                    let (stdout, stderr) = match exit_code {
                        Some(0) => (Some("Execution completed successfully".to_string()), None),
                        _ => (None, Some("Execution failed".to_string())),
                    };

                    let exec_status = if exit_code == Some(0) {
                        ExecutionStatus::Completed
                    } else {
                        ExecutionStatus::Failed
                    };

                    exec_repo_clone
                        .update_result(&exec_id, stdout, stderr, exit_code, exec_status)
                        .await
                        .ok();
                }
                Err(e) => {
                    tracing::error!("Error waiting for process: {}", e);
                    exec_repo_clone
                        .update_result(
                            &exec_id,
                            None,
                            Some(format!("Error: {}", e)),
                            None,
                            ExecutionStatus::Failed,
                        )
                        .await
                        .ok();
                }
            }
        });

        Ok(execution)
    }

    pub async fn get_execution(&self, id: &str) -> Result<Execution> {
        self.exec_repo.get(id).await
    }

    pub async fn list_executions(&self, plugin_id: Option<String>) -> Result<Vec<Execution>> {
        if let Some(pid) = plugin_id {
            self.exec_repo.list_by_plugin(&pid).await
        } else {
            self.exec_repo.list_all().await
        }
    }

    pub async fn stop_execution(&self, id: &str) -> Result<()> {
        let execution = self.exec_repo.get(id).await?;

        if let Some(pid) = execution.pid {
            // Try to kill the process
            // TODO: Implement proper process management
            tracing::info!("Stopping execution {} with pid {}", id, pid);
        }

        self.exec_repo
            .update_status(id, ExecutionStatus::Stopped)
            .await?;

        Ok(())
    }
}
