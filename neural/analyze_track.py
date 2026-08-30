import sys
import os
import torch
import numpy as np
import torchaudio

# Keep CPU footprint tiny during background inference
torch.set_num_threads(2)

from audio_features import AudioFeatureExtractor
from models.key_classifier import CAMELOT_KEYS
from ort_utils import create_inference_session

def estimate_bpm_from_beats(beat_probs: np.ndarray, fps: float = 22050.0 / 512.0) -> float:
    """
    Estimate BPM from beat probability sequence using autocorrelation.
    """
    if len(beat_probs) < 50:
        return 120.0
    
    # Autocorrelation of beat activation
    corr = np.correlate(beat_probs - beat_probs.mean(), beat_probs - beat_probs.mean(), mode="full")
    corr = corr[len(corr)//2:]
    
    # Restrict lag to 60 BPM - 200 BPM
    min_lag = int(fps * 60.0 / 200.0) # ~13 frames
    max_lag = int(fps * 60.0 / 60.0)  # ~43 frames
    
    if max_lag >= len(corr):
        return 120.0
        
    lag_peak = min_lag + np.argmax(corr[min_lag:max_lag])
    if lag_peak > 0:
        bpm = (fps * 60.0) / lag_peak
        # Standardize range between 70 and 160
        while bpm < 75.0: bpm *= 2.0
        while bpm > 175.0: bpm /= 2.0
        return round(float(bpm), 1)
    return 120.0

def analyze_track(audio_path: str):
    if not os.path.exists(audio_path):
        print(f"❌ File not found: {audio_path}")
        return

    print(f"\n🎧 [ANALYZING]: {os.path.basename(audio_path)}")
    print(f"📁 Path: {audio_path}")

    # 1. Feature Extraction on GPU
    device = "cuda" if torch.cuda.is_available() else "cpu"
    extractor = AudioFeatureExtractor().to(device)
    
    waveform, sr = torchaudio.load(audio_path)
    duration_sec = waveform.shape[-1] / sr
    print(f"⏱️ Duration: {duration_sec:.2f}s | Sample Rate: {sr}Hz | Channels: {waveform.shape[0]}")

    # Convert to log-mel spectrogram on GPU
    spec_tensor = extractor.process_file(audio_path, device=device) # [1, 1, time_frames, 128]
    spec_np = spec_tensor.cpu().numpy().astype(np.float32)

    # 2. Run ONNX Models
    beat_session = create_inference_session("weights/cyberdj_beat.onnx")
    cue_session = create_inference_session("weights/cyberdj_cue.onnx")
    key_session = create_inference_session("weights/cyberdj_key.onnx")

    # Beat analysis
    beat_out = beat_session.run(None, {"spectrogram": spec_np})[0] # [1, time, 2]
    beat_probs = beat_out[0, :, 0]
    downbeat_probs = beat_out[0, :, 1]
    estimated_bpm = estimate_bpm_from_beats(beat_probs)

    # Key analysis
    key_logits = key_session.run(None, {"spectrogram": spec_np})[0] # [1, 24]
    pred_key_idx = int(np.argmax(key_logits, axis=1)[0])
    detected_key = CAMELOT_KEYS[pred_key_idx]

    # Cue / Boundary analysis
    boundary, section, energy = cue_session.run(None, {"spectrogram": spec_np})
    energy_curve = energy[0]

    # Map each cue-model output frame to a time in seconds.
    # Cue output time axis is downsampled by 4x from the 512-sample mel hop:
    #   frame_dur = 4 * (512 / 22050)
    frame_dur = 4.0 * (512.0 / 22050.0)
    n = boundary.shape[1] if boundary.ndim > 1 else boundary.shape[0]
    boundary_t = np.arange(n) * frame_dur

    # Section class per frame: 0=Intro, 1=Verse/Body, 2=Drop/Chorus, 3=Outro
    section = np.argmax(section[0], axis=0) if section.ndim == 3 else np.argmax(section, axis=0)
    boundary_p = boundary[0] if boundary.ndim > 1 else boundary

    # Detect structural boundary peaks (mean + 1.5*std, local maxima).
    b_mean = float(boundary_p.mean())
    b_std = float(boundary_p.std())
    threshold = b_mean + 1.5 * b_std
    peaks = []
    for i in range(1, n - 1):
        if boundary_p[i] > threshold and boundary_p[i] >= boundary_p[i - 1] and boundary_p[i] >= boundary_p[i + 1]:
            peaks.append(boundary_t[i])

    # ---- Mix-In cue: end of intro / first steady section ----
    # Prefer the first frame that moves out of the Intro section within a
    # sane window (4s .. 40% of track). Fall back to an early boundary peak.
    early_end = min(duration_sec * 0.4, duration_sec)
    mix_in_time = None
    for i in range(n):
        if section[i] != 0 and 4.0 <= boundary_t[i] <= early_end:
            mix_in_time = float(boundary_t[i])
            break
    if mix_in_time is None:
        for p in peaks:
            if 4.0 <= p <= early_end:
                mix_in_time = float(p)
                break
    if mix_in_time is None:
        mix_in_time = min(15.0, duration_sec * 0.1)

    # ---- Mix-Out cue: start of outro / last structural boundary ----
    # Search strictly in the final outro window (final 85% or last 25s of the track)
    # so we NEVER cut songs in half.
    late_start = max(duration_sec * 0.85, duration_sec - 25.0)
    mix_out_time = None
    for i in range(n):
        if section[i] == 3 and boundary_t[i] >= late_start and boundary_t[i] < duration_sec - 3.0:
            mix_out_time = float(boundary_t[i])
            break
    if mix_out_time is None:
        for p in reversed(peaks):
            if p >= late_start and p < duration_sec - 3.0:
                mix_out_time = float(p)
                break
    if mix_out_time is None:
        mix_out_time = max(0.0, duration_sec - 8.0)
    # Clamp to a safe outro window (never earlier than 85% of duration)
    mix_out_time = mix_out_time.clamp(duration_sec * 0.85, max(0.0, duration_sec * 0.98)) if duration_sec > 10.0 else max(0.0, duration_sec - 2.0)

    # Save to Boombox Neural Profile Cache
    import json
    config_dir = os.path.expanduser("~/.config/boombox")
    os.makedirs(config_dir, exist_ok=True)
    cache_path = os.path.join(config_dir, "neural_profiles.json")

    profiles = {}
    if os.path.exists(cache_path):
        try:
            with open(cache_path, "r") as f:
                profiles = json.load(f)
        except Exception:
            profiles = {}

    profiles[os.path.abspath(audio_path)] = {
        "bpm": float(estimated_bpm),
        "camelot_key": detected_key,
        "energy": float(np.mean(energy_curve)),
        "mix_in_sec": float(mix_in_time),
        "mix_out_sec": float(mix_out_time),
        "duration_sec": float(duration_sec),
    }

    with open(cache_path, "w") as f:
        json.dump(profiles, f, indent=2)

    gpu_name = torch.cuda.get_device_name(0) if torch.cuda.is_available() else "CPU"
    active_prov = beat_session.get_providers()[0]
    print("\n" + "="*55)
    print(f"🤖 BOOMBOX CYBER-DJ NEURAL ANALYSIS REPORT")
    print(f"⚡ GPU Accelerator    : {gpu_name} ({active_prov})")
    print("="*55)
    print(f"  🎵 Estimated BPM     : {estimated_bpm} BPM")
    print(f"  🎹 Harmonic Key      : {detected_key} (Camelot Wheel)")
    print(f"  ⚡ Average Energy    : {float(np.mean(energy_curve)):.2f} / 1.00")
    print(f"  🎛️ Suggested Mix-In  : {mix_in_time:.1f}s")
    print(f"  🎛️ Suggested Mix-Out : {mix_out_time:.1f}s (Fade Duration: ~8s)")
    print(f"  💾 Cached to         : {cache_path}")
    print("="*55 + "\n")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        analyze_track(sys.argv[1])
    else:
        # Default test on user's local music
        test_file = "/home/aki/Music/2025 - Justin Biebier - Swag (2025) [Flac 24-44] AtM/01 - ALL I CAN TAKE.flac"
        analyze_track(test_file)
