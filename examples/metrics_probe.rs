//! Read three real CPU samples without creating artificial processor load.
//! Run with `cargo run --example metrics_probe` on the machine to inspect.

use std::{
    thread,
    time::{Duration, Instant},
};

use cpu_express::metrics::{MetricStatus, MetricsWorker};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let worker = MetricsWorker::spawn(Duration::from_secs(1))?;
    let start = Instant::now();
    let mut ready_samples = 0;
    println!("elapsed_s,cpu_percent,ram_percent,used_bytes,total_bytes,status");
    while ready_samples < 3 && start.elapsed() < Duration::from_secs(10) {
        if let Some(sample) = worker.poll() {
            println!(
                "{:.3},{},{},{},{},{:?}",
                start.elapsed().as_secs_f64(),
                sample
                    .cpu_percent
                    .map_or_else(|| "NA".to_owned(), |v| format!("{v:.3}")),
                sample
                    .ram_percent
                    .map_or_else(|| "NA".to_owned(), |v| format!("{v:.3}")),
                sample.used_memory_bytes,
                sample.total_memory_bytes,
                sample.status,
            );
            if sample.status == MetricStatus::Ready {
                ready_samples += 1;
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
    if ready_samples < 3 {
        return Err("could not obtain three valid samples within ten seconds".into());
    }
    Ok(())
}
