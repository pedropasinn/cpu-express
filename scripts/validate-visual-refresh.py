"""Check native frame pacing, visible client bounds and four mode transitions."""
from pathlib import Path
import hashlib
import json
import sys
from PIL import Image

project = Path(__file__).resolve().parents[1]
run = Path(sys.argv[1])
evidence = project / "evidence/current"
binary = project / "target/release/cpu-express.exe"
sha = hashlib.sha256(binary.read_bytes()).hexdigest()
def rows(path):
    return [json.loads(line) for line in path.read_text().splitlines()]
def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2), encoding="utf-8")

comparison = {"scope": "Short local runs; same 60 FPS limit, DEMO, 640x180 floating window, closed panel. Rendered frames, not compositor frames or CPU cost."}
for tag, path, executable in [("before", evidence / "visual-refresh/baseline.jsonl", project / "../../release/CPU-Express-0.3.0-rc.4/cpu-express.exe"), ("after", run / "after.jsonl", binary)]:
    records = rows(path)
    stable = [r for r in records if r["elapsed_seconds"] >= 2]
    first, last = stable[0], stable[-1]
    assert last["event"] == "shutdown"
    comparison[tag] = {"binary_sha256": hashlib.sha256(executable.read_bytes()).hexdigest(), "rendered_frames": last["rendered_frames"], "elapsed_seconds": last["elapsed_seconds"], "warm_average_fps": (last["rendered_frames"]-first["rendered_frames"])/(last["elapsed_seconds"]-first["elapsed_seconds"]), "settings": {"fps":60,"mode":"demo","window":[640,180]}}
save(evidence / "visual-refresh/comparison.json", comparison)
real = rows(run / "real.jsonl")
valid = [r for r in real if r["mode"] == "real" and r["metric_status"] == "ready" and r["cpu_percent"] is not None and r["ram_percent"] is not None]
assert len(valid) >= 2 and real[-1]["event"] == "shutdown"
save(evidence / "real-metrics-result.json", {"exit_code":0,"binary_sha256":sha,"valid_real_samples":len(valid),"last":real[-1]})
records = rows(run / "transitions.jsonl")
assert records[-1]["event"] == "shutdown"
stages = []
for stage in range(5):
    stable = [r for r in records if stage*4+1.5 <= r["elapsed_seconds"] < stage*4+3.9]
    assert stable, stage
    for r in stable:
        window = r["native_window"]
        assert r["requested_taskbar_mode"] == (stage%2 == 0)
        assert window["panel"]["visible"]
        if stage%2 == 0:
            assert window["taskbar_aligned"] and window["height"] == window["taskbar_height"]
        for actual, expected in zip(r["render_surface_size"], [window["client_width"],window["client_height"]]):
            assert abs(actual-expected) < 2, (stage, r)
    capture = run / f"faixa-stage-{stage}.ppm"
    assert capture.exists()
    Image.open(capture).save(capture.with_suffix(".png"))
    stages.append({"stage":stage,"taskbar":stage%2==0,"geometry":stable[-1]["native_window"],"render_surface_size":stable[-1]["render_surface_size"]})
save(evidence / "transitions-expansion/result.json", {"exit_code":0,"binary_sha256":sha,"evidence_directory":str(run.relative_to(project)),"stages":stages})
print(json.dumps({"pacing":comparison,"mode_transitions":"four passed","real_samples":len(valid)},indent=2))
