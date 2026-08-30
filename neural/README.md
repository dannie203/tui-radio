# 🧠 Boombox Cyber-DJ Neural Engine (BNE)

Deep learning models for real-time and offline audio analysis, beat tracking, cue detection, and smart DJ automixing in `boombox-rs`.

## 📦 Neural Models

1. **`BNEBeatTracker` (`models/beat_tracker.py`)**
   - **Architecture:** CRNN (3-layer 2D-CNN + 2-layer Bidirectional GRU + Projection Head).
   - **Target:** Predicts beat and downbeat (phách số 1) probabilities from 128-bin Log-Mel Spectrograms.
   - **Use case:** Seamless beatmatching and downbeat synchronization between Deck A and Deck B.

2. **`BNECueDetector` (`models/cue_detector.py`)**
   - **Architecture:** Dilated Temporal Convolutional Network with multi-scale receptive field.
   - **Target:** Detects structural boundaries (Intro, Verse, Chorus/Drop, Outro), optimal mix-in / mix-out timestamps, and instantaneous energy curve.
   - **Use case:** Triggering crossfades at the exact musical phrase boundaries instead of arbitrary seconds.

3. **`BNEKeyClassifier` (`models/key_classifier.py`)**
   - **Architecture:** MobileNetV3-style 2D CNN classifier.
   - **Target:** Classifies tracks into 24 Camelot Harmonic Keys (`1A` - `12B`).
   - **Use case:** Harmonic mixing recommendation to prevent musical clashing.

## 🚀 Quickstart

### 1. Training
```bash
python train.py
```

### 2. Exporting to ONNX
```bash
python export_onnx.py
```
Exported ONNX models are saved to `weights/*.onnx` for high-speed inference in Rust via the `ort` crate.
