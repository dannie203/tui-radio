import numpy as np
from ort_utils import create_inference_session

def test_inference():
    print("🧪 Testing ONNX Runtime Inference...")
    
    # 1. Test Beat Tracker ONNX
    beat_session = create_inference_session("weights/cyberdj_beat.onnx")
    active_providers = beat_session.get_providers()
    print(f"🚀 Execution Provider : {active_providers[0]} (CUDA GPU Accelerated)")
    dummy_spec = np.random.randn(1, 1, 200, 128).astype(np.float32)
    beat_out = beat_session.run(None, {"spectrogram": dummy_spec})[0]
    print(f"✅ BNEBeatTracker Inference OK! Output shape: {beat_out.shape} (Time: {beat_out.shape[1]} frames, Channels: {beat_out.shape[2]})")

    # 2. Test Cue Detector ONNX
    cue_session = create_inference_session("weights/cyberdj_cue.onnx")
    boundary, section, energy = cue_session.run(None, {"spectrogram": dummy_spec})
    print(f"✅ BNECueDetector Inference OK! Boundary: {boundary.shape}, Section: {section.shape}, Energy: {energy.shape}")

    # 3. Test Key Classifier ONNX
    key_session = create_inference_session("weights/cyberdj_key.onnx")
    key_logits = key_session.run(None, {"spectrogram": dummy_spec})[0]
    predicted_key_idx = np.argmax(key_logits, axis=1)[0]
    from models.key_classifier import CAMELOT_KEYS
    print(f"✅ BNEKeyClassifier Inference OK! Logits: {key_logits.shape} -> Predicted Key: {CAMELOT_KEYS[predicted_key_idx]}")

if __name__ == "__main__":
    test_inference()
