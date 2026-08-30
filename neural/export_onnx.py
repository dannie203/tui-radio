import os
import torch
import onnx
from models.beat_tracker import BNEBeatTracker
from models.cue_detector import BNECueDetector
from models.key_classifier import BNEKeyClassifier

def export_beat_tracker(weights_path=None, output_path="weights/cyberdj_beat.onnx"):
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    model = BNEBeatTracker()
    if weights_path and os.path.exists(weights_path):
        model.load_state_dict(torch.load(weights_path, map_location="cpu"))
    model.eval()

    # Dynamic input: [batch, 1, time_frames, n_mels]
    dummy_input = torch.randn(1, 1, 300, 128)

    torch.onnx.export(
        model,
        dummy_input,
        output_path,
        export_params=True,
        opset_version=17,
        do_constant_folding=True,
        input_names=["spectrogram"],
        output_names=["beat_probs"],
        dynamic_axes={
            "spectrogram": {0: "batch_size", 2: "time_frames"},
            "beat_probs": {0: "batch_size", 1: "time_frames"},
        },
    )
    print(f"✅ Exported BNEBeatTracker to {output_path}")

def export_cue_detector(weights_path=None, output_path="weights/cyberdj_cue.onnx"):
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    model = BNECueDetector()
    if weights_path and os.path.exists(weights_path):
        model.load_state_dict(torch.load(weights_path, map_location="cpu"))
    model.eval()

    dummy_input = torch.randn(1, 1, 400, 128)

    torch.onnx.export(
        model,
        dummy_input,
        output_path,
        export_params=True,
        opset_version=17,
        do_constant_folding=True,
        input_names=["spectrogram"],
        output_names=["boundary", "section", "energy"],
        dynamic_axes={
            "spectrogram": {0: "batch_size", 2: "time_frames"},
            "boundary": {0: "batch_size", 1: "time_reduced"},
            "section": {0: "batch_size", 2: "time_reduced"},
            "energy": {0: "batch_size", 1: "time_reduced"},
        },
    )
    print(f"✅ Exported BNECueDetector to {output_path}")

def export_key_classifier(weights_path=None, output_path="weights/cyberdj_key.onnx"):
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    model = BNEKeyClassifier()
    if weights_path and os.path.exists(weights_path):
        model.load_state_dict(torch.load(weights_path, map_location="cpu"))
    model.eval()

    dummy_input = torch.randn(1, 1, 300, 128)

    torch.onnx.export(
        model,
        dummy_input,
        output_path,
        export_params=True,
        opset_version=17,
        do_constant_folding=True,
        input_names=["spectrogram"],
        output_names=["key_logits"],
        dynamic_axes={
            "spectrogram": {0: "batch_size", 2: "time_frames"},
            "key_logits": {0: "batch_size"},
        },
    )
    print(f"✅ Exported BNEKeyClassifier to {output_path}")

if __name__ == "__main__":
    print("🚀 Exporting Boombox Neural Engine models to ONNX...")
    export_beat_tracker()
    export_cue_detector()
    export_key_classifier()
