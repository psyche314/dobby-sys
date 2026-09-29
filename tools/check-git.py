"""Verify that an external Cargo Git dependency initializes and builds the pinned native source."""
from pathlib import Path
import subprocess
import sys
import tempfile

revision = sys.argv[1]
if len(revision) != 40 or any(c not in "0123456789abcdef" for c in revision):
    raise SystemExit("Expected a full Git commit hash")
with tempfile.TemporaryDirectory(prefix="dobby-consumer-") as temp:
    root = Path(temp)
    (root / "src").mkdir()
    (root / "Cargo.toml").write_text(f'''[package]
name = "dobby-git-consumer"
version = "0.1.0"
edition = "2024"
[dependencies]
dobby-sys = {{ git = "https://github.com/psyche314/dobby-sys", rev = "{revision}" }}
''')
    (root / "src/main.rs").write_text('''fn main() {
    let version = unsafe { std::ffi::CStr::from_ptr(dobby_sys::DobbyGetVersion()) };
    assert_eq!(version.to_bytes(), b"1.0.0");
    println!("PASS external Cargo Git consumer");
}
''')
    subprocess.run(["cargo", "run", "--manifest-path", str(root / "Cargo.toml")], check=True)
