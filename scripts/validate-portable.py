"""Extract and run this release with isolated preferences and no development PATH.

Keeps every generated file under target; never deletes or uses the real profile.
"""
from pathlib import Path
import datetime
import hashlib
import json
import os
import re
import subprocess
import zipfile

project = Path(__file__).resolve().parents[1]
root = project.parents[1]
version = re.search(r'(?m)^version = "([^"]+)"', (project / "Cargo.toml").read_text()).group(1)
name = f"CPU-Express-{version}"
archives = [root / "release" / f"{name}{suffix}.zip" for suffix in ("", "-Fonte", "-Evidencias")]
for archive in archives:
    with zipfile.ZipFile(archive) as zipped:
        assert zipped.testzip() is None, f"CRC failure: {archive.name}"
run = project / "target" / ("portable acceptance " + datetime.datetime.now().strftime("%Y%m%d-%H%M%S-%f"))
run.mkdir(parents=True, exist_ok=False)
with zipfile.ZipFile(archives[0]) as zipped:
    for item in zipped.infolist():
        assert (run / item.filename).resolve().is_relative_to(run.resolve()), "Unsafe archive path"
    zipped.extractall(run)
exe = run / name / "cpu-express.exe"
digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
assert digest(exe) == digest(project / "target/release/cpu-express.exe")
config = run / "acceptance.json"
config.write_text(json.dumps({"taskbar_mode": False, "floating_rect": [100, 100, 640, 180], "fps": 60}), encoding="utf-8")
diagnostics = run / "acceptance.jsonl"
env = os.environ.copy()
for key in list(env):
    if key.startswith("CPU_EXPRESS_") or key in {"CARGO_HOME", "RUSTUP_HOME"}:
        env.pop(key)
system_root = os.environ["SystemRoot"]  # Windows environment lookup is case-insensitive.
env["PATH"] = os.pathsep.join([str(Path(system_root) / "System32"), system_root])
startup = subprocess.STARTUPINFO()
startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
startup.wShowWindow = subprocess.SW_HIDE
result = subprocess.run([str(exe), "--demo", "--config", str(config), "--smoke-seconds", "8", "--diagnostics", str(diagnostics)], cwd=exe.parent, env=env, startupinfo=startup, timeout=30, check=True)
records = [json.loads(line) for line in diagnostics.read_text(encoding="utf-8-sig").splitlines() if line.strip()]
assert records[-1]["event"] == "shutdown", "Missing graceful shutdown"
evidence = {"scope": "Portable extracted into path with spaces; isolated profile and Windows-only PATH", "version": version, "binary_sha256": digest(exe), "zip_crc_verified": True, "exit_code": result.returncode}
(project / "evidence/current/portable-result.json").write_text(json.dumps(evidence, indent=2), encoding="utf-8")
print(json.dumps(evidence, indent=2))
