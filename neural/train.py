import os
import time
import torch
import torch.nn as nn
from torch.utils.data import Dataset, DataLoader
from models.beat_tracker import BNEBeatTracker
from models.cue_detector import BNECueDetector
from models.key_classifier import BNEKeyClassifier

class SyntheticRhythmDataset(Dataset):
    """
    Synthetic Audio Feature Dataset for rapid verification of the training pipeline & loss convergence.
    Generates synthetic log-mel spectrograms with periodic pulses imitating 120-130 BPM beats and downbeats.
    """
    def __init__(self, num_samples: int = 500, time_frames: int = 300, n_mels: int = 128):
        self.num_samples = num_samples
        self.time_frames = time_frames
        self.n_mels = n_mels

    def __len__(self):
        return self.num_samples

    def __getitem__(self, idx):
        # Generate background noise
        spec = torch.randn(1, self.time_frames, self.n_mels) * 0.2
        labels = torch.zeros(self.time_frames, 2) # [0: beat, 1: downbeat]

        # Simulate a 125 BPM beat pulse (every ~20 frames at hop_length=512, sr=22050)
        period = 20
        for t in range(10, self.time_frames - 5, period):
            # Kick/Bass transient in lower mel bins
            spec[0, t:t+2, :30] += 2.5
            # Hi-hat transient in high mel bins
            spec[0, t:t+1, 80:] += 1.8
            labels[t, 0] = 1.0 # Beat

            # Every 4th beat is a downbeat (Bar start)
            if (t // period) % 4 == 0:
                labels[t, 1] = 1.0 # Downbeat
                spec[0, t:t+3, :40] += 1.5 # Stronger kick on downbeat

        return spec, labels

def train_beat_tracker(epochs: int = 5, batch_size: int = 16, lr: float = 1e-3):
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    print(f"⚡ Training on device: {device} ({torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'CPU'})")

    dataset = SyntheticRhythmDataset(num_samples=400)
    val_dataset = SyntheticRhythmDataset(num_samples=80)
    train_loader = DataLoader(dataset, batch_size=batch_size, shuffle=True, pin_memory=True)
    val_loader = DataLoader(val_dataset, batch_size=batch_size, shuffle=False)

    model = BNEBeatTracker().to(device)
    # Weighted BCE Loss with Logits because beats are sparse (mostly zeros)
    pos_weight = torch.tensor([5.0, 10.0]).to(device)
    criterion = nn.BCEWithLogitsLoss(pos_weight=pos_weight)
    
    optimizer = torch.optim.AdamW(model.parameters(), lr=lr, weight_decay=1e-4)
    scaler = torch.amp.GradScaler('cuda') if device.type == "cuda" else None

    os.makedirs("weights", exist_ok=True)

    print("🚀 Starting training BNEBeatTracker...")
    for epoch in range(1, epochs + 1):
        model.train()
        total_loss = 0.0
        start_t = time.time()

        for specs, targets in train_loader:
            specs, targets = specs.to(device), targets.to(device)
            optimizer.zero_grad()

            if scaler:
                with torch.amp.autocast('cuda'):
                    preds = model(specs, return_logits=True)
                    loss = criterion(preds, targets)
                scaler.scale(loss).backward()
                scaler.step(optimizer)
                scaler.update()
            else:
                preds = model(specs, return_logits=True)
                loss = criterion(preds, targets)
                loss.backward()
                optimizer.step()

            total_loss += loss.item()

        elapsed = time.time() - start_t
        avg_loss = total_loss / len(train_loader)
        print(f"Epoch [{epoch}/{epochs}] - Loss: {avg_loss:.4f} - Time: {elapsed:.2f}s")

    torch.save(model.state_dict(), "weights/bne_beat_tracker.pth")
    print("💾 Saved PyTorch checkpoint: weights/bne_beat_tracker.pth")

if __name__ == "__main__":
    train_beat_tracker(epochs=5, batch_size=16)
