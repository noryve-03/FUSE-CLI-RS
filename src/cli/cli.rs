/// CLI Module - Command Line Interface Implementation
///
/// This module is responsible for handling all command-line interface aspects of the tool.
/// It uses the clap crate for argument parsing and provides a type-safe way to handle
/// user commands and options.
///
/// Key Components:
/// - Cli struct: Main entry point for CLI parsing
/// - Commands enum: Available commands (list, copy, sync, mount)
/// - Command-specific structs: Arguments for each command
///
/// The module follows a hierarchical structure:
/// 1. Global options (config file, verbosity)
/// 2. Subcommands (list, copy, sync, mount)
/// 3. Command-specific options
///
/// Usage:
/// The main application instantiates this module through Cli::parse_args()
/// and uses pattern matching on Commands to execute the appropriate action.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "mytool")]
#[command(about = "ML Training Artifact Management Tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Copy files between local and cloud storage
    Copy {
        /// Source path (local path or s3:// URL)
        source: String,
        /// Destination path (local path or s3:// URL)
        destination: String,
        /// Recursively copy directories
        #[arg(short, long)]
        recursive: bool,
    },

    /// Mount cloud storage as local filesystem
    Mount {
        /// Cloud storage URI
        #[arg(short, long)]
        source: String,

        /// Local mount point
        #[arg(short, long)]
        mountpoint: PathBuf,

        /// Read-only mount
        #[arg(short, long)]
        readonly: bool,
    },

    /// Sync directories between local and cloud
    Sync {
        /// Source directory
        source: String,

        /// Destination directory
        destination: String,

        /// Delete files in destination that don't exist in source
        #[arg(short = 'D', long)]
        delete: bool,
    },

    /// List files in a directory
    List {
        /// Path to list (local path or s3:// URL)
        path: String,
        /// Use long listing format
        #[arg(short, long)]
        long: bool,
    },
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}