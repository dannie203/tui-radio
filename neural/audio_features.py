import torch
import torch.nn as nn
import torchaudio.transforms as T
from typing import Tuple, Optional

SAMPLE_RATE = 22050
N_FFT = 2048
HOP_LENGTH = 512
N_MELS = 128
F_MIN = 30.0
F_MAX = 11000.0

class AudioFeatureExtractor(nn.Module):
    """
    High-performance GPU-accelerated Audio Feature Extractor for Boombox Neural Engine.
    Converts raw waveforms into Log-Mel Spectrograms and Harmonic Chromagrams.
    """
    def __init__(
        self,
        sample_rate: int = SAMPLE_RATE,
        n_fft: int = N_FFT,
        hop_length: int = HOP_LENGTH,
        n_mels: int = N_MELS,
        f_min: float = F_MIN,
        f_max: float = F_MAX,
    ):
        super().__init__()
        self.sample_rate = sample_rate
        self.hop_length = hop_length
        self.n_fft = n_fft

        # 1. Mel-Spectrogram transform for Beat & Cue detection
        self.mel_transform = T.MelSpectrogram(
            sample_rate=sample_rate,
            n_fft=n_fft,
            win_length=n_fft,
            hop_length=hop_length,
            f_min=f_min,
            f_max=f_max,
            n_mels=n_mels,
            power=2.0,
        )

        # 2. Amplitude to dB (Log scale)
        self.amplitude_to_db = T.AmplitudeToDB(top_db=80.0)

    def forward(self, waveform: torch.Tensor) -> torch.Tensor:
        """
        Input: waveform of shape [batch, channels, samples] or [batch, samples]
        Output: log-mel spectrogram [batch, 1, time_frames, n_mels]
        """
        if waveform.ndim == 2:
            # [batch, samples] -> [batch, 1, samples]
            waveform = waveform.unsqueeze(1)

        # If stereo, convert to mono by averaging channels
        if waveform.shape[1] > 1:
            waveform = waveform.mean(dim=1, keepdim=True)

        # Compute Mel Spectrogram: [batch, 1, n_mels, time_frames]
        mel_spec = self.mel_transform(waveform)
        log_mel = self.amplitude_to_db(mel_spec)

        # Normalize per instance (zero mean, unit variance)
        mean = log_mel.mean(dim=(-2, -1), keepdim=True)
        std = log_mel.std(dim=(-2, -1), keepdim=True) + 1e-6
        norm_mel = (log_mel - mean) / std

        # Transpose to [batch, 1, time_frames, n_mels] for 2D Conv & RNN processing
        return norm_mel.permute(0, 1, 3, 2)

    @torch.no_grad()
    def process_file(self, audio_path: str, device: str = "cuda") -> torch.Tensor:
        """
        Load an audio file, resample to 22.05kHz, and extract features on GPU.
        """
        import torchaudio
        waveform, sr = torchaudio.load(audio_path)
        if sr != self.sample_rate:
            resampler = T.Resample(orig_freq=sr, new_freq=self.sample_rate)
            waveform = resampler(waveform)

        waveform = waveform.unsqueeze(0).to(device) # [1, channels, samples]
        return self.to(device)(waveform)
