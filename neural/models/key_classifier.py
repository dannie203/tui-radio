import torch
import torch.nn as nn

# 24 Camelot keys mapping
CAMELOT_KEYS = [
    "1A", "1B", "2A", "2B", "3A", "3B", "4A", "4B",
    "5A", "5B", "6A", "6B", "7A", "7B", "8A", "8B",
    "9A", "9B", "10A", "10B", "11A", "11B", "12A", "12B"
]

class BNEKeyClassifier(nn.Module):
    """
    Boombox Neural Engine - 24-Key Camelot Harmonic Key Classifier.
    - Inputs: CQT / Chromagram or Log-Mel Spectrogram of the entire track (or steady chorus).
    - Outputs: [batch, 24] Logits corresponding to Camelot Keys.
    """
    def __init__(self, in_features: int = 128, num_classes: int = 24):
        super().__init__()
        
        self.features = nn.Sequential(
            nn.Conv2d(1, 32, kernel_size=3, padding=1),
            nn.BatchNorm2d(32),
            nn.ReLU(),
            nn.MaxPool2d(2, 2),

            nn.Conv2d(32, 64, kernel_size=3, padding=1),
            nn.BatchNorm2d(64),
            nn.ReLU(),
            nn.MaxPool2d(2, 2),

            nn.Conv2d(64, 128, kernel_size=3, padding=1),
            nn.BatchNorm2d(128),
            nn.ReLU(),
            nn.AdaptiveAvgPool2d((1, 1)),
        )

        self.classifier = nn.Sequential(
            nn.Flatten(),
            nn.Linear(128, 64),
            nn.ReLU(),
            nn.Dropout(0.3),
            nn.Linear(64, num_classes)
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Input: [batch, 1, time_frames, n_mels]
        Output: [batch, 24] (logits)
        """
        feat = self.features(x)
        return self.classifier(feat)

    @staticmethod
    def get_camelot_key(class_idx: int) -> str:
        return CAMELOT_KEYS[class_idx]
