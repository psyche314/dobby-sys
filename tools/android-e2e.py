"""Run the Rust JNI E2E through Dobby's pinned Android test launcher."""
import argparse
from pathlib import Path
import shutil
import subprocess
import sys

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--library", required=True, type=Path)
parser.add_argument("--out", required=True, type=Path)
args, launcher_args = parser.parse_known_args()
root = Path(__file__).resolve().parents[1]
staging = args.out.resolve() / "staging"
destination = staging / "tests/e2e/libdobby_e2e_jni.so"
destination.parent.mkdir(parents=True, exist_ok=True)
shutil.copy2(args.library, destination)
subprocess.run([sys.executable, str(root / "dobby/tests/e2e/android/run.py"),
                "--build-dir", str(staging), "--out", str(args.out), *launcher_args], check=True)
