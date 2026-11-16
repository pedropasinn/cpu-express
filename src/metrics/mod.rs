//! Real metrics collected on one worker and handed off through one replaceable slot.
//! The UI never waits for a provider, a full queue, or a held snapshot mutex.

#[cfg(windows)]
mod machine_windows;
mod sysinfo_provider;
use crate::simulation::machine::MachineSignals;
use std::sync::atomic::{AtomicBool, Ordering};

use std::{
    io,
    panic::{self, AssertUnwindSafe},
    sync::{
        Arc, Mutex, TryLockError,
        mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TryRecvError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub use sysinfo_provider::SysinfoProvider;

#[derive(Clone, Debug, PartialEq)]
pub enum MetricStatus {
    WarmingUp,
    Ready,
    Unavailable(String),
}

/// Freshness is separate from provider status so existing exhaustive matches work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampleFreshness {
    Current,
    Stale,
}

/// A snapshot owns its numbers; no sysinfo state or locks reach the UI.
///
/// The legacy byte fields use zero as an absence sentinel when `ram_percent` is
/// `None`. Use `memory_bytes()` to avoid displaying those sentinels as readings.
/// `Unavailable` can contain one valid metric when the other metric failed.
#[derive(Clone, Debug)]
pub struct MetricsSample {
    pub machine: MachineSignals,
    pub cpu_percent: Option<f32>,
    pub ram_percent: Option<f32>,
    pub used_memory_bytes: u64,
    pub total_memory_bytes: u64,
    pub measured_at: Instant,
    pub status: MetricStatus,
}

impl MetricsSample {
    #[must_use]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self {
            machine: MachineSignals::default(),
            cpu_percent: None,
            ram_percent: None,
            used_memory_bytes: 0,
            total_memory_bytes: 0,
            measured_at: Instant::now(),
            status: MetricStatus::Unavailable(message.into()),
        }
    }

    #[must_use]
    pub fn age(&self) -> Duration {
        self.age_at(Instant::now())
    }

    /// Deterministic counterpart of `age`. A future timestamp saturates at zero;
    /// `freshness_at` rejects it, rather than treating it as a current reading.
    #[must_use]
    pub fn age_at(&self, now: Instant) -> Duration {
        now.saturating_duration_since(self.measured_at)
    }

    #[must_use]
    pub fn freshness(&self, max_age: Duration) -> SampleFreshness {
        self.freshness_at(Instant::now(), max_age)
    }

    /// The age limit is inclusive. Clock anomalies/future timestamps are stale.
    #[must_use]
    pub fn freshness_at(&self, now: Instant, max_age: Duration) -> SampleFreshness {
        if now
            .checked_duration_since(self.measured_at)
            .is_some_and(|age| age <= max_age)
        {
            SampleFreshness::Current
        } else {
            SampleFreshness::Stale
        }
    }

    /// A missing RAM reading is not a measurement of zero physical memory.
    #[must_use]
    pub fn memory_bytes(&self) -> Option<(u64, u64)> {
        let percent = self.ram_percent?;
        (percent.is_finite()
            && (0.0..=100.0).contains(&percent)
            && self.total_memory_bytes > 0
            && self.used_memory_bytes <= self.total_memory_bytes)
            .then_some((self.used_memory_bytes, self.total_memory_bytes))
    }

    fn clear_readings(&mut self) {
        self.machine = MachineSignals::default();
        self.cpu_percent = None;
        self.ram_percent = None;
        self.used_memory_bytes = 0;
        self.total_memory_bytes = 0;
    }

    /// Validate an injected provider too, without overwriting a legitimate
    /// partial reading or changing the observation's original timestamp.
    fn validated(mut self) -> Self {
        self.machine = self.machine.validated();
        let mut issue = None;
        if let Some(cpu) = self.cpu_percent {
            if cpu.is_finite() {
                self.cpu_percent = Some(cpu.clamp(0.0, 100.0));
            } else {
                self.cpu_percent = None;
                issue = Some("The provider returned a non-finite CPU value");
            }
        }
        if matches!(self.status, MetricStatus::WarmingUp) {
            self.cpu_percent = None;
        } else if matches!(self.status, MetricStatus::Ready) && self.cpu_percent.is_none() {
            issue.get_or_insert("The provider returned no usable CPU reading");
        }

        let memory_supplied = self.ram_percent.is_some()
            || self.used_memory_bytes != 0
            || self.total_memory_bytes != 0;
        if let Some((used, total)) = self.memory_bytes() {
            // Derive the percentage from the same byte pair, avoiding two
            // contradictory memory measurements from a custom provider.
            self.ram_percent = Some((100.0 * used as f64 / total as f64) as f32);
        } else {
            self.ram_percent = None;
            self.used_memory_bytes = 0;
            self.total_memory_bytes = 0;
            if memory_supplied {
                issue.get_or_insert("The provider returned invalid physical memory data");
            }
        }
        if let Some(message) = issue {
            self.status = MetricStatus::Unavailable(match self.status {
                MetricStatus::Unavailable(original) => format!("{original}; {message}"),
                _ => message.to_owned(),
            });
        }
        self
    }
}

