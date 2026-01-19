use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "sqlite:atom_node.db".to_string(),
            host: "0.0.0.0".to_string(),
            port: 3000,
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            config.database_url = db_url;
        }

        if let Ok(host) = std::env::var("HOST") {
            config.host = host;
        }

        if let Ok(port) = std::env::var("PORT") {
            config.port = port.parse().unwrap_or(3000);
        }

        config
    }
}
