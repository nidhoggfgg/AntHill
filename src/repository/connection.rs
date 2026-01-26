use crate::repository::DbPool;
use anyhow::Result;

pub async fn establish_connection(database_url: &str) -> Result<DbPool> {
    // Ensure the database URL has the correct format
    let db_url = if database_url.starts_with("sqlite:") {
        database_url.to_string()
    } else {
        format!("sqlite:{}", database_url)
    };

    // Create connection with create_if_missing option
    let connection_string = format!("{}?mode=rwc", db_url);
    let pool = sqlx::SqlitePool::connect(&connection_string).await?;

    // Run migrations
    sqlx::query(
        r#"
        -- 插件表
        CREATE TABLE IF NOT EXISTS plugins (
            id TEXT PRIMARY KEY,
            plugin_id TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            version TEXT NOT NULL,
            plugin_type INTEGER NOT NULL,
            description TEXT,
            author TEXT,
            plugin_path TEXT NOT NULL,
            entry_point TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            parameters TEXT,
            python_venv_path TEXT,
            python_dependencies TEXT
        );

        -- 执行记录表
        CREATE TABLE IF NOT EXISTS executions (
            id TEXT PRIMARY KEY,
            plugin_id TEXT NOT NULL,
            status INTEGER NOT NULL,
            pid INTEGER,
            exit_code INTEGER,
            stdout TEXT,
            stderr TEXT,
            started_at INTEGER NOT NULL,
            finished_at INTEGER,
            FOREIGN KEY (plugin_id) REFERENCES plugins(plugin_id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_executions_plugin_id ON executions(plugin_id);
        CREATE INDEX IF NOT EXISTS idx_plugins_enabled ON plugins(enabled);
        CREATE INDEX IF NOT EXISTS idx_plugins_plugin_id ON plugins(plugin_id);
        CREATE INDEX IF NOT EXISTS idx_plugins_name ON plugins(name);
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
