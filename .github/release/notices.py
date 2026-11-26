"""Collect upstream distribution notices from the locked Windows build inputs."""
import json
from pathlib import Path
import subprocess

app = Path.cwd()
output = app / "artifact" / "THIRD-PARTY-NOTICES.txt"
packages = json.loads(subprocess.check_output([
    "cargo", "metadata", "--locked", "--format-version", "1",
    "--features", "gui,custom-protocol", "--filter-platform", "x86_64-pc-windows-msvc",
], encoding="utf-8"))["packages"]
inputs = [(p["name"], p["version"], p.get("license"), Path(p["manifest_path"]).parent)
          for p in packages if p["source"]]
for name in ["react", "react-dom", "scheduler", "@tauri-apps/api"]:
    path = app / "node_modules" / name
    meta = json.loads((path / "package.json").read_text(encoding="utf-8"))
    inputs.append((name, meta["version"], meta.get("license"), path))
with output.open("w", encoding="utf-8") as stream:
    for name, version, license_id, path in sorted(inputs, key=lambda p: p[:2]):
        print(f"\n=== {name} {version} ({license_id or 'see upstream'}) ===", file=stream)
        for child in sorted(path.iterdir()):
            if child.name.lower().startswith(("license", "licence", "copying", "notice", "copyright")):
                files = [child] if child.is_file() else sorted(child.rglob("*"))
                for file in files:
                    if file.is_file():
                        print(file.read_text(encoding="utf-8", errors="replace"), file=stream)
