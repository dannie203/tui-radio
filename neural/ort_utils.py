import onnxruntime as ort
import torch


def get_onnx_providers():
    """Return the preferred provider list for this machine.

    Prefer CUDA when the runtime exposes it and the GPU is visible to PyTorch.
    Otherwise fall back to CPU.
    """
    available = ort.get_available_providers()

    if "CUDAExecutionProvider" in available and torch.cuda.is_available():
        return ["CUDAExecutionProvider", "CPUExecutionProvider"]

    if "ROCMExecutionProvider" in available and torch.cuda.is_available():
        return ["ROCMExecutionProvider", "CPUExecutionProvider"]

    if "DirectMLExecutionProvider" in available:
        return ["DirectMLExecutionProvider", "CPUExecutionProvider"]

    return ["CPUExecutionProvider"]


def create_inference_session(model_path: str, *, session_options=None):
    """Construct an inference session while preferring GPU when supported."""
    session_options = session_options or ort.SessionOptions()
    providers = get_onnx_providers()

    try:
        return ort.InferenceSession(model_path, sess_options=session_options, providers=providers)
    except Exception:
        return ort.InferenceSession(model_path, sess_options=session_options, providers=["CPUExecutionProvider"])
