//! Regressions use handshakes with fake collectors, not operating-system counters.
//! Timeouts below are deadlock watchdogs, never the source of an expected value.

use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use cpu_express::metrics::{MetricStatus, MetricsProvider, MetricsSample, MetricsWorker};

const WATCHDOG: Duration = Duration::from_secs(3);
const PROVIDER_WATCHDOG: Duration = Duration::from_secs(10);

fn ready(cpu: f32) -> MetricsSample {
    MetricsSample {
        cpu_percent: Some(cpu),
        ram_percent: Some(25.0),
        used_memory_bytes: 250,
        total_memory_bytes: 1_000,
        measured_at: Instant::now(),
        status: MetricStatus::Ready,
    }
}

/// A call cannot return until the test supplies its result. Entering call N+1
/// therefore proves that result N has passed through the worker's publication.
struct ControlledProvider {
    results: Receiver<MetricsSample>,
    entered: Sender<usize>,
    calls: usize,
}

impl MetricsProvider for ControlledProvider {
    fn sample(&mut self) -> MetricsSample {
        self.calls += 1;
        let _ = self.entered.send(self.calls);
        self.results
            .recv_timeout(PROVIDER_WATCHDOG)
            .unwrap_or_else(|_| MetricsSample::unavailable("test collector gate closed"))
    }
}

/// Closing the input before joining also releases the collector after an assert
/// fails: a failed test must not leave Drop waiting on its own fake provider.
struct ControlledWorker {
    worker: Option<MetricsWorker>,
    results: Option<Sender<MetricsSample>>,
    entered: Receiver<usize>,
    entered_call: usize,
}

impl ControlledWorker {
    fn new() -> Self {
        let (result_tx, results) = mpsc::channel();
        let (entered, entered_rx) = mpsc::channel();
        let worker = MetricsWorker::spawn_with_provider(
            Duration::from_millis(10),
            ControlledProvider {
                results,
                entered,
                calls: 0,
            },
        )
        .expect("start controlled worker");
        let harness = Self {
            worker: Some(worker),
            results: Some(result_tx),
            entered: entered_rx,
            entered_call: 1,
        };
        assert_eq!(harness.entered.recv_timeout(WATCHDOG).unwrap(), 1);
        harness
    }

    fn publish(&mut self, sample: MetricsSample) {
        self.results
            .as_ref()
            .unwrap()
            .send(sample)
            .expect("deliver fake measurement");
        self.entered_call += 1;
        assert_eq!(
            self.entered
                .recv_timeout(WATCHDOG)
                .expect("worker did not proceed after publishing"),
            self.entered_call
        );
    }

    fn worker(&self) -> &MetricsWorker {
        self.worker.as_ref().unwrap()
    }

    fn release(&mut self) {
        self.results.take();
    }
}

impl Drop for ControlledWorker {
    fn drop(&mut self) {
        self.release();
        self.worker.take();
    }
}

#[test]
fn unread_slot_yields_newest_completed_measurement_and_no_backlog() {
    let mut harness = ControlledWorker::new();
    // Deliberately do not poll between these publications: the old sync_channel
    // retained 10%, discarded 60% and 90%, and fails the assertion below.
    harness.publish(ready(10.0));
    harness.publish(ready(60.0));
    harness.publish(ready(90.0));

    let sample = harness.worker().poll().expect("one pending measurement");
    assert_eq!(sample.cpu_percent, Some(90.0));
    // Call four remains blocked at the provider, so this cannot race a publish.
    assert!(harness.worker().poll().is_none());
}

// END BASELINE-COMPATIBLE REGRESSION.
// Everything above this marker can also be compiled against the original API
// to demonstrate that the previous full-channel policy fails the regression.

use cpu_express::metrics::SampleFreshness;

#[test]
fn ui_can_poll_while_the_next_system_query_is_still_blocked() {
    let mut harness = ControlledWorker::new();
    harness.publish(ready(42.0));
    let worker = harness.worker.take().unwrap();
    let (returned, result) = mpsc::channel();
    let reader = thread::spawn(move || {
        let sample = worker.poll();
        let _ = returned.send(sample);
        worker
    });

    // The answer must arrive while the collector is blocked; only then do we
    // release it. A mutex held across provider.sample() fails this handshake.
    let sample = result.recv_timeout(WATCHDOG);
    harness.release();
    let mut worker = reader.join().expect("reader thread");
    worker.shutdown();
    assert_eq!(
        sample
            .expect("UI waited for the collector")
            .expect("published result")
            .cpu_percent,
        Some(42.0)
    );
}

