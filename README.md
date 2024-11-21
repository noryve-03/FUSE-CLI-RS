# ML Artifact Management Tool

A robust command-line tool for managing machine learning training artifacts across cloud storage platforms. Currently supports AWS S3 with plans for other cloud providers.

## Features

### Cloud Storage Operations

- **List Files**: View files and directories in cloud storage
  ```bash
  mytool list s3://bucket/prefix
  mytool list --long s3://bucket/prefix  # Detailed view
  ```

- **Copy Files**: Copy files between local and cloud storage
  ```bash
  # Upload local file/directory to cloud
  mytool copy local_file s3://bucket/path
  mytool copy --recursive local_dir s3://bucket/path

  # Download from cloud to local
  mytool copy s3://bucket/path local_file
  mytool copy --recursive s3://bucket/path local_dir
  ```

- **Sync Directories**: Bidirectional synchronization between local and cloud storage
  ```bash
  # Sync local to cloud
  mytool sync local_dir s3://bucket/prefix

  # Sync cloud to local
  mytool sync s3://bucket/prefix local_dir

  # Sync between cloud directories
  mytool sync s3://bucket/prefix1 s3://bucket/prefix2

  # Sync with deletion of files not in source
  mytool sync --delete source_dir destination_dir
  ```

- **Mount Cloud Storage**: Mount cloud storage as a local filesystem (experimental)
  ```bash
  mytool mount --source s3://bucket/prefix --mountpoint /path/to/mount
  mytool mount --readonly --source s3://bucket/prefix --mountpoint /path/to/mount
  ```

### Key Features

- **Efficient File Transfer**:
  - Parallel upload/download operations
  - Streaming support for large files
  - Automatic retry on failure

- **Smart Synchronization**:
  - File comparison using size and modification time
  - Selective file transfer (only modified files)
  - Optional deletion of files not in source
  - Preserves directory structure

- **Flexible Configuration**:
  - JSON-based configuration file
  - Support for AWS credentials
  - Configurable transfer settings

## Installation

1. Ensure you have Rust installed (1.70.0 or later)
2. Clone the repository
3. Build the project:
   ```bash
   cargo build --release
   ```

## Configuration

Create a `config.json` file in your config directory:
```json
{
  "default_storage": {
    "provider": "s3",
    "region": "us-east-1",
    "bucket": "your-bucket-name"
  },
  "mount_options": {
    "cache_size_mb": 1024,
    "timeout_seconds": 300,
    "read_only": true
  },
  "transfer_options": {
    "concurrent_uploads": 4,
    "chunk_size": 8388608,
    "retry_attempts": 3
  }
}
```

### Configuration Options

- **default_storage**:
  - `provider`: Storage provider (currently only "s3")
  - `region`: AWS region
  - `bucket`: Default S3 bucket
  - `endpoint`: Optional custom endpoint

- **mount_options**:
  - `cache_size_mb`: Local cache size for mounted filesystem
  - `timeout_seconds`: Operation timeout
  - `read_only`: Mount in read-only mode

- **transfer_options**:
  - `concurrent_uploads`: Number of parallel transfers
  - `chunk_size`: Size of upload chunks in bytes
  - `retry_attempts`: Number of retry attempts on failure

## Usage Examples

### Basic Operations

```bash
# List contents of a directory in S3
mytool list s3://bucket/path

# Upload a file to S3
mytool copy data.txt s3://bucket/data.txt

# Download a file from S3
mytool copy s3://bucket/data.txt local_data.txt

# Sync ML model artifacts to S3
mytool sync --delete ./models s3://bucket/models

# Mount S3 bucket for read-only access
mytool mount --source s3://bucket/data --mountpoint ./mounted_data --readonly
```

### Advanced Usage

```bash
# Recursive directory upload with progress
mytool copy --recursive ./experiment_results s3://bucket/results

# Sync between two S3 directories
mytool sync s3://bucket/model_v1 s3://bucket/model_v2

# List files with detailed information
mytool list --long s3://bucket/path
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
