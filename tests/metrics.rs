use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use cpu_express::metrics::{
    ManualProvider, MetricStatus, MetricsProvider, MetricsSample, MetricsWorker, SysinfoProvider,
};

fn await_sample(worker: &MetricsWorker) -> MetricsSample {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(sample) = worker.poll() {
            return sample;
        }
        assert!(Instant::now() < deadline, "worker did not publish a sample");
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn manual_provider_sanitizes_cpu_and_does_not_fabricate_memory() {
    let mut provider = ManualProvider::new(50.0);
    let sample = provider.sample();
    assert_eq!(sample.cpu_percent, Some(50.0));
    assert_eq!(sample.ram_percent, None);
    assert_eq!(sample.total_memory_bytes, 0);
    for (input, expected) in [(-5.0, 0.0), (125.0, 100.0), (f32::NAN, 0.0)] {
        provider.set_cpu(input);
        assert_eq!(provider.sample().cpu_percent, Some(expected));
    }
}

#[test]
fn first_real_cpu_sample_and_immediate_repeat_are_not_reported_as_valid() {
    let mut provider =
        SysinfoProvider::new().expect("sysinfo initialization on supported test host");
    let first = provider.sample();
    assert_eq!(first.cpu_percent, None);
    assert_eq!(provider.sample().cpu_percent, None);
    assert!(provider.minimum_interval() >= sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
}

#[test]
fn sleeping_worker_shuts_down_without_waiting_for_the_sample_interval() {
    let worker =
        MetricsWorker::spawn_with_provider(Duration::from_secs(60), ManualProvider::new(25.0))
            .expect("create test worker");
    assert_eq!(await_sample(&worker).cpu_percent, Some(25.0));
    let before = Instant::now();
    drop(worker);
    assert!(before.elapsed() < Duration::from_secs(1));
}

#[test]
fn full_sample_queue_does_not_block_the_collector_or_shutdown() {
    struct CountingProvider(Arc<AtomicUsize>);
    impl MetricsProvider for CountingProvider {
        fn sample(&mut self) -> MetricsSample {
            self.0.fetch_add(1, Ordering::Relaxed);
            ManualProvider::new(42.0).sample()
        }
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let worker = MetricsWorker::spawn_with_provider(
        Duration::from_millis(10),
        CountingProvider(Arc::clone(&calls)),
    )
    .expect("create test worker");
    let deadline = Instant::now() + Duration::from_secs(2);
    // No polling: the single queue slot stays full while sampling continues.
    while calls.load(Ordering::Relaxed) < 4 {
        assert!(
            Instant::now() < deadline,
            "collector blocked on a full queue"
        );
        thread::sleep(Duration::from_millis(5));
    }
    // poll is intentionally nonblocking: a concurrent publication may briefly
    // hold its mutex even though the slot is full. Await availability, not a
    // particular scheduling interleaving.
    assert_eq!(await_sample(&worker).cpu_percent, Some(42.0));
    let before = Instant::now();
    drop(worker);
    assert!(before.elapsed() < Duration::from_secs(1));
}

#[test]
fn unavailable_provider_publishes_an_error_without_stopping_the_app() {
    struct UnavailableProvider;
    impl MetricsProvider for UnavailableProvider {
        fn sample(&mut self) -> MetricsSample {
            MetricsSample::unavailable("test counter unavailable")
        }
    }
    let worker = MetricsWorker::spawn_with_provider(Duration::from_secs(1), UnavailableProvider)
        .expect("create test worker");
    let sample = await_sample(&worker);
    assert_eq!(sample.cpu_percent, None);
    assert_eq!(sample.ram_percent, None);
    assert_eq!(
        sample.status,
        MetricStatus::Unavailable("test counter unavailable".to_owned())
    );
}