#[test]
fn temporary_unavailability_recovers_without_fabricating_zero_cpu_or_ram() {
    let mut harness = ControlledWorker::new();
    harness.publish(MetricsSample::unavailable("counter temporarily missing"));
    let missing = harness.worker().poll().unwrap();
    assert_eq!(missing.cpu_percent, None);
    assert_eq!(missing.ram_percent, None);
    assert_eq!(missing.memory_bytes(), None);
    assert_eq!(
        missing.status,
        MetricStatus::Unavailable("counter temporarily missing".into())
    );

    harness.publish(ready(37.0));
    let recovered = harness.worker().poll().unwrap();
    assert_eq!(recovered.status, MetricStatus::Ready);
    assert_eq!(recovered.cpu_percent, Some(37.0));
    assert_eq!(recovered.memory_bytes(), Some((250, 1_000)));
}

#[test]
fn nonfinite_real_cpu_is_unavailable_and_valid_ram_survives() {
    let mut harness = ControlledWorker::new();
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        harness.publish(ready(invalid));
        let sample = harness.worker().poll().unwrap();
        assert_eq!(sample.cpu_percent, None);
        assert!(matches!(sample.status, MetricStatus::Unavailable(_)));
        assert_eq!(sample.ram_percent, Some(25.0));
        assert_eq!(sample.memory_bytes(), Some((250, 1_000)));
    }
}

#[test]
fn finite_cpu_outliers_are_clamped_without_inventing_a_failure() {
    let mut harness = ControlledWorker::new();
    for (input, expected) in [(-12.0, 0.0), (130.0, 100.0)] {
        harness.publish(ready(input));
        let sample = harness.worker().poll().unwrap();
        assert_eq!(sample.cpu_percent, Some(expected));
        assert_eq!(sample.status, MetricStatus::Ready);
    }
}

#[test]
fn warming_up_does_not_expose_an_uninitialized_cpu_counter() {
    let mut harness = ControlledWorker::new();
    let mut sample = ready(86.0);
    sample.status = MetricStatus::WarmingUp;
    harness.publish(sample);
    let sample = harness.worker().poll().unwrap();
    assert_eq!(sample.cpu_percent, None);
    assert_eq!(sample.status, MetricStatus::WarmingUp);
    assert_eq!(sample.memory_bytes(), Some((250, 1_000)));
}

#[test]
fn freshness_boundary_and_age_are_deterministic() {
    let mut sample = ready(50.0);
    let measured_at = Instant::now();
    sample.measured_at = measured_at;
    let threshold = Duration::from_secs(3);

    assert_eq!(sample.age_at(measured_at), Duration::ZERO);
    assert_eq!(sample.age_at(measured_at + threshold), threshold);
    assert_eq!(
        sample.freshness_at(measured_at + threshold, threshold),
        SampleFreshness::Current
    );
    assert_eq!(
        sample.freshness_at(measured_at + threshold + Duration::from_nanos(1), threshold),
        SampleFreshness::Stale
    );
    assert_eq!(
        sample.freshness_at(measured_at, Duration::ZERO),
        SampleFreshness::Current
    );
}

#[test]
fn future_timestamps_are_not_mislabeled_current_by_saturating_age() {
    let now = Instant::now();
    let mut sample = ready(50.0);
    sample.measured_at = now + Duration::from_secs(60);
    assert_eq!(sample.age_at(now), Duration::ZERO);
    assert_eq!(
        sample.freshness_at(now, Duration::from_secs(3)),
        SampleFreshness::Stale
    );
}

