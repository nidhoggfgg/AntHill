pub mod node_executor;
pub mod process_manager;
pub mod python_executor;

pub use node_executor::NodeExecutor;
pub use process_manager::ProcessManager;
pub use python_executor::PythonExecutor;

use crate::error::Result;
use crate::models::Plugin;
use std::collections::HashMap;
use std::path::Path;

pub struct ExecutionOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

pub trait PluginExecutor {
    async fn execute(
        &self,
        plugin: &Plugin,
        args: Vec<String>,
        env: HashMap<String, String>,
        work_dir: &Path,
    ) -> Result<(u32, tokio::process::Child)>;
}
