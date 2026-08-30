import torch
import torch.nn as nn

class BNEBeatTracker(nn.Module):
    """
    Boombox Neural Engine - Beat & Downbeat Phase Estimator.
    Architecture: CRNN (Convolutional Recurrent Neural Network) with Temporal Pooling
    - Inputs: Log-Mel Spectrogram [batch, 1, time_frames, n_mels]
    - Outputs: [batch, time_frames, 2] -> (Channel 0: Beat probability, Channel 1: Downbeat/Phách 1 probability)
    """
    def __init__(self, n_mels: int = 128, hidden_size: int = 128, num_layers: int = 2):
        super().__init__()
        
        # 1. 2D Convolutional Feature Extractor
        self.conv_stack = nn.Sequential(
            # Block 1: Capture fine-grained spectral transients (kicks, hi-hats, percussions)
            nn.Conv2d(1, 32, kernel_size=(3, 3), padding=1),
            nn.BatchNorm2d(32),
            nn.ELU(),
            nn.MaxPool2d(kernel_size=(1, 2)), # [batch, 32, time, 64]

            # Block 2: Broader rhythmic harmonic features
            nn.Conv2d(32, 64, kernel_size=(3, 3), padding=1),
            nn.BatchNorm2d(64),
            nn.ELU(),
            nn.MaxPool2d(kernel_size=(1, 2)), # [batch, 64, time, 32]

            # Block 3: High-level rhythmic patterns
            nn.Conv2d(64, 128, kernel_size=(3, 3), padding=1),
            nn.BatchNorm2d(128),
            nn.ELU(),
            nn.MaxPool2d(kernel_size=(1, 2)), # [batch, 128, time, 16]
            nn.Dropout2d(0.15),
        )

        conv_out_features = 128 * 16 # channels * remaining mel bins

        # 2. Bidirectional GRU to learn rhythmic meter and meter boundaries
        self.gru = nn.GRU(
            input_size=conv_out_features,
            hidden_size=hidden_size,
            num_layers=num_layers,
            bidirectional=True,
            batch_first=True,
            dropout=0.2 if num_layers > 1 else 0.0,
        )

        # 3. Output Projection Layer (Logits)
        self.head = nn.Sequential(
            nn.Linear(hidden_size * 2, 64),
            nn.ELU(),
            nn.Dropout(0.1),
            nn.Linear(64, 2), # 0: beat, 1: downbeat
        )

    def forward(self, x: torch.Tensor, return_logits: bool = False) -> torch.Tensor:
        """
        Input: [batch, 1, time_frames, n_mels]
        Output: [batch, time_frames, 2]
        """
        batch, _, time, _ = x.shape

        # CNN Feature extraction
        c_out = self.conv_stack(x) # [batch, 128, time, 16]

        # Reshape to [batch, time, features] for GRU
        c_out = c_out.permute(0, 2, 1, 3).contiguous().view(batch, time, -1)

        # Recurrent temporal processing
        gru_out, _ = self.gru(c_out) # [batch, time, hidden * 2]

        # Output prediction
        logits = self.head(gru_out) # [batch, time, 2]
        if return_logits:
            return logits
        return torch.sigmoid(logits)
