/// ML Artifact Management Tool - Main Entry Point
///
/// This is the main entry point for the ML artifact management tool. It coordinates
/// all the different components of the application and handles the primary control flow.
///
/// Main Responsibilities:
/// 1. Application Setup
///    - Initialize logging system
///    - Parse command line arguments
///    - Load configuration
///
/// 2. Command Routing
///    - Match CLI commands to appropriate handlers
///    - Initialize storage backends
///    - Execute operations
///
/// 3. Error Handling
///    - Catch and format errors
///    - Provide user-friendly error messages
///    - Ensure proper cleanup on failure
///
/// The application uses tokio for async runtime and tracing for logging.

mod cli;
mod config;
mod error;
mod storage;
mod fuse;

use cli::{Cli, Commands};
use config::Config;
use error::{Result, ToolError};
use storage::s3::S3Storage;
use fuse::CloudFS;

use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse_args();

    // Load configuration
    let config = Config::load(cli.config)?;

    // Initialize storage backend
    let storage = S3Storage::new(&config.default_storage).await?;

    match cli.command {
        Commands::Copy { source, destination, recursive } => {
            info!("Copying {} to {}", source, destination);
            
            let is_source_cloud = source.starts_with("s3://");
            let is_dest_cloud = destination.starts_with("s3://");

            match (is_source_cloud, is_dest_cloud) {
                // Local to cloud
                (false, true) => {
                    let local_path = std::path::Path::new(&source);
                    let remote_path = destination.trim_start_matches("s3://")
                        .trim_start_matches(&config.default_storage.bucket.unwrap_or_default())
                        .trim_start_matches('/');
                    
                    if recursive && local_path.is_dir() {
                        storage.upload_directory(local_path, remote_path).await?;
                    } else {
                        storage.upload_file(local_path, remote_path).await?;
                    }
                }
                // Cloud to local
                (true, false) => {
                    let remote_path = source.trim_start_matches("s3://")
                        .trim_start_matches(&config.default_storage.bucket.unwrap_or_default())
                        .trim_start_matches('/');
                    let local_path = std::path::Path::new(&destination);
                    
                    if recursive {
                        storage.download_directory(remote_path, local_path).await?;
                    } else {
                        storage.download_file(remote_path, local_path).await?;
                    }
                }
                // Cloud to cloud
                (true, true) => {
                    error!("Cloud to cloud copy not yet implemented");
                    return Err(ToolError::NotImplemented("Cloud to cloud copy".into()));
                }
                // Local to local
                (false, false) => {
                    error!("Local to local copy should use system commands");
                    return Err(ToolError::InvalidOperation("Use system commands for local copy".into()));
                }
            }
        }

        Commands::Mount { source, mountpoint, readonly: _ } => {
            info!("Mounting {} at {}", source, mountpoint.display());
            let fs = CloudFS::new(storage);
            fs.mount(mountpoint)?;
        }

        Commands::Sync { source, destination, delete } => {
            info!("Syncing {} to {}", source, destination);
            
            let is_source_cloud = source.starts_with("s3://");
            let is_dest_cloud = destination.starts_with("s3://");

            match (is_source_cloud, is_dest_cloud) {
                // Cloud to cloud sync
                (true, true) => {
                    let bucket = config.default_storage.bucket.clone().unwrap_or_default();
                    let source_path = source.trim_start_matches("s3://")
                        .trim_start_matches(&bucket)
                        .trim_start_matches('/');
                    let dest_path = destination.trim_start_matches("s3://")
                        .trim_start_matches(&bucket)
                        .trim_start_matches('/');
                    
                    storage.sync_directories(source_path, dest_path, delete).await?;
                }
                // Local to cloud sync
                (false, true) => {
                    let local_dir = std::path::Path::new(&source);
                    let remote_prefix = destination.trim_start_matches("s3://")
                        .trim_start_matches(&config.default_storage.bucket.unwrap_or_default())
                        .trim_start_matches('/');
                    
                    storage.sync_local_to_remote(local_dir, remote_prefix, delete).await?;
                }
                // Cloud to local sync
                (true, false) => {
                    let remote_prefix = source.trim_start_matches("s3://")
                        .trim_start_matches(&config.default_storage.bucket.unwrap_or_default())
                        .trim_start_matches('/');
                    let local_dir = std::path::Path::new(&destination);
                    
                    storage.sync_remote_to_local(remote_prefix, local_dir, delete).await?;
                }
                // Local to local sync
                (false, false) => {
                    error!("Local to local sync should use system commands");
                    return Err(ToolError::InvalidOperation("Use system commands for local sync".into()));
                }
            }
        }

        Commands::List { path, long } => {
            info!("Listing contents of {}", path);
            // Strip s3:// prefix and bucket name if present
            let prefix = if path.starts_with("s3://") {
                let without_scheme = path.trim_start_matches("s3://");
                if let Some(rest) = without_scheme.find('/') {
                    &without_scheme[rest + 1..]
                } else {
                    ""
                }
            } else {
                &path
            };
            
            let objects = storage.list_objects(prefix).await?;
            for obj in objects {
                if long {
                    // TODO: Add more details in long format
                    println!("{}", obj);
                } else {
                    println!("{}", obj);
                }
            }
        }
    }

    Ok(())
}