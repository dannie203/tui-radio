import torch
import torch.nn as nn

class BNECueDetector(nn.Module):
    """
    Boombox Neural Engine - Track Structure & DJ Cue Point Detector.
    Detects:
    1. Structural Boundaries (Intro, Verse, Chorus/Drop, Outro transitions)
    2. Ideal Mix-In Points (end of intro / first steady beat)
    3. Ideal Mix-Out Points (start of outro / breakdown)
    4. Energy Curve (0.0 -> 1.0) for dynamic EQ adjustment
    """
    def __init__(self, n_mels: int = 128, hidden_size: int = 96):
        super().__init__()
        
        # Temporal convolution with dilated receptive field to capture long-range structural changes (bars / phrases)
        self.conv = nn.Sequential(
            nn.Conv2d(1, 32, kernel_size=(5, 5), padding=(2, 2)),
            nn.BatchNorm2d(32),
            nn.ReLU(),
            nn.MaxPool2d(kernel_size=(2, 2)), # [batch, 32, time/2, 64]

            nn.Conv2d(32, 64, kernel_size=(5, 5), padding=(2, 2)),
            nn.BatchNorm2d(64),
            nn.ReLU(),
            nn.MaxPool2d(kernel_size=(2, 4)), # [batch, 64, time/4, 16]
        )

        self.temporal_layers = nn.Sequential(
            nn.Conv1d(64 * 16, hidden_size, kernel_size=7, padding=3, dilation=1),
            nn.BatchNorm1d(hidden_size),
            nn.ReLU(),
            nn.Conv1d(hidden_size, hidden_size, kernel_size=7, padding=6, dilation=2),
            nn.BatchNorm1d(hidden_size),
            nn.ReLU(),
            nn.Conv1d(hidden_size, hidden_size, kernel_size=7, padding=12, dilation=4),
            nn.BatchNorm1d(hidden_size),
            nn.ReLU(),
        )

        # Output heads:
        # 1. Boundary probability [batch, time_frames_downsampled, 1]
        # 2. Section Class: 0: Intro, 1: Verse/Body, 2: Drop/Chorus, 3: Outro
        # 3. Energy level: [batch, time_frames_downsampled, 1]
        self.boundary_head = nn.Sequential(
            nn.Conv1d(hidden_size, 32, kernel_size=1),
            nn.ReLU(),
            nn.Conv1d(32, 1, kernel_size=1),
            nn.Sigmoid()
        )

        self.section_head = nn.Sequential(
            nn.Conv1d(hidden_size, 32, kernel_size=1),
            nn.ReLU(),
            nn.Conv1d(32, 4, kernel_size=1), # 4 classes
        )

        self.energy_head = nn.Sequential(
            nn.Conv1d(hidden_size, 16, kernel_size=1),
            nn.ReLU(),
            nn.Conv1d(16, 1, kernel_size=1),
            nn.Sigmoid()
        )

    def forward(self, x: torch.Tensor):
        """
        Input: [batch, 1, time_frames, n_mels]
        Returns:
            - boundary_prob: [batch, time_reduced]
            - section_logits: [batch, 4, time_reduced]
            - energy: [batch, time_reduced]
        """
        batch, _, time, _ = x.shape
        c_out = self.conv(x) # [batch, 64, time/4, 16]
        c_time = c_out.shape[2]
        
        # Reshape to [batch, 64*16, time/4]
        c_flat = c_out.permute(0, 1, 3, 2).contiguous().view(batch, 64 * 16, c_time)
        
        feat = self.temporal_layers(c_flat) # [batch, hidden, time/4]
        
        boundary = self.boundary_head(feat).squeeze(1)
        section = self.section_head(feat)
        energy = self.energy_head(feat).squeeze(1)

        return {
            "boundary": boundary,
            "section": section,
            "energy": energy
        }
