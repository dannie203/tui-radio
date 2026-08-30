import os
import torch
import torch.nn as nn

class BNEDjTransitionPolicy(nn.Module):
    """
    Boombox Neural Engine - AI DJ Transition Policy Network.
    Takes high-level neural features of Track A (outgoing) and Track B (incoming):
    Inputs: [batch, 8]
      - 0: BPM_A normalized (bpm / 200.0)
      - 1: BPM_B normalized (bpm / 200.0)
      - 2: BPM_diff normalized (|bpm_a - bpm_b| / 50.0)
      - 3: Key_Match_Score (1.0 for perfect, 0.7 for adjacent, 0.0 for clashing)
      - 4: Energy_A (0.0 -> 1.0)
      - 5: Energy_B (0.0 -> 1.0)
      - 6: Section_A (0: Intro, 1: Verse, 2: Chorus, 3: Outro)
      - 7: Section_B (0: Intro, 1: Verse, 2: Chorus, 3: Outro)

    Outputs:
      - strategy_logits: [batch, 4] -> (0: BassSwap, 1: FilterSweep, 2: EchoOutDrop, 3: DownbeatCut)
      - transition_bars: [batch, 1] -> (Predicted optimal overlap length: 4, 8, 16, 32 bars)
      - tempo_sync_enabled: [batch, 1] -> Probability of applying tempo-stretching
    """
    def __init__(self):
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(8, 64),
            nn.ELU(),
            nn.Linear(64, 64),
            nn.ELU(),
            nn.Dropout(0.1),
        )

        self.strategy_head = nn.Linear(64, 4) # 4 transition styles
        self.bars_head = nn.Sequential(
            nn.Linear(64, 1),
            nn.Sigmoid() # 0.0 -> 1.0 (mapped to 4 -> 32 bars)
        )
        self.tempo_sync_head = nn.Sequential(
            nn.Linear(64, 1),
            nn.Sigmoid()
        )

    def forward(self, x: torch.Tensor):
        feat = self.net(x)
        strategy = self.strategy_head(feat)
        bars = self.bars_head(feat) * 28.0 + 4.0 # 4 to 32 bars
        tempo_sync = self.tempo_sync_head(feat)
        return strategy, bars, tempo_sync

def export_transition_policy(output_path="weights/cyberdj_transition.onnx"):
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    model = BNEDjTransitionPolicy()
    model.eval()

    dummy_input = torch.randn(1, 8)
    torch.onnx.export(
        model,
        dummy_input,
        output_path,
        export_params=True,
        opset_version=17,
        input_names=["track_pair_features"],
        output_names=["strategy_logits", "transition_bars", "tempo_sync_prob"],
        dynamic_axes={
            "track_pair_features": {0: "batch_size"},
            "strategy_logits": {0: "batch_size"},
            "transition_bars": {0: "batch_size"},
            "tempo_sync_prob": {0: "batch_size"},
        },
    )
    print(f"✅ Exported BNEDjTransitionPolicy to {output_path}")

if __name__ == "__main__":
    export_transition_policy()
