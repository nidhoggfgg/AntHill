use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Execution {
    pub id: String,
    pub plugin_id: String,
    pub status: ExecutionStatus,
    pub pid: Option<i32>,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[repr(i32)]
pub enum ExecutionStatus {
    Pending = 0,
    Running = 1,
    Completed = 2,
    Failed = 3,
    Stopped = 4,
}
