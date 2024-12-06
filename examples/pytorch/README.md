# PyTorch Training Example with Mounted Storage

This example demonstrates how to use our storage tool with PyTorch for managing training data and checkpoints efficiently.

## Setup

1. Install the required dependencies:
```bash
pip install -r requirements.txt
```

2. Configure your storage mount point in `config.json`:
```json
{
    "mount_point": "/path/to/mount",
    "storage_type": "s3"
}
```

## Directory Structure

The mounted directory will contain:
```
mounted_dir/
├── data/
│   ├── train/
│   └── val/
└── checkpoints/
```

## Running the Example

1. First, mount your storage:
```bash
mytool mount
```

2. Place your training data in the mounted directory:
```bash
cp -r /path/to/your/dataset/* /path/to/mount/data/
```

3. Run the training script:
```bash
python train.py --data-dir /path/to/mount/data --checkpoint-dir /path/to/mount/checkpoints
```

The script will automatically:
- Load training data from the mounted directory
- Save checkpoints periodically
- Resume training from the latest checkpoint if available

## Key Features

- Automatic checkpoint saving and loading
- Efficient data loading from mounted storage
- Fault tolerance with automatic resume
- Progress tracking with TensorBoard

## Notes

- Checkpoints are automatically saved every epoch
- Training can be safely interrupted and resumed
- Data is streamed efficiently from the mounted storage
