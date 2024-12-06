import os
import argparse
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import DataLoader
from torchvision import datasets, transforms
from torch.utils.tensorboard import SummaryWriter
from tqdm import tqdm

def parse_args():
    parser = argparse.ArgumentParser(description='PyTorch Training with Mounted Storage')
    parser.add_argument('--data-dir', type=str, required=True,
                        help='Path to mounted data directory')
    parser.add_argument('--checkpoint-dir', type=str, required=True,
                        help='Path to mounted checkpoint directory')
    parser.add_argument('--batch-size', type=int, default=32,
                        help='Training batch size')
    parser.add_argument('--epochs', type=int, default=10,
                        help='Number of epochs to train')
    parser.add_argument('--lr', type=float, default=0.001,
                        help='Learning rate')
    return parser.parse_args()

class SimpleNet(nn.Module):
    def __init__(self):
        super(SimpleNet, self).__init__()
        self.conv1 = nn.Conv2d(3, 6, 5)
        self.pool = nn.MaxPool2d(2, 2)
        self.conv2 = nn.Conv2d(6, 16, 5)
        self.fc1 = nn.Linear(16 * 5 * 5, 120)
        self.fc2 = nn.Linear(120, 84)
        self.fc3 = nn.Linear(84, 10)

    def forward(self, x):
        x = self.pool(torch.relu(self.conv1(x)))
        x = self.pool(torch.relu(self.conv2(x)))
        x = x.view(-1, 16 * 5 * 5)
        x = torch.relu(self.fc1(x))
        x = torch.relu(self.fc2(x))
        x = self.fc3(x)
        return x

def save_checkpoint(model, optimizer, epoch, loss, path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    torch.save({
        'epoch': epoch,
        'model_state_dict': model.state_dict(),
        'optimizer_state_dict': optimizer.state_dict(),
        'loss': loss,
    }, path)

def load_checkpoint(model, optimizer, path):
    if not os.path.exists(path):
        return 0, float('inf')
    
    checkpoint = torch.load(path)
    model.load_state_dict(checkpoint['model_state_dict'])
    optimizer.load_state_dict(checkpoint['optimizer_state_dict'])
    return checkpoint['epoch'], checkpoint['loss']

def main():
    args = parse_args()
    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')

    # Create directories if they don't exist
    os.makedirs(args.data_dir, exist_ok=True)
    os.makedirs(args.checkpoint_dir, exist_ok=True)

    # Data loading and preprocessing
    transform = transforms.Compose([
        transforms.RandomHorizontalFlip(),
        transforms.RandomCrop(32, padding=4),
        transforms.ToTensor(),
        transforms.Normalize((0.5, 0.5, 0.5), (0.5, 0.5, 0.5))
    ])

    # Load datasets from mounted directory
    train_dataset = datasets.CIFAR10(
        root=args.data_dir, train=True, download=True, transform=transform
    )
    train_loader = DataLoader(
        train_dataset, batch_size=args.batch_size, shuffle=True, num_workers=2
    )

    # Initialize model, optimizer, and loss function
    model = SimpleNet().to(device)
    optimizer = optim.Adam(model.parameters(), lr=args.lr)
    criterion = nn.CrossEntropyLoss()

    # Setup TensorBoard
    writer = SummaryWriter(os.path.join(args.checkpoint_dir, 'runs'))

    # Load checkpoint if exists
    checkpoint_path = os.path.join(args.checkpoint_dir, 'latest_checkpoint.pth')
    start_epoch, best_loss = load_checkpoint(model, optimizer, checkpoint_path)

    # Training loop
    for epoch in range(start_epoch, args.epochs):
        model.train()
        running_loss = 0.0
        progress_bar = tqdm(train_loader, desc=f'Epoch {epoch+1}/{args.epochs}')

        for i, (inputs, labels) in enumerate(progress_bar):
            inputs, labels = inputs.to(device), labels.to(device)

            optimizer.zero_grad()
            outputs = model(inputs)
            loss = criterion(outputs, labels)
            loss.backward()
            optimizer.step()

            running_loss += loss.item()
            progress_bar.set_postfix({'loss': running_loss / (i + 1)})

        epoch_loss = running_loss / len(train_loader)
        writer.add_scalar('Loss/train', epoch_loss, epoch)

        # Save checkpoint
        save_checkpoint(model, optimizer, epoch + 1, epoch_loss, checkpoint_path)

        if epoch_loss < best_loss:
            best_loss = epoch_loss
            best_checkpoint_path = os.path.join(args.checkpoint_dir, 'best_checkpoint.pth')
            save_checkpoint(model, optimizer, epoch + 1, epoch_loss, best_checkpoint_path)

    writer.close()
    print('Training completed!')

if __name__ == '__main__':
    main()
