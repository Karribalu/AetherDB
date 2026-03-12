//! Runtime configuration loaded from environment variables or a config file.
//!
//! All fields have explicit defaults so the server starts with zero config for
//! development. Production deployments override via environment variables or a
//! JSON config file passed via `--config`.

use serde::{Deserialize, Serialize};

/// Top-level AetherDB configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Object storage configuration.
    pub storage: StorageConfig,

    /// NVMe cache configuration.
    pub cache: CacheConfig,

    /// HTTP server configuration.
    pub server: ServerConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageConfig::default(),
            cache: CacheConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    /// Object storage bucket name.
    pub bucket: String,

    /// Object storage endpoint URL (leave empty for AWS default).
    pub endpoint: Option<String>,

    /// Key prefix inside the bucket for all AetherDB objects.
    pub prefix: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            bucket: String::from("aetherdb"),
            endpoint: None,
            prefix: String::from("data/"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    /// Local directory for the NVMe cache.
    pub dir: String,

    /// Maximum cache size in bytes. Default 10 GiB.
    pub max_bytes: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            dir: String::from("/tmp/aetherdb-cache"),
            max_bytes: 10 * 1024 * 1024 * 1024, // 10 GiB
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Address to bind the HTTP server.
    pub bind: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: String::from("0.0.0.0:7700"),
        }
    }
}
