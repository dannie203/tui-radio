import importlib.util
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT))


def test_selects_cuda_when_available():
    import onnxruntime as ort
    import torch

    from ort_utils import get_onnx_providers

    original = ort.get_available_providers
    original_cuda = torch.cuda.is_available

    try:
        ort.get_available_providers = lambda: ["CUDAExecutionProvider", "CPUExecutionProvider"]
        torch.cuda.is_available = lambda: True
        assert get_onnx_providers() == ["CUDAExecutionProvider", "CPUExecutionProvider"]
    finally:
        ort.get_available_providers = original
        torch.cuda.is_available = original_cuda


def test_falls_back_to_cpu_when_cuda_missing():
    import onnxruntime as ort
    import torch

    from ort_utils import get_onnx_providers

    original = ort.get_available_providers
    original_cuda = torch.cuda.is_available

    try:
        ort.get_available_providers = lambda: ["CPUExecutionProvider"]
        torch.cuda.is_available = lambda: False
        assert get_onnx_providers() == ["CPUExecutionProvider"]
    finally:
        ort.get_available_providers = original
        torch.cuda.is_available = original_cuda
