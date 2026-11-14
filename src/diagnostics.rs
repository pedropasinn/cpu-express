//! Explicit, optional local diagnostics for reproducible smoke tests.
//! Normal gameplay does not write a history or send telemetry anywhere.
//!
//! Metric records use schema v2 with new metadata and nullable byte fields.
//! Missing RAM is represented by JSON
//! null, not the legacy in-memory 0/0 byte sentinels. Stale real observations
//! are also null; `effective_cpu_percent` remains the simulation's input,
//! which may be a manual value or fallback rather than a real measurement.

use std::{
    borrow::Cow,
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

use serde_json::Value;

const MAX_RECORDS: u64 = 86_402;
const METRICS_SCHEMA_VERSION: u64 = 2;

pub struct Diagnostics {
    output: BufWriter<File>,
    records: u64,
}

impl Diagnostics {
    pub fn open(path: &Path) -> io::Result<Self> {
        Ok(Self {
            output: BufWriter::new(File::create(path)?),
            records: 0,
        })
    }

    pub fn record(&mut self, value: &Value) -> io::Result<()> {
        // Even an accidentally unattended diagnostic run has a finite output.
        if self.records >= MAX_RECORDS {
            return Ok(());
        }
        let normalized = normalize_record(value);
        serde_json::to_writer(&mut self.output, normalized.as_ref()).map_err(io::Error::other)?;
        self.output.write_all(b"\n")?;
        self.output.flush()?;
        self.records += 1;
        Ok(())
    }
}

fn valid_percent(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_f64)
        .is_some_and(|number| number.is_finite() && (0.0..=100.0).contains(&number))
}

