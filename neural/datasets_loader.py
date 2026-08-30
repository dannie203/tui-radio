import os
import torch
from torch.utils.data import Dataset
import torchaudio
import torchaudio.transforms as T
import numpy as np
from typing import Optional, Tuple, Dict

# Complete Musical Key to Camelot Mapping
KEY_TO_CAMELOT: Dict[str, str] = {
    # Minor Keys (A)
    "g# minor": "1A", "ab minor": "1A", "g# min": "1A", "ab min": "1A",
    "d# minor": "2A", "eb minor": "2A", "d# min": "2A", "eb min": "2A",
    "a# minor": "3A", "bb minor": "3A", "a# min": "3A", "bb min": "3A",
    "f minor": "4A",  "f min": "4A",
    "c minor": "5A",  "c min": "5A",
    "g minor": "6A",  "g min": "6A",
    "d minor": "7A",  "d min": "7A",
    "a minor": "8A",  "a min": "8A",
    "e minor": "9A",  "e min": "9A",
    "b minor": "10A", "b min": "10A",
    "f# minor": "11A", "gb minor": "11A", "f# min": "11A", "gb min": "11A",
    "c# minor": "12A", "db minor": "12A", "c# min": "12A", "db min": "12A",

    # Major Keys (B)
    "b major": "1B",  "b maj": "1B",
    "f# major": "2B", "gb major": "2B", "f# maj": "2B", "gb maj": "2B",
    "c# major": "3B", "db major": "3B", "c# maj": "3B", "db maj": "3B",
    "g# major": "4B", "ab major": "4B", "g# maj": "4B", "ab maj": "4B",
    "d# major": "5B", "eb major": "5B", "d# maj": "5B", "eb maj": "5B",
    "a# major": "6B", "bb major": "6B", "a# maj": "6B", "bb maj": "6B",
    "f major": "7B",  "f maj": "7B",
    "c major": "8B",  "c maj": "8B",
    "g major": "9B",  "g maj": "9B",
    "d major": "10B", "d maj": "10B",
    "a major": "11B", "a maj": "11B",
    "e major": "12B", "e maj": "12B",
}

CAMELOT_TO_IDX = {
    "1A": 0, "1B": 1, "2A": 2, "2B": 3, "3A": 4, "3B": 5, "4A": 6, "4B": 7,
    "5A": 8, "5B": 9, "6A": 10, "6B": 11, "7A": 12, "7B": 13, "8A": 14, "8B": 15,
    "9A": 16, "9B": 17, "10A": 18, "10B": 19, "11A": 20, "11B": 21, "12A": 22, "12B": 23,
}

class RealBeatDataset(Dataset):
    """
    Dataset loader for real-world beat tracking datasets (e.g. Ballroom / Beatles / GTZAN).
    Loads audio, resamples to 22050Hz, computes Log-Mel Spectrogram, and creates continuous Gaussian-blurred targets for beat & downbeat frames.
    """
    def __init__(
        self,
        audio_files: list[str],
        annotation_files: list[str],
        sample_rate: int = 22050,
        hop_length: int = 512,
        n_mels: int = 128,
        chunk_seconds: float = 10.0,
    ):
        self.samples = []
        self.sample_rate = sample_rate
        self.hop_length = hop_length
        self.n_mels = n_mels
        self.chunk_frames = int(chunk_seconds * sample_rate / hop_length)

        self.mel_transform = T.MelSpectrogram(
            sample_rate=sample_rate,
            n_fft=2048,
            win_length=2048,
            hop_length=hop_length,
            n_mels=n_mels,
        )
        self.amp_to_db = T.AmplitudeToDB(top_db=80.0)

        for a_path, ann_path in zip(audio_files, annotation_files):
            if os.path.exists(a_path) and os.path.exists(ann_path):
                self.samples.append((a_path, ann_path))

    def __len__(self):
        return len(self.samples)

    def __getitem__(self, idx):
        audio_path, ann_path = self.samples[idx]
        waveform, sr = torchaudio.load(audio_path)
        if sr != self.sample_rate:
            waveform = T.Resample(sr, self.sample_rate)(waveform)
        if waveform.shape[0] > 1:
            waveform = waveform.mean(dim=0, keepdim=True)

        # Mel Spectrogram [1, n_mels, time_frames] -> [1, time_frames, n_mels]
        spec = self.amp_to_db(self.mel_transform(waveform))
        spec = (spec - spec.mean()) / (spec.std() + 1e-6)
        spec = spec.permute(0, 2, 1) # [1, time, n_mels]

        total_frames = spec.shape[1]
        labels = torch.zeros(total_frames, 2)

        # Parse annotations: timestamp beat_num
        with open(ann_path, "r") as f:
            for line in f:
                parts = line.strip().split()
                if len(parts) >= 2:
                    try:
                        timestamp = float(parts[0])
                        beat_num = int(parts[1])
                        frame_idx = int(timestamp * self.sample_rate / self.hop_length)

                        if 0 <= frame_idx < total_frames:
                            # Apply small Gaussian pulse around the beat frame (+- 1 frame)
                            labels[frame_idx, 0] = 1.0 # Beat
                            if frame_idx > 0: labels[frame_idx - 1, 0] = max(labels[frame_idx - 1, 0], 0.5)
                            if frame_idx + 1 < total_frames: labels[frame_idx + 1, 0] = max(labels[frame_idx + 1, 0], 0.5)

                            if beat_num == 1:
                                labels[frame_idx, 1] = 1.0 # Downbeat
                                if frame_idx > 0: labels[frame_idx - 1, 1] = max(labels[frame_idx - 1, 1], 0.5)
                                if frame_idx + 1 < total_frames: labels[frame_idx + 1, 1] = max(labels[frame_idx + 1, 1], 0.5)
                    except ValueError:
                        continue

        # Slice fixed chunk for batching
        if total_frames > self.chunk_frames:
            spec = spec[:, :self.chunk_frames, :]
            labels = labels[:self.chunk_frames, :]
        elif total_frames < self.chunk_frames:
            pad_len = self.chunk_frames - total_frames
            spec = torch.nn.functional.pad(spec, (0, 0, 0, pad_len))
            labels = torch.nn.functional.pad(labels, (0, 0, 0, pad_len))

        return spec, labels
