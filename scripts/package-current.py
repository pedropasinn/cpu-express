"""Package only a freshly validated native build, with hashes and current evidence."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import shutil
import struct
import subprocess
import zipfile
import zlib

project = Path(__file__).resolve().parents[1]
root = project.parents[1]
evidence = project / "evidence/current"
checks = json.loads((evidence / "checks.json").read_text(encoding="utf-8-sig"))
assert {c["name"] for c in checks} == {"format", "clippy", "tests"}
assert all(c["exit_code"] == 0 for c in checks), "Validation failed"
smoke = json.loads((evidence / "native-smoke-result.json").read_text(encoding="utf-8-sig"))
assert smoke["exit_code"] == 0, "Native smoke failed"
binary = project / "target/release/cpu-express.exe"
assert binary.is_file()
version = re.search(r'(?m)^version = "([^"]+)"', (project / "Cargo.toml").read_text()).group(1)
release = root / "release" / f"CPU-Express-{version}"
release.mkdir(parents=True, exist_ok=True)

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

assert smoke["binary_sha256"] == digest(binary), "Smoke test does not match final executable"
assert smoke["taskbar_surface_verified"] and smoke["independent_panel_verified"] and smoke["strip_capture"]
real = json.loads((evidence / "real-metrics-result.json").read_text(encoding="utf-8-sig"))
assert real["exit_code"] == 0 and real["valid_real_samples"] >= 2
assert real["binary_sha256"] == digest(binary)
transitions = json.loads((evidence / "transitions-expansion/result.json").read_text(encoding="utf-8"))
assert transitions["exit_code"] == 0 and len(transitions["stages"]) == 5
assert transitions["binary_sha256"] == digest(binary)
visual_refresh = json.loads((evidence / "visual-refresh/comparison.json").read_text(encoding="utf-8"))
assert visual_refresh["after"]["binary_sha256"] == digest(binary)
weather = json.loads((evidence / "weather/result.json").read_text(encoding="utf-8"))
assert weather["binary_sha256"] == digest(binary)
assert weather["exit_code"] == 0 and weather["online_verified"]
soak = json.loads((evidence / "soak/result.json").read_text(encoding="utf-8-sig"))
assert soak["binary_sha256"] == digest(binary) and len(soak["runs"]) == 2
assert all(r["exit_code"] == 0 for r in soak["runs"])
migration = json.loads((evidence / "profile-migration-result.json").read_text(encoding="utf-8-sig"))
assert migration["binary_sha256"] == digest(binary) and migration["exit_code"] == 0 and migration["migration_backup_verified"]
portable_path = evidence / "portable-result.json"
portable = json.loads(portable_path.read_text(encoding="utf-8-sig")) if portable_path.exists() else None
if portable is not None and portable["binary_sha256"] != digest(binary):
    portable = None  # A previous release test does not certify this executable.
if portable is not None:
    assert portable["binary_sha256"] == digest(binary) and portable["exit_code"] == 0 and portable["zip_crc_verified"]
ui_review = json.loads((evidence / "game-ui-result.json").read_text(encoding="utf-8"))
assert len(ui_review["panels"]) >= 13
assert all(digest(project / p) == h for p, h in ui_review["sources"].items())
assert all(digest(project / p) == h for p, h in ui_review["panels"].items())

def decode_log(path):
    data = path.read_bytes()
    return data.decode("utf-16" if data.startswith(b"\xff\xfe") else "utf-8-sig", errors="replace")

def ppm_to_png(path):
    data = path.read_bytes()
    magic, size, maximum, pixels = data.split(b"\n", 3)
    assert magic == b"P6" and maximum == b"255"
    w, h = map(int, size.split())
    assert len(pixels) == w * h * 3
    def chunk(kind, payload):
        return struct.pack(">I", len(payload)) + kind + payload + struct.pack(">I", zlib.crc32(kind + payload))
    raw = b"".join(b"\0" + pixels[y*w*3:(y+1)*w*3] for y in range(h))
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b"")
    path.with_suffix(".png").write_bytes(png)

for ppm in evidence.rglob("*.ppm"):
    ppm_to_png(ppm)
if not (release / binary.name).exists() or digest(binary) != digest(release / binary.name):
    shutil.copy2(binary, release / binary.name)
for name in ["README.md", "LICENSE", "THIRD_PARTY_NOTICES.md"]:
    shutil.copy2(project / name, release / name)
shutil.copytree(project / "licenses", release / "licenses", dirs_exist_ok=True)
(release / "docs").mkdir(exist_ok=True)
objdump = root / ".tools/mingw64/bin/objdump.exe"
imports = subprocess.check_output([str(objdump), "-p", str(binary)], text=True, errors="replace")
dlls = re.findall(r"DLL Name:\s*(\S+)", imports)
for dll in dlls:
    if dll.lower().startswith(("libgcc", "libstdc", "libwinpthread")):
        dependency = root / ".tools/mingw64/bin" / dll
        if not (release / dll).exists() or digest(dependency) != digest(release / dll):
            shutil.copy2(dependency, release / dll)
(evidence / "imports.json").write_text(json.dumps(dlls, indent=2), encoding="utf-8")
counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", decode_log(evidence / "tests.log"))
assert counts, "No successful test suites in log"
summary = {name: sum(int(row[i]) for row in counts) for i, name in enumerate(["passed", "failed", "ignored"])}
production = [project / "Cargo.toml", project / "Cargo.lock", *sorted((project / "src").rglob("*.rs"))]
info = {
    "version": version, "built_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "target": "x86_64-pc-windows-gnu", "rust": "1.100.0", "profile": "release", "signed": False,
    "executable_bytes": binary.stat().st_size, "executable_sha256": digest(binary),
    "checks": checks, "rust_tests": summary, "extracted_portable": portable, "short_native_soak": soak, "profile_migration": migration, "local_weather": weather, "game_ui": ui_review, "mode_transitions": transitions, "visual_refresh": visual_refresh, "native_smoke": smoke, "real_metrics": real,
    "python_tests": {"scope": "Historical research tests are not certified for this build"},
    "production_sources": {str(p.relative_to(project)).replace('\\', '/'): digest(p) for p in production},
    "imports": dlls,
    "not_validated": ["Windows 10", "mixed-DPI monitors", "Explorer restart", "autohide", "long-duration stability", "taskbar left-widget conflicts"],
    "capture_scope": "Native app framebuffers only; not a desktop compositor screenshot",
}
text = json.dumps(info, ensure_ascii=False, indent=2)
(project / "BUILD-INFO.json").write_text(text, encoding="utf-8")
(release / "BUILD-INFO.json").write_text(text, encoding="utf-8")
# Evidence ships separately; the companion does not need build logs or headless meshes.
with zipfile.ZipFile(root / "release" / f"CPU-Express-{version}-Evidencias.zip", "w", zipfile.ZIP_DEFLATED) as z:
    for path in sorted(evidence.rglob("*")):
        if path.is_file() and path.suffix not in {".ppm", ".lock"}: z.write(path, path.relative_to(project))
for name in ["install-local.ps1", "uninstall-local.ps1"]:
    if (project / "scripts" / name).is_file(): shutil.copy2(project / "scripts" / name, release / name)
shutil.copy2(project / "docs/GUIA_04.md", release / "docs/GUIA_04.md")
shutil.copy2(project / "docs/VALIDACAO_04.md", release / "docs/VALIDACAO_04.md")
(release / "Iniciar_DEMO.cmd").write_text('@echo off\nstart "" "%~dp0cpu-express.exe" --demo\n', encoding="ascii")
(release / "LEIA-ME.txt").write_text(
    "CPU Express 0.4\nAbra cpu-express.exe. Clique no trem para abrir a central; botao direito pausa.\n"
    "Janela: modo integrado/flutuante e deslocamento para evitar widgets.\n"
    "Viagem: paradas, objetivos e salvamento. Visual: vagoes e pinturas.\n"
    "Sair: central ou menu da bandeja. Detalhes em docs/GUIA_04.md.\n", encoding="utf-8")
(release / "SHA256SUMS.txt").write_text(f"{digest(binary)}  cpu-express.exe\n", encoding="ascii")
with zipfile.ZipFile(Path(str(release) + ".zip"), "w", zipfile.ZIP_DEFLATED) as z:
    for path in sorted(release.rglob("*")):
        if path.is_file(): z.write(path, path.relative_to(release.parent))
source_files = [p for p in project.rglob("*") if p.is_file() and not set(p.relative_to(project).parts).intersection({"target", "dist", ".git", "__pycache__"}) and p.name != "SOURCE-MANIFEST.json" and p.suffix not in {".ppm", ".lockfile"}]
(project / "SOURCE-MANIFEST.json").write_text(json.dumps({str(p.relative_to(project)).replace('\\', '/'): digest(p) for p in source_files}, indent=2), encoding="utf-8")
with zipfile.ZipFile(root / "release" / f"CPU-Express-{version}-Fonte.zip", "w", zipfile.ZIP_DEFLATED) as z:
    for path in source_files + [project / "SOURCE-MANIFEST.json"]:
        z.write(path, Path("cpu-express") / path.relative_to(project))
print(json.dumps({"release":str(release), "tests":summary, "binary_sha256":digest(binary)}, indent=2))
