import os
import glob
import json
import time
import torch
from analyze_track import analyze_track

# Limit CPU threads so background analysis stays ultra-lightweight and silent
torch.set_num_threads(2)
try:
    torch.set_num_interop_threads(1)
except Exception:
    pass

def scan_and_analyze_all(music_dir: str = "~/Music"):
    resolved_dir = os.path.expanduser(music_dir)
    print(f"🔍 Scanning music files in {resolved_dir}...")

    cache_path = os.path.expanduser("~/.config/boombox/neural_profiles.json")
    cached_profiles = {}
    if os.path.exists(cache_path):
        try:
            with open(cache_path, "r") as f:
                cached_profiles = json.load(f)
        except Exception:
            cached_profiles = {}
    
    extensions = ["*.flac", "*.mp3", "*.opus", "*.m4a", "*.wav", "*.ogg"]
    audio_files = []
    for ext in extensions:
        audio_files.extend(glob.glob(os.path.join(resolved_dir, "**", ext), recursive=True))

    to_process = [f for f in audio_files if os.path.abspath(f) not in cached_profiles]
    print(f"📦 Found {len(audio_files)} audio tracks ({len(cached_profiles)} cached, {len(to_process)} remaining).")
    
    for idx, fpath in enumerate(to_process, 1):
        print(f"\n[{idx}/{len(to_process)}] 🚀 Processing {os.path.basename(fpath)}")
        try:
            analyze_track(fpath)
        except Exception as e:
            print(f"⚠️ Error analyzing {fpath}: {e}")
        time.sleep(0.05) # Yield CPU

if __name__ == "__main__":
    scan_and_analyze_all()

