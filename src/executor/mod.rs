pub mod node_executor;
pub mod python_executor;

pub use node_executor::NodeExecutor;
pub use python_executor::PythonExecutor;

use crate::error::Result;
use crate::models::Plugin;
use std::collections::HashMap;
use std::path::Path;

pub(crate) trait PluginExecutor {
    async fn execute(
        &self,
        plugin: &Plugin,
        args: Vec<String>,
        env: HashMap<String, String>,
        work_dir: &Path,
    ) -> Result<(u32, tokio::process::Child)>;
}