fn normalize_record(value: &Value) -> Cow<'_, Value> {
    let Some(original) = value.as_object() else {
        return Cow::Borrowed(value);
    };
    if ![
        "cpu_percent",
        "ram_percent",
        "used_memory_bytes",
        "total_memory_bytes",
        "metric_status",
        "metric_age_seconds",
        "metrics_stale",
    ]
    .iter()
    .any(|key| original.contains_key(*key))
    {
        // Preserve generic diagnostic records byte-for-value, including their
        // own metadata; the metric schema only applies to metric records.
        return Cow::Borrowed(value);
    }

    let mut object = original.clone();
    let stale = object.get("metrics_stale").and_then(Value::as_bool);
    object.insert(
        "diagnostics_schema_version".to_owned(),
        Value::from(METRICS_SCHEMA_VERSION),
    );
    object.insert(
        "metric_freshness".to_owned(),
        match stale {
            Some(true) => Value::from("stale"),
            Some(false) => Value::from("current"),
            // Do not declare a reading current when a generic caller supplied
            // no usable age/freshness classification. The app supplies bool.
            None => Value::Null,
        },
    );

    if stale == Some(true)
        || (object.contains_key("cpu_percent") && !valid_percent(object.get("cpu_percent")))
    {
        object.insert("cpu_percent".to_owned(), Value::Null);
    }

    let valid_ram = valid_percent(object.get("ram_percent"))
        && matches!(
            (
                object.get("used_memory_bytes").and_then(Value::as_u64),
                object.get("total_memory_bytes").and_then(Value::as_u64),
            ),
            (Some(used), Some(total)) if total > 0 && used <= total
        );
    if stale == Some(true) || !valid_ram {
        for field in ["ram_percent", "used_memory_bytes", "total_memory_bytes"] {
            object.insert(field.to_owned(), Value::Null);
        }
    }
    Cow::Owned(Value::Object(object))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn measured_record() -> Value {
        json!({
            "event": "sample",
            "cpu_percent": 37.0,
            "ram_percent": 50.0,
            "used_memory_bytes": 512,
            "total_memory_bytes": 1024,
            "metric_status": "ready",
            "metrics_stale": false,
            "metric_age_seconds": 0.25,
            "effective_cpu_percent": 37.0,
            "rendered_frames": 30,
            "hidden": false,
            "extra": { "keep": true },
        })
    }

    #[test]
    fn valid_memory_and_unrelated_fields_are_preserved() {
        let input = measured_record();
        let normalized = normalize_record(&input);
        for (key, value) in input.as_object().expect("object") {
            assert_eq!(normalized[key], *value, "field changed: {key}");
        }
        assert_eq!(normalized["diagnostics_schema_version"], 2);
        assert_eq!(normalized["metric_freshness"], "current");
        assert!(input.get("diagnostics_schema_version").is_none());
    }

    #[test]
    fn absent_or_invalid_memory_is_null_in_all_three_fields() {
        for replacement in [
            json!({}),
            json!({"ram_percent": null, "used_memory_bytes": 0, "total_memory_bytes": 0}),
            json!({"ram_percent": 50.0, "used_memory_bytes": 0, "total_memory_bytes": 0}),
            json!({"ram_percent": 50.0, "used_memory_bytes": 101, "total_memory_bytes": 100}),
            json!({"ram_percent": 101.0, "used_memory_bytes": 1, "total_memory_bytes": 100}),
            json!({"ram_percent": "50", "used_memory_bytes": 50, "total_memory_bytes": 100}),
            json!({"ram_percent": 50.0, "used_memory_bytes": -1, "total_memory_bytes": 100}),
            json!({"ram_percent": 50.0, "used_memory_bytes": 50}),
        ] {
            let mut input = json!({ "cpu_percent": 25.0, "metrics_stale": false });
            input
                .as_object_mut()
                .expect("object")
                .extend(replacement.as_object().expect("object").clone());
            let normalized = normalize_record(&input);
            for key in ["ram_percent", "used_memory_bytes", "total_memory_bytes"] {
                assert!(normalized[key].is_null(), "{key} with input {input}");
            }
            assert_eq!(normalized["cpu_percent"], 25.0);
        }
    }

    #[test]
    fn measured_zero_memory_usage_is_preserved_with_a_valid_total() {
        let input = json!({
            "ram_percent": 0.0, "used_memory_bytes": 0,
            "total_memory_bytes": 1024, "metrics_stale": false,
        });
        let normalized = normalize_record(&input);
        assert_eq!(normalized["ram_percent"], 0.0);
        assert_eq!(normalized["used_memory_bytes"], 0);
        assert_eq!(normalized["total_memory_bytes"], 1024);
    }

    #[test]
    fn stale_observations_do_not_become_current_real_metrics() {
        let mut input = measured_record();
        input["metrics_stale"] = json!(true);
        input["metric_age_seconds"] = json!(60.0);
        let normalized = normalize_record(&input);
        for key in [
            "cpu_percent",
            "ram_percent",
            "used_memory_bytes",
            "total_memory_bytes",
        ] {
            assert!(normalized[key].is_null());
        }
        assert_eq!(normalized["metric_freshness"], "stale");
        assert_eq!(normalized["metric_status"], "ready");
        assert_eq!(normalized["metric_age_seconds"], 60.0);
        assert_eq!(normalized["rendered_frames"], 30);
    }

    #[test]
    fn manual_and_fallback_inputs_remain_explicit_even_if_real_metrics_are_stale() {
        for (mode, effective) in [("manual", 72.0), ("real", 0.0)] {
            let mut input = measured_record();
            input["mode"] = json!(mode);
            input["effective_cpu_percent"] = json!(effective);
            input["metrics_stale"] = json!(true);
            input["metric_status"] = json!("unavailable");
            let normalized = normalize_record(&input);
            assert!(normalized["cpu_percent"].is_null());
            assert_eq!(normalized["effective_cpu_percent"], effective);
            assert_eq!(normalized["mode"], mode);
            assert_eq!(normalized["metric_status"], "unavailable");
        }
    }

    #[test]
    fn invalid_cpu_is_null_without_erasing_valid_ram_or_simulation_input() {
        for invalid in [
            Value::Null,
            json!("37"),
            json!(-1),
            json!(101),
            json!(f64::NAN),
        ] {
            let mut input = measured_record();
            input["cpu_percent"] = invalid;
            let normalized = normalize_record(&input);
            assert!(normalized["cpu_percent"].is_null());
            assert_eq!(normalized["ram_percent"], 50.0);
            assert_eq!(normalized["used_memory_bytes"], 512);
            assert_eq!(normalized["effective_cpu_percent"], 37.0);
        }
    }

    #[test]
    fn generic_records_are_unchanged_and_missing_freshness_is_not_invented() {
        for input in [
            json!({"event": "hello", "custom": [1, 2]}),
            json!([1, 2]),
            json!(null),
        ] {
            assert_eq!(normalize_record(&input).as_ref(), &input);
        }
        let input = json!({"cpu_percent": 37.0});
        assert!(normalize_record(&input)["metric_freshness"].is_null());
    }

    struct TestFile {
        directory: PathBuf,
        path: PathBuf,
    }

    impl TestFile {
        fn new() -> Self {
            static NEXT_ID: AtomicU64 = AtomicU64::new(0);
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock is after Unix epoch")
                .as_nanos();
            let directory = std::env::temp_dir().join(format!(
                "cpu-express-diagnostics-{}-{timestamp}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed),
            ));
            fs::create_dir(&directory).expect("create a unique temporary test directory");
            let path = directory.join("samples.jsonl");
            Self { directory, path }
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
            let _ = fs::remove_dir(&self.directory);
        }
    }

    #[test]
    fn jsonl_is_normalized_flushed_and_bounded_without_a_large_fixture() {
        let file = TestFile::new();
        let mut diagnostics = Diagnostics::open(&file.path).expect("open diagnostics");
        diagnostics
            .record(&measured_record())
            .expect("write valid sample");
        let mut stale = measured_record();
        stale["metrics_stale"] = json!(true);
        diagnostics.record(&stale).expect("write stale sample");
        diagnostics.records = MAX_RECORDS - 1;
        diagnostics
            .record(&json!({"event": "last"}))
            .expect("write last permitted record");
        diagnostics
            .record(&json!({"event": "excluded"}))
            .expect("bounded no-op");
        assert_eq!(diagnostics.records, MAX_RECORDS);
        // Read before dropping the writer to verify that record() flushed.
        let contents = fs::read_to_string(&file.path).expect("read JSONL");
        let records: Vec<Value> = contents
            .lines()
            .map(|line| serde_json::from_str(line).expect("JSONL line"))
            .collect();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0]["diagnostics_schema_version"], 2);
        assert_eq!(records[0]["total_memory_bytes"], 1024);
        assert!(records[1]["cpu_percent"].is_null());
        assert!(records[1]["total_memory_bytes"].is_null());
        assert_eq!(records[1]["metric_freshness"], "stale");
        assert_eq!(records[2], json!({"event": "last"}));
        assert!(contents.ends_with('\n'));
    }
}