/// A consumption-time view. Stale samples retain their original timestamp but
/// expose no numbers; `freshness` still distinguishes age from a provider error.
#[derive(Clone, Debug)]
pub struct MetricsSnapshot {
    pub sample: MetricsSample,
    pub age: Duration,
    pub freshness: SampleFreshness,
}

impl MetricsSnapshot {
    fn at(mut sample: MetricsSample, now: Instant, max_age: Duration) -> Self {
        let age = sample.age_at(now);
        let freshness = sample.freshness_at(now, max_age);
        if freshness == SampleFreshness::Stale {
            sample.clear_readings();
            sample.status = MetricStatus::Unavailable(match sample.status {
                MetricStatus::Unavailable(original) => {
                    format!("The metrics sample is stale; {original}")
                }
                _ => "The metrics sample is stale".to_owned(),
            });
        }
        Self {
            sample,
            age,
            freshness,
        }
    }
}

/// The worker owns this provider. A fake can exercise the same path as sysinfo.
pub trait MetricsProvider: Send {
    fn sample(&mut self) -> MetricsSample;

    /// A cheap policy query; it must not collect metrics or wait for the OS.
    fn minimum_interval(&self) -> Duration {
        Duration::ZERO
    }
}

/// A CPU-only provider for tests and experiments; it does not invent RAM data.
/// Non-finite manual inputs deliberately retain the baseline's simulated 0%.
/// This input normalization is never used to conceal a real provider failure.
#[derive(Clone, Debug)]
pub struct ManualProvider {
    cpu_percent: f32,
    machine: MachineSignals,
}

impl ManualProvider {
    #[must_use]
    pub fn new(cpu_percent: f32) -> Self {
        let mut provider = Self {
            cpu_percent: 0.0,
            machine: MachineSignals::default(),
        };
        provider.set_cpu(cpu_percent);
        provider
    }

    pub fn set_machine(&mut self, signals: MachineSignals) {
        self.machine = signals.validated();
    }

    pub fn set_cpu(&mut self, cpu_percent: f32) {
        self.cpu_percent = if cpu_percent.is_finite() {
            cpu_percent.clamp(0.0, 100.0)
        } else {
            0.0
        };
    }
}

impl MetricsProvider for ManualProvider {
    fn sample(&mut self) -> MetricsSample {
        MetricsSample {
            machine: self.machine,
            cpu_percent: Some(self.cpu_percent),
            ram_percent: None,
            used_memory_bytes: 0,
            total_memory_bytes: 0,
            measured_at: Instant::now(),
            status: MetricStatus::Ready,
        }
    }
}

/// The lock protects only an owned Option swap, never provider work, validation,
/// allocation or the destruction of an overwritten error String.
#[derive(Default)]
struct LatestSample {
    slot: Mutex<Option<MetricsSample>>,
}

impl LatestSample {
    fn publish(&self, sample: MetricsSample) {
        let previous = {
            let mut slot = self.slot.lock().unwrap_or_else(|poisoned| {
                // The only invariant is a valid Option<MetricsSample>; no user
                // code runs under this lock. Safe recovery cannot expose a
                // partially mutated provider or system object.
                self.slot.clear_poison();
                poisoned.into_inner()
            });
            slot.replace(sample)
        };
        drop(previous);
    }

    fn try_take(&self) -> Option<MetricsSample> {
        match self.slot.try_lock() {
            Ok(mut slot) => slot.take(),
            Err(TryLockError::WouldBlock) => None,
            Err(TryLockError::Poisoned(poisoned)) => {
                self.slot.clear_poison();
                poisoned.into_inner().take()
            }
        }
    }
}

/// One replaceable sample, one bounded stop signal and one owned worker thread.
///
/// Publication replaces an unread value. `poll` uses `try_lock`: contention
/// returns None immediately and leaves the latest value available for next time.
/// No system call runs while the slot is locked, and no metrics history grows.
pub struct MetricsWorker {
    input_enabled: Arc<AtomicBool>,
    samples: Arc<LatestSample>,
    stop: SyncSender<()>,
    thread: Option<JoinHandle<()>>,
    stale_after: Duration,
}