#[test]
fn stale_snapshot_clears_readings_and_preserves_the_original_measurement_time() {
    let mut harness = ControlledWorker::new();
    assert_eq!(harness.worker().stale_after(), Duration::from_secs(3));
    let mut old = ready(87.0);
    old.measured_at = Instant::now() - Duration::from_secs(30);
    let measured_at = old.measured_at;
    harness.publish(old.clone());
    let snapshot = harness.worker().poll_snapshot().unwrap();
    assert_eq!(snapshot.freshness, SampleFreshness::Stale);
    assert!(snapshot.age >= Duration::from_secs(30));
    assert_eq!(snapshot.sample.measured_at, measured_at);
    assert_eq!(snapshot.sample.cpu_percent, None);
    assert_eq!(snapshot.sample.ram_percent, None);
    assert_eq!(snapshot.sample.used_memory_bytes, 0);
    assert_eq!(snapshot.sample.total_memory_bytes, 0);
    assert!(matches!(
        snapshot.sample.status,
        MetricStatus::Unavailable(_)
    ));

    // The original app uses poll(), so the compatibility wrapper must enforce
    // the same stale-data protection without requiring an app.rs edit.
    harness.publish(old);
    let via_original_api = harness.worker().poll().unwrap();
    assert_eq!(via_original_api.measured_at, measured_at);
    assert_eq!(via_original_api.cpu_percent, None);
    assert_eq!(via_original_api.ram_percent, None);
    assert!(matches!(
        via_original_api.status,
        MetricStatus::Unavailable(_)
    ));
}

#[test]
fn current_unavailable_snapshot_distinguishes_age_from_counter_status() {
    let mut harness = ControlledWorker::new();
    harness.publish(MetricsSample::unavailable("transient read error"));
    let snapshot = harness.worker().poll_snapshot().unwrap();
    assert_eq!(snapshot.freshness, SampleFreshness::Current);
    assert!(matches!(
        snapshot.sample.status,
        MetricStatus::Unavailable(_)
    ));
    assert_eq!(snapshot.sample.cpu_percent, None);
}

#[test]
fn memory_accessor_does_not_turn_missing_or_inconsistent_counters_into_bytes() {
    assert_eq!(MetricsSample::unavailable("missing").memory_bytes(), None);
    let mut sample = ready(50.0);
    assert_eq!(sample.memory_bytes(), Some((250, 1_000)));
    sample.used_memory_bytes = 1_001;
    assert_eq!(sample.memory_bytes(), None);
    sample.used_memory_bytes = 0;
    sample.total_memory_bytes = 0;
    assert_eq!(sample.memory_bytes(), None);
}

#[test]
fn idle_shutdown_wakes_a_long_timer_and_is_idempotent() {
    struct OneSample;
    impl MetricsProvider for OneSample {
        fn sample(&mut self) -> MetricsSample {
            ready(20.0)
        }
    }

    let mut worker = MetricsWorker::spawn_with_provider(Duration::from_secs(60), OneSample)
        .expect("start idle worker");
    assert_eq!(worker.stale_after(), Duration::from_secs(180));
    let deadline = Instant::now() + WATCHDOG;
    while worker.poll().is_none() {
        assert!(Instant::now() < deadline, "worker never initialized");
        thread::sleep(Duration::from_millis(1));
    }

    let (finished, result) = mpsc::channel();
    let closer = thread::spawn(move || {
        worker.shutdown();
        worker.shutdown();
        let _ = finished.send(());
    });
    result
        .recv_timeout(WATCHDOG)
        .expect("shutdown waited for the sixty-second sample interval");
    closer.join().unwrap();
}

#[test]
fn provider_minimum_interval_also_sets_the_freshness_window() {
    struct SlowCounters;
    impl MetricsProvider for SlowCounters {
        fn sample(&mut self) -> MetricsSample {
            ready(20.0)
        }

        fn minimum_interval(&self) -> Duration {
            Duration::from_secs(15)
        }
    }

    let mut worker = MetricsWorker::spawn_with_provider(Duration::ZERO, SlowCounters).unwrap();
    assert_eq!(worker.stale_after(), Duration::from_secs(45));
    worker.shutdown();
}

#[test]
fn a_panicking_provider_publishes_an_explicit_terminal_failure() {
    struct BrokenProvider;
    impl MetricsProvider for BrokenProvider {
        fn sample(&mut self) -> MetricsSample {
            panic!("deliberate fake-provider failure");
        }
    }

    let mut worker =
        MetricsWorker::spawn_with_provider(Duration::from_secs(60), BrokenProvider).unwrap();
    let deadline = Instant::now() + WATCHDOG;
    let failed = loop {
        if let Some(sample) = worker.poll()
            && matches!(sample.status, MetricStatus::Unavailable(_))
        {
            break sample;
        }
        assert!(Instant::now() < deadline, "worker failure was lost");
        thread::sleep(Duration::from_millis(1));
    };
    assert_eq!(failed.cpu_percent, None);
    assert_eq!(failed.ram_percent, None);
    assert_eq!(failed.memory_bytes(), None);
    worker.shutdown();
    assert!(worker.poll().is_none());
}
