# ML Artifact Management Tool

A robust command-line tool for managing machine learning training artifacts across cloud storage platforms. Currently supports AWS S3 with plans for other cloud providers.

## Configuration

The tool requires a configuration file that specifies your storage settings. You can provide it in two ways:

1. Use the `-c` flag to specify the config file location:
   ```bash
   mytool -c /path/to/config.json list s3://bucket/prefix
   ```

2. Place it in the default location: `~/.config/mytool/config.json`

Example config.json:
```json
{
    "default_storage": {
        "provider": "s3",
        "region": "us-east-1",
        "endpoint": null,
        "bucket": "your-bucket-name"
    },
    "mount_options": {
        "cache_size_mb": 1024,
        "timeout_seconds": 300,
        "read_only": true
    },
    "transfer_options": {
        "concurrent_uploads": 4,
        "chunk_size": 8,
        "retry_attempts": 3
    }
}
```

## Features

### Cloud Storage Operations

- **List Files**: View files and directories in cloud storage
  ```bash
  mytool -c config.json list s3://bucket/prefix
  mytool -c config.json list --long s3://bucket/prefix  # Detailed view
  ```

- **Copy Files**: Copy files between local and cloud storage
  ```bash
  # Upload local file/directory to cloud
  mytool -c config.json copy local_file s3://bucket/path
  mytool -c config.json copy --recursive local_dir s3://bucket/path

  # Download from cloud to local
  mytool -c config.json copy s3://bucket/path local_file
  mytool -c config.json copy --recursive s3://bucket/path local_dir
  ```

- **Sync Directories**: Bidirectional synchronization between local and cloud storage
  ```bash
  # Sync local to cloud
  mytool -c config.json sync local_dir s3://bucket/prefix

  # Sync cloud to local
  mytool -c config.json sync s3://bucket/prefix local_dir

  # Sync between cloud directories
  mytool -c config.json sync s3://bucket/prefix1 s3://bucket/prefix2

  # Sync with deletion of files not in source
  mytool -c config.json sync --delete source_dir destination_dir
  ```

- **Mount Cloud Storage**: Mount cloud storage as a local filesystem (experimental)
  ```bash
  mytool -c config.json mount --source s3://bucket/prefix --mountpoint /path/to/mount
  mytool -c config.json mount --readonly --source s3://bucket/prefix --mountpoint /path/to/mount
  ```

### Key Features

- **Efficient File Transfer**:
  - Parallel upload/download operations
  - Streaming support for large files
  - Configurable chunk size and concurrency

- **Robust Error Handling**:
  - Automatic retries for transient failures
  - Detailed error messages
  - Configurable retry attempts

- **Security**:
  - AWS credentials from environment or credentials file
  - Support for different AWS regions
  - Optional endpoint configuration for S3-compatible services

## Installation

1. Clone the repository
2. Build the tool:
   ```bash
   cargo build --release
   ```
3. Create your config file either in:
   - `~/.config/mytool/config.json` (default location)
   - Or any location to be specified with `-c`

## Debugging

If you encounter issues, you can enable debug logging:
```bash
RUST_LOG=debug mytool -c config.json [command]
```

## Usage Examples

### Basic Operations

```bash
# List contents of a directory in S3
mytool -c config.json list s3://bucket/path

# Upload a file to S3
mytool -c config.json copy data.txt s3://bucket/data.txt

# Download a file from S3
mytool -c config.json copy s3://bucket/data.txt local_data.txt

# Sync ML model artifacts to S3
mytool -c config.json sync --delete ./models s3://bucket/models

# Mount S3 bucket for read-only access
mytool -c config.json mount --source s3://bucket/data --mountpoint ./mounted_data --readonly
```

### Advanced Usage

```bash
# Recursive directory upload with progress
mytool -c config.json copy --recursive ./experiment_results s3://bucket/results

# Sync between two S3 directories
mytool -c config.json sync s3://bucket/model_v1 s3://bucket/model_v2

# List files with detailed information
mytool -c config.json list --long s3://bucket/path
```

## Security

- Currently uses static AWS credentials
- Future versions will support:
  - AWS IAM roles
  - Credential rotation
  - Encryption at rest
  - Access control policies

## Limitations

- Local-to-local sync not supported (use system commands instead)
- Currently only supports AWS S3
- Static credential configuration

## Future Plans

1. Support for additional cloud providers:
   - Google Cloud Storage
   - Azure Blob Storage
   - MinIO

2. Enhanced security features:
   - Secure credential management
   - Client-side encryption
   - Access control integration

3. Performance improvements:
   - Smart chunking for large files
   - Improved caching
   - Bandwidth throttling

4. Additional features:
   - Progress bars
   - Bandwidth usage statistics
   - Detailed logging options
   - File checksums and verification

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
