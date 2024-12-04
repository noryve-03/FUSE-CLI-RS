/// Configuration Management Module
///
/// This module handles all configuration aspects of the tool, including storage
/// settings, mount options, and transfer parameters. It uses serde for JSON
/// serialization/deserialization of configuration files.
///
/// Configuration Structure:
/// 1. Storage Configuration
///    - Provider selection (S3)
///    - Region and endpoint settings
///    - Bucket configuration
///
/// 2. Mount Options
///    - Cache size and timeout settings
///    - Read-only mode configuration
///
/// 3. Transfer Options
///    - Concurrent transfer limits
///    - Chunk size configuration
///    - Retry settings
///
/// The configuration can be loaded from a file or environment variables,
/// with sensible defaults provided when no configuration is specified.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{Result, ToolError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default_storage: StorageConfig,
    pub mount_options: MountOptions,
    pub transfer_options: TransferOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    pub provider: StorageProvider,
    pub region: Option<String>,
    pub endpoint: Option<String>,
    pub bucket: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StorageProvider {
    #[serde(rename = "s3")]
    S3,
    #[serde(rename = "gcs")]
    GCS,
    #[serde(rename = "azure")]
    Azure,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MountOptions {
    pub cache_size_mb: u64,
    pub timeout_seconds: u64,
    pub read_only: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferOptions {
    pub concurrent_uploads: usize,
    pub chunk_size: usize,
    pub retry_attempts: u32,
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> Result<Self> {
        let config_path = path.or_else(|| {
            dirs::config_dir().map(|mut p| {
                p.push("mytool");
                p.push("config.json");
                p
            })
        }).ok_or_else(|| ToolError::Config("Could not determine config path".into()))?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(config_path)
            .map_err(|e| ToolError::Config(format!("Failed to read config file: {}", e)))?;

        serde_json::from_str(&contents)
            .map_err(|e| ToolError::Config(format!("Failed to parse config file: {}", e)))
    }

    pub fn save(&self, path: Option<PathBuf>) -> Result<()> {
        let config_path = path.or_else(|| {
            dirs::config_dir().map(|mut p| {
                p.push("mytool");
                p.push("config.json");
                p
            })
        }).ok_or_else(|| ToolError::Config("Could not determine config path".into()))?;

        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ToolError::Config(format!("Failed to create config directory: {}", e)))?;
        }

        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| ToolError::Config(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(config_path, contents)
            .map_err(|e| ToolError::Config(format!("Failed to write config file: {}", e)))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_storage: StorageConfig {
                provider: StorageProvider::S3,
                region: Some("us-east-1".to_string()),
                endpoint: None,
                bucket: None,
                access_key_id: None,
                secret_access_key: None,
            },
            mount_options: MountOptions {
                cache_size_mb: 1024,
                timeout_seconds: 300,
                read_only: true,
            },
            transfer_options: TransferOptions {
                concurrent_uploads: 4,
                chunk_size: 8 * 1024 * 1024, // 8MB
                retry_attempts: 3,
            },
        }
    }
}