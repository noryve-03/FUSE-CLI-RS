/// S3 Storage Implementation Module
///
/// This module provides the core functionality for interacting with AWS S3 storage.
/// It implements high-level operations for file and directory management, including
/// upload, download, listing, and synchronization capabilities.
///
/// Key Features:
/// - File Operations: upload/download single files
/// - Directory Operations: recursive upload/download of directories
/// - Sync Operations: bidirectional sync between local and S3
/// - Metadata Management: file size and modification time tracking
///
/// The module uses two levels of abstraction:
/// 1. AWS SDK (aws-sdk-s3): Low-level S3 operations
/// 2. object_store: High-level storage abstractions
///
/// Implementation Details:
/// - Async/await for all operations
/// - Streaming for large file transfers
/// - Error handling with custom ToolError types
/// - Metadata-based file comparison for sync

use aws_sdk_s3::Client;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::config::Region;
use object_store::aws::AmazonS3Builder;
use object_store::{ObjectStore, path::Path as ObjectPath};
use futures_util::StreamExt;
use crate::error::{Result, ToolError};
use crate::config::StorageConfig;
use std::sync::Arc;
use tracing::{info, error};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use bytes::Bytes;
use futures::TryStreamExt;
use object_store::GetResult;

pub struct S3Storage {
    client: Client,
    store: Arc<dyn ObjectStore>,
    bucket: String,
}

impl S3Storage {
    pub async fn new(config: &StorageConfig) -> Result<Self> {
        let region_str = config.region.clone()
            .unwrap_or_else(|| "us-east-1".to_string());
        
        let region = Region::new(region_str.clone());
        let region_provider = RegionProviderChain::first_try(region.clone());

        let aws_config = aws_config::from_env()
            .region(region_provider)
            .load()
            .await;

        let client = Client::new(&aws_config);

        let bucket = config.bucket.clone()
            .ok_or_else(|| ToolError::Config("S3 bucket not specified".into()))?;

        info!("Building S3 storage with bucket: {} and region: {}", bucket, region_str);

        let store = AmazonS3Builder::new()
            .with_bucket_name(&bucket)
            .with_region(&region_str)
            .with_access_key_id("AKIA4SZHOBCV4RHZYNCY")
            .with_secret_access_key("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")
            .with_allow_http(true)
            .build()?;

        info!("Successfully initialized S3 storage");

        Ok(Self {
            client,
            store: Arc::new(store),
            bucket,
        })
    }

    pub async fn upload_file(&self, local_path: &std::path::Path, remote_path: &str) -> Result<()> {
        info!("Uploading file to S3: {}", remote_path);
        let contents = tokio::fs::read(local_path).await
            .map_err(|e| {
                error!("Error reading file: {}", e);
                ToolError::Io(e)
            })?;

        let remote = ObjectPath::from(remote_path);
        self.store.put(&remote, contents.into()).await
            .map_err(|e| {
                error!("Error uploading file to S3: {}", e);
                ToolError::Storage(e)
            })?;

        info!("Successfully uploaded file to S3: {}", remote_path);
        Ok(())
    }

    pub async fn download_file(&self, remote_path: &str, local_path: &std::path::Path) -> Result<()> {
        info!("Downloading file from S3: {}", remote_path);
        let remote = ObjectPath::from(remote_path);
        let data = self.store.get(&remote).await
            .map_err(|e| {
                error!("Error downloading file from S3: {}", e);
                ToolError::Storage(e)
            })?;
        
        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| {
                    error!("Error creating directory: {}", e);
                    ToolError::Io(e)
                })?;
        }

        tokio::fs::write(local_path, data.bytes().await?)
            .await
            .map_err(|e| {
                error!("Error writing file: {}", e);
                ToolError::Io(e)
            })?;

        info!("Successfully downloaded file from S3: {}", remote_path);
        Ok(())
    }

    pub async fn list_objects(&self, prefix: &str) -> Result<Vec<String>> {
        info!("Listing objects in S3 with prefix: {}", prefix);
        let path = ObjectPath::from(prefix);
        let mut objects = Vec::new();

        let mut list_stream = self.store.list(Some(&path)).await
            .map_err(|e| {
                error!("Error listing objects in S3: {}", e);
                ToolError::Storage(e)
            })?;
        
        while let Some(obj) = list_stream.next().await {
            let obj = obj
                .map_err(|e| {
                    error!("Error listing objects in S3: {}", e);
                    ToolError::Storage(e)
                })?;
            objects.push(obj.location.to_string());
        }

        info!("Successfully listed {} objects in S3 with prefix: {}", objects.len(), prefix);
        Ok(objects)
    }

    pub async fn delete_object(&self, path: &str) -> Result<()> {
        info!("Deleting object in S3: {}", path);
        let path = ObjectPath::from(path);
        self.store.delete(&path).await
            .map_err(|e| {
                error!("Error deleting object in S3: {}", e);
                ToolError::Storage(e)
            })?;
        info!("Successfully deleted object in S3: {}", path);
        Ok(())
    }

    async fn list_files_recursively(path: &std::path::Path) -> Result<Vec<(std::path::PathBuf, std::path::PathBuf)>> {
        let mut files = Vec::new();
        let mut dirs = vec![path.to_path_buf()];

        while let Some(dir) = dirs.pop() {
            let mut entries = tokio::fs::read_dir(&dir).await
                .map_err(|e| {
                    error!("Error reading directory {}: {}", dir.display(), e);
                    ToolError::Io(e)
                })?;

            while let Some(entry) = entries.next_entry().await
                .map_err(|e| {
                    error!("Error reading directory entry: {}", e);
                    ToolError::Io(e)
                })? {
                let file_type = entry.file_type().await
                    .map_err(|e| {
                        error!("Error getting file type: {}", e);
                        ToolError::Io(e)
                    })?;

                if file_type.is_dir() {
                    dirs.push(entry.path());
                } else {
                    let relative_path = entry.path().strip_prefix(path)
                        .map_err(|e| {
                            error!("Error computing relative path: {}", e);
                            ToolError::Config(e.to_string())
                        })?
                        .to_path_buf();
                    files.push((entry.path(), relative_path));
                }
            }
        }

        Ok(files)
    }

    pub async fn upload_directory(&self, local_dir: &std::path::Path, remote_prefix: &str) -> Result<()> {
        info!("Uploading directory {} to S3 prefix: {}", local_dir.display(), remote_prefix);
        
        let files = Self::list_files_recursively(local_dir).await?;
        let file_count = files.len();
        for (local_path, relative_path) in files {
            let remote_path = if remote_prefix.is_empty() {
                relative_path.to_string_lossy().to_string()
            } else {
                format!("{}/{}", remote_prefix.trim_matches('/'), relative_path.to_string_lossy())
            };

            self.upload_file(&local_path, &remote_path).await?;
        }

        info!("Successfully uploaded {} files from directory {}", file_count, local_dir.display());
        Ok(())
    }

    pub async fn download_directory(&self, remote_prefix: &str, local_dir: &std::path::Path) -> Result<()> {
        info!("Downloading S3 prefix {} to directory {}", remote_prefix, local_dir.display());

        let objects = self.list_objects(remote_prefix).await?;
        let object_count = objects.len();
        for obj in objects {
            let relative_path = obj.trim_start_matches(remote_prefix).trim_start_matches('/');
            let local_path = local_dir.join(relative_path);

            if let Some(parent) = local_path.parent() {
                tokio::fs::create_dir_all(parent).await
                    .map_err(|e| {
                        error!("Error creating directory {}: {}", parent.display(), e);
                        ToolError::Io(e)
                    })?;
            }

            self.download_file(&obj, &local_path).await?;
        }

        info!("Successfully downloaded {} files to directory {}", object_count, local_dir.display());
        Ok(())
    }

    async fn list_files_with_metadata(&self, prefix: &str) -> Result<HashMap<String, (u64, SystemTime)>> {
        info!("Listing files with metadata in S3 with prefix: {}", prefix);
        let mut files = HashMap::new();
        
        let prefix_path = ObjectPath::from(prefix);
        let list_stream = self.store.list(Some(&prefix_path));
        
        let mut stream = list_stream.await.map_err(|e| {
            error!("Error listing objects in S3: {}", e);
            ToolError::Storage(e)
        })?;

        while let Some(meta) = stream.try_next().await.map_err(|e| {
            error!("Error listing objects in S3: {}", e);
            ToolError::Storage(e)
        })? {
            let path = meta.location.to_string();
            let size = meta.size as u64;
            let last_modified = UNIX_EPOCH + std::time::Duration::from_secs(
                meta.last_modified.timestamp() as u64
            );
            files.insert(path, (size, last_modified));
        }

        info!("Successfully listed {} files with metadata", files.len());
        Ok(files)
    }

    pub async fn sync_directories(&self, source: &str, dest: &str, delete: bool) -> Result<()> {
        info!("Syncing from {} to {}", source, dest);
        
        // List files in source and destination
        let source_files = self.list_files_with_metadata(source).await?;
        let dest_files = self.list_files_with_metadata(dest).await?;

        // Find files to copy (missing or different size/timestamp)
        let mut files_to_copy = Vec::new();
        for (src_path, (src_size, src_time)) in &source_files {
            // Get the relative path by removing the source prefix
            let rel_path = src_path.strip_prefix(source)
                .unwrap_or(src_path)
                .trim_start_matches('/');

            // Construct the destination path
            let dest_path = if dest.ends_with('/') {
                format!("{}{}", dest, rel_path)
            } else {
                format!("{}/{}", dest, rel_path)
            };

            if let Some((dest_size, dest_time)) = dest_files.get(&dest_path) {
                // File exists in both places, check if different
                if src_size != dest_size || src_time != dest_time {
                    files_to_copy.push((src_path.clone(), dest_path));
                }
            } else {
                // File doesn't exist in destination
                files_to_copy.push((src_path.clone(), dest_path));
            }
        }

        // Copy files that are missing or different
        for (src_path, dest_path) in files_to_copy {
            info!("Copying {} to {}", src_path, dest_path);
            
            // Download from source
            let src_path_obj = ObjectPath::from(src_path.as_str());
            let get_result = self.store.get(&src_path_obj).await.map_err(|e| {
                error!("Error downloading from S3: {}", e);
                ToolError::Storage(e)
            })?;
            
            let mut data = Vec::new();
            match get_result {
                GetResult::File(file, _) => {
                    let mut file = std::io::BufReader::new(file);
                    std::io::copy(&mut file, &mut data).map_err(|e| {
                        error!("Error reading from file: {}", e);
                        ToolError::Io(e)
                    })?;
                },
                GetResult::Stream(mut stream) => {
                    while let Some(chunk) = stream.try_next().await.map_err(|e| {
                        error!("Error reading from S3: {}", e);
                        ToolError::Storage(e)
                    })? {
                        data.extend_from_slice(&chunk);
                    }
                }
            }
            
            // Upload to destination
            let dest_path_obj = ObjectPath::from(dest_path.as_str());
            self.store.put(&dest_path_obj, Bytes::from(data)).await.map_err(|e| {
                error!("Error uploading to S3: {}", e);
                ToolError::Storage(e)
            })?;
        }

        // Delete files that exist in destination but not in source
        if delete {
            for dest_path in dest_files.keys() {
                // Get the relative path by removing the destination prefix
                let rel_path = dest_path.strip_prefix(dest)
                    .unwrap_or(dest_path)
                    .trim_start_matches('/');

                // Construct the source path
                let src_path = if source.ends_with('/') {
                    format!("{}{}", source, rel_path)
                } else {
                    format!("{}/{}", source, rel_path)
                };

                if !source_files.contains_key(&src_path) {
                    info!("Deleting {}", dest_path);
                    self.delete_object(dest_path).await?;
                }
            }
        }

        info!("Successfully synced directories");
        Ok(())
    }

    pub async fn sync_local_to_remote(&self, local_dir: &std::path::Path, remote_prefix: &str, delete: bool) -> Result<()> {
        info!("Syncing from local {} to remote {}", local_dir.display(), remote_prefix);
        
        // List files in source (local) and destination (remote)
        let local_files = Self::list_files_recursively(local_dir).await?;
        let remote_files = self.list_files_with_metadata(remote_prefix).await?;

        // Convert local files to a map of relative path -> (size, mtime) for comparison
        let mut local_files_map = HashMap::new();
        for (local_path, rel_path) in local_files {
            let metadata = tokio::fs::metadata(&local_path).await
                .map_err(|e| {
                    error!("Error getting file metadata: {}", e);
                    ToolError::Io(e)
                })?;
            
            let mtime = metadata.modified()
                .map_err(|e| {
                    error!("Error getting file mtime: {}", e);
                    ToolError::Io(e)
                })?;

            local_files_map.insert(
                rel_path.to_string_lossy().to_string(),
                (metadata.len(), mtime)
            );
        }

        // Find files to copy (missing or different size/timestamp)
        for (rel_path, (local_size, local_time)) in &local_files_map {
            let remote_path = if remote_prefix.is_empty() {
                rel_path.clone()
            } else {
                format!("{}/{}", remote_prefix.trim_matches('/'), rel_path)
            };

            if let Some((remote_size, remote_time)) = remote_files.get(&remote_path) {
                // File exists in both places, check if different
                if local_size != remote_size || local_time != remote_time {
                    let local_path = local_dir.join(rel_path);
                    info!("Updating {} in remote storage", remote_path);
                    self.upload_file(&local_path, &remote_path).await?;
                }
            } else {
                // File doesn't exist in destination
                let local_path = local_dir.join(rel_path);
                info!("Copying {} to remote storage", remote_path);
                self.upload_file(&local_path, &remote_path).await?;
            }
        }

        // Delete remote files that don't exist locally
        if delete {
            for remote_path in remote_files.keys() {
                let rel_path = remote_path.strip_prefix(remote_prefix)
                    .unwrap_or(remote_path)
                    .trim_start_matches('/');

                if !local_files_map.contains_key(rel_path) {
                    info!("Deleting {} from remote storage", remote_path);
                    self.delete_object(remote_path).await?;
                }
            }
        }

        info!("Successfully synced from local to remote");
        Ok(())
    }

    pub async fn sync_remote_to_local(&self, remote_prefix: &str, local_dir: &std::path::Path, delete: bool) -> Result<()> {
        info!("Syncing from remote {} to local {}", remote_prefix, local_dir.display());
        
        // List files in source (remote) and destination (local)
        let remote_files = self.list_files_with_metadata(remote_prefix).await?;
        
        // Create local directory if it doesn't exist
        tokio::fs::create_dir_all(local_dir).await
            .map_err(|e| {
                error!("Error creating directory: {}", e);
                ToolError::Io(e)
            })?;
        
        let local_files = if local_dir.exists() {
            Self::list_files_recursively(local_dir).await?
        } else {
            Vec::new()
        };

        // Convert local files to a map of relative path -> (size, mtime) for comparison
        let mut local_files_map = HashMap::new();
        for (local_path, rel_path) in local_files {
            let metadata = tokio::fs::metadata(&local_path).await
                .map_err(|e| {
                    error!("Error getting file metadata: {}", e);
                    ToolError::Io(e)
                })?;
            
            let mtime = metadata.modified()
                .map_err(|e| {
                    error!("Error getting file mtime: {}", e);
                    ToolError::Io(e)
                })?;

            local_files_map.insert(
                rel_path.to_string_lossy().to_string(),
                (local_path, metadata.len(), mtime)
            );
        }

        // Find files to copy (missing or different size/timestamp)
        for (remote_path, (remote_size, remote_time)) in &remote_files {
            let rel_path = remote_path.strip_prefix(remote_prefix)
                .unwrap_or(remote_path)
                .trim_start_matches('/');

            let local_path = local_dir.join(rel_path);

            if let Some((_, local_size, local_time)) = local_files_map.get(rel_path) {
                // File exists in both places, check if different
                if remote_size != local_size || remote_time != local_time {
                    info!("Updating {} in local storage", local_path.display());
                    self.download_file(remote_path, &local_path).await?;
                }
            } else {
                // File doesn't exist locally
                info!("Copying {} to local storage", local_path.display());
                self.download_file(remote_path, &local_path).await?;
            }
        }

        // Delete local files that don't exist in remote
        if delete {
            for (rel_path, (local_path, _, _)) in local_files_map {
                let remote_path = if remote_prefix.is_empty() {
                    rel_path
                } else {
                    format!("{}/{}", remote_prefix.trim_matches('/'), rel_path)
                };

                if !remote_files.contains_key(&remote_path) {
                    info!("Deleting {}", local_path.display());
                    tokio::fs::remove_file(&local_path).await
                        .map_err(|e| {
                            error!("Error deleting file: {}", e);
                            ToolError::Io(e)
                        })?;
                }
            }
        }

        info!("Successfully synced from remote to local");
        Ok(())
    }
}