impl MetricsWorker {
    /// Initialize sysinfo inside the worker, including expensive Windows PDH
    /// initialization. Recoverable constructor errors retry at most once/second.
    pub fn spawn(interval: Duration) -> io::Result<Self> {
        Self::spawn_with_input(interval, true)
    }

    pub fn spawn_with_input(interval: Duration, enabled: bool) -> io::Result<Self> {
        let input_enabled = Arc::new(AtomicBool::new(enabled));
        let collector_input = Arc::clone(&input_enabled);
        let interval = sampling_interval(
            interval.max(Duration::from_secs(1)),
            sysinfo::MINIMUM_CPU_UPDATE_INTERVAL,
        );
        let mut worker = Self::launch(interval, move |stop, samples| {
            let retry_interval = interval.max(Duration::from_secs(1));
            loop {
                if stop_is_pending(&stop) {
                    return;
                }
                match SysinfoProvider::with_input(Arc::clone(&collector_input)) {
                    Ok(provider) => {
                        run_provider(interval, provider, stop, samples);
                        return;
                    }
                    Err(error) => {
                        samples.publish(MetricsSample::unavailable(error));
                        if stop_requested(&stop, retry_interval) {
                            return;
                        }
                    }
                }
            }
        })?;
        worker.input_enabled = input_enabled;
        Ok(worker)
    }

    pub fn set_input_enabled(&self, enabled: bool) {
        self.input_enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn spawn_with_provider(
        interval: Duration,
        provider: impl MetricsProvider + 'static,
    ) -> io::Result<Self> {
        let interval = sampling_interval(interval, provider.minimum_interval());
        Self::launch(interval, move |stop, samples| {
            run_provider(interval, provider, stop, samples);
        })
    }

    fn launch(
        interval: Duration,
        run: impl FnOnce(Receiver<()>, Arc<LatestSample>) + Send + 'static,
    ) -> io::Result<Self> {
        let samples = Arc::new(LatestSample::default());
        let output = Arc::clone(&samples);
        let (stop, stop_rx) = mpsc::sync_channel(1);
        let thread = thread::Builder::new()
            .name("cpu-express-metrics".to_owned())
            .spawn(move || {
                let panic_output = Arc::clone(&output);
                // A panicking provider is never reused. With unwind-enabled
                // builds the UI receives an explicit error instead of an old
                // Ready value indefinitely. Panic=abort cannot be recovered.
                if panic::catch_unwind(AssertUnwindSafe(|| run(stop_rx, output))).is_err() {
                    panic_output.publish(MetricsSample::unavailable(
                        "The metrics worker panicked; restart the application to retry",
                    ));
                }
            })?;
        Ok(Self {
            input_enabled: Arc::new(AtomicBool::new(false)),
            samples,
            stop,
            thread: Some(thread),
            stale_after: interval.saturating_mul(3).max(Duration::from_secs(3)),
        })
    }

    /// Maximum accepted sample age: max(3 * actual sampling interval, 3 seconds).
    #[must_use]
    pub fn stale_after(&self) -> Duration {
        self.stale_after
    }

    /// Nonblocking legacy wrapper. A stale pending sample contains no readings.
    /// Consumers caching returned samples must keep checking their age too.
    pub fn poll(&self) -> Option<MetricsSample> {
        self.poll_snapshot().map(|snapshot| snapshot.sample)
    }

    /// Freshness is computed at consumption, not when the worker published it.
    /// None means empty or briefly contended; it is not a CPU measurement of 0%.
    /// This consumes the same slot as `poll`; choose one API per update, since
    /// neither method peeks or supplies an independent copy to another caller.
    pub fn poll_snapshot(&self) -> Option<MetricsSnapshot> {
        let sample = self.samples.try_take()?;
        Some(MetricsSnapshot::at(
            sample,
            Instant::now(),
            self.stale_after,
        ))
    }

    /// Wake the interruptible timer and join once. Repeated calls are harmless.
    /// An in-progress OS/provider call must finish; Rust cannot safely kill it.
    pub fn shutdown(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = self.stop.try_send(());
            if thread.join().is_err() {
                eprintln!("CPU Express: the metrics worker terminated unexpectedly");
            }
        }
    }
}

impl Drop for MetricsWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn sampling_interval(requested: Duration, minimum: Duration) -> Duration {
    // Even a fake with a zero interval must not accidentally create a busy loop.
    requested.max(minimum).max(Duration::from_millis(10))
}

fn run_provider(
    interval: Duration,
    mut provider: impl MetricsProvider,
    stop: Receiver<()>,
    samples: Arc<LatestSample>,
) {
    loop {
        if stop_is_pending(&stop) {
            return;
        }
        // All potentially slow system calls and validation happen off-lock.
        let sample = provider.sample().validated();
        if stop_is_pending(&stop) {
            return;
        }
        samples.publish(sample);
        if stop_requested(&stop, interval) {
            return;
        }
    }
}

