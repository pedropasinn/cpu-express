"""Validate a native app weather run, keeping its actual provider result."""
from pathlib import Path
import hashlib
import json
import sys
project = Path(__file__).resolve().parents[1]
run = Path(sys.argv[1])
records = [json.loads(line) for line in (run / "weather.jsonl").read_text(encoding="utf-8-sig").splitlines()]
assert records[-1]["event"] == "shutdown"
ready = [r for r in records if r.get("local_weather", {}).get("status") == "ready"]
last = records[-1]["local_weather"]
assert last["enabled"] and last["city"] == "São Paulo, SP, Brasil"
result = {"exit_code": 0, "binary_sha256": hashlib.sha256((project / "target/release/cpu-express.exe").read_bytes()).hexdigest(), "online_verified": bool(ready), "ready_samples": len(ready), "weather": last, "evidence_directory": str(run.relative_to(project))}
(project / "evidence/current/weather/result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps(result, ensure_ascii=True, indent=2))
if not ready:
    raise SystemExit("No successful live weather response; inspect result before claiming online validation")
