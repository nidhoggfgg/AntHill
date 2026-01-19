use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<u32, tokio::process::Child>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_process(&self, pid: u32, child: tokio::process::Child) {
        self.processes.write().await.insert(pid, child);
    }

    pub async fn stop_process(&self, pid: u32) -> std::io::Result<()> {
        if let Some(mut child) = self.processes.write().await.remove(&pid) {
            child.kill().await?;
        }
        Ok(())
    }

    pub async fn is_running(&self, pid: u32) -> bool {
        self.processes.read().await.contains_key(&pid)
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