fn stop_is_pending(stop: &Receiver<()>) -> bool {
    match stop.try_recv() {
        Ok(()) | Err(TryRecvError::Disconnected) => true,
        Err(TryRecvError::Empty) => false,
    }
}

fn stop_requested(stop: &Receiver<()>, interval: Duration) -> bool {
    match stop.recv_timeout(interval) {
        Ok(()) | Err(RecvTimeoutError::Disconnected) => true,
        Err(RecvTimeoutError::Timeout) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_poll_returns_while_the_slot_is_locked() {
        let samples = Arc::new(LatestSample::default());
        samples.publish(ManualProvider::new(47.0).sample());
        let (stop, _stop_rx) = mpsc::sync_channel(1);
        let worker = MetricsWorker {
            input_enabled: Arc::new(AtomicBool::new(false)),
            samples: Arc::clone(&samples),
            stop,
            thread: None,
            stale_after: Duration::from_secs(3),
        };
        let guard = samples.slot.lock().expect("lock test slot");
        let (result_tx, result_rx) = mpsc::channel();
        let reader = thread::spawn(move || {
            result_tx.send(worker.poll()).expect("send poll result");
        });
        let result = result_rx.recv_timeout(Duration::from_secs(2));
        // Release before any assertion so a broken blocking implementation can
        // finish, rather than hanging the test process on failure.
        drop(guard);
        reader.join().expect("reader did not panic");
        assert!(matches!(result, Ok(None)));
        assert_eq!(samples.try_take().unwrap().cpu_percent, Some(47.0));
    }

    #[test]
    fn a_poisoned_slot_is_recoverable_without_losing_the_owned_sample() {
        let samples = Arc::new(LatestSample::default());
        samples.publish(ManualProvider::new(63.0).sample());
        let poison = Arc::clone(&samples);
        assert!(
            thread::spawn(move || {
                let _guard = poison.slot.lock().unwrap();
                panic!("intentional slot poisoning in test");
            })
            .join()
            .is_err()
        );
        assert_eq!(samples.try_take().unwrap().cpu_percent, Some(63.0));
        assert!(!samples.slot.is_poisoned());
        samples.publish(ManualProvider::new(72.0).sample());
        assert_eq!(samples.try_take().unwrap().cpu_percent, Some(72.0));
    }

    #[test]
    fn stop_disconnection_is_an_exit_signal() {
        let (tx, rx) = mpsc::sync_channel(1);
        drop(tx);
        assert!(stop_is_pending(&rx));
        assert!(stop_requested(&rx, Duration::from_secs(60)));
    }

    #[test]
    fn provider_minimum_and_busy_loop_floor_are_respected() {
        assert_eq!(
            sampling_interval(Duration::ZERO, Duration::ZERO),
            Duration::from_millis(10)
        );
        assert_eq!(
            sampling_interval(Duration::from_millis(10), Duration::from_secs(2)),
            Duration::from_secs(2)
        );
    }

    #[test]
    fn age_is_not_reset_when_a_delayed_sample_is_consumed() {
        let mut sample = ManualProvider::new(85.0).sample();
        let measured = sample.measured_at;
        sample.ram_percent = Some(25.0);
        sample.used_memory_bytes = 25;
        sample.total_memory_bytes = 100;
        sample.machine = MachineSignals::demo(85.0);
        let snapshot = MetricsSnapshot::at(
            sample,
            measured + Duration::from_secs(8),
            Duration::from_secs(3),
        );
        assert_eq!(snapshot.freshness, SampleFreshness::Stale);
        assert_eq!(snapshot.age, Duration::from_secs(8));
        assert_eq!(snapshot.sample.measured_at, measured);
        assert_eq!(snapshot.sample.cpu_percent, None);
        assert_eq!(snapshot.sample.memory_bytes(), None);
        assert_eq!(snapshot.sample.machine, MachineSignals::default());
    }

    #[test]
    fn manual_machine_signals_validate_independently() {
        let mut manual = ManualProvider::new(15.0);
        manual.set_machine(MachineSignals {
            disk_bytes_per_second: Some(f32::NAN),
            network_bytes_per_second: Some(42.0),
            battery_percent: Some(255.0),
            input_active: Some(false),
            ..Default::default()
        });
        let sample = manual.sample().validated();
        assert_eq!(sample.cpu_percent, Some(15.0));
        assert_eq!(sample.machine.disk_bytes_per_second, None);
        assert_eq!(sample.machine.network_bytes_per_second, Some(42.0));
        assert_eq!(sample.machine.battery_percent, None);
        assert_eq!(sample.machine.input_active, Some(false));
    }
}
