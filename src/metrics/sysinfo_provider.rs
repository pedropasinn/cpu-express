use std::time::{Duration, Instant};

use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, MemoryRefreshKind, System};

use super::{MetricStatus, MetricsProvider, MetricsSample};

/// One reusable sysinfo instance preserves the CPU baseline between samples.
///
/// `sysinfo`'s refresh methods do not return a success flag. We reject missing
/// counters and invalid exposed values, but cannot detect a platform backend
/// silently retaining or substituting a plausible number after an OS error.
pub struct SysinfoProvider {
    system: System,
    #[cfg(windows)]
    machine: super::machine_windows::MachineCollector,
    previous: Option<LastRefresh>,
}

struct LastRefresh {
    sample: MetricsSample,
    completed_at: Instant,
    has_cpu_baseline: bool,
}

impl LastRefresh {
    fn cached_if_too_early(&self, now: Instant) -> Option<MetricsSample> {
        (now.saturating_duration_since(self.completed_at)
            < MINIMUM_CPU_UPDATE_INTERVAL.max(Duration::from_secs(1)))
        .then(|| self.sample.clone())
    }
}

impl SysinfoProvider {
    pub fn new() -> Result<Self, String> {
        Self::with_input(std::sync::Arc::new(std::sync::atomic::AtomicBool::new(
            true,
        )))
    }

    pub(super) fn with_input(
        _input: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<Self, String> {
        let system = System::new().map_err(|error| error.to_string())?;
        Ok(Self {
            system,
            #[cfg(windows)]
            machine: super::machine_windows::MachineCollector::new(_input),
            previous: None,
        })
    }
}

impl MetricsProvider for SysinfoProvider {
    fn sample(&mut self) -> MetricsSample {
        let measured_at = Instant::now();
        if let Some(sample) = self
            .previous
            .as_ref()
            .and_then(|previous| previous.cached_if_too_early(measured_at))
        {
            // Keep the original timestamp, CPU and RAM together. A direct
            // caller polling too quickly must not manufacture fresh readings
            // or send a ready provider back into warming-up.
            return sample;
        }

        let had_baseline = self
            .previous
            .as_ref()
            .is_some_and(|previous| previous.has_cpu_baseline);
        self.system.refresh_cpu_usage();
        self.system
            .refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());

        let counters = Counters {
            has_cpu: !self.system.cpus().is_empty(),
            cpu: self.system.global_cpu_usage(),
            used_memory: self.system.used_memory(),
            total_memory: self.system.total_memory(),
            available_memory: self.system.available_memory(),
        };
        #[allow(unused_mut)]
        let mut sample = counters.to_sample(had_baseline, measured_at);
        #[cfg(windows)]
        {
            sample.machine = self.machine.sample();
        }
        self.previous = Some(LastRefresh {
            sample: sample.clone(),
            completed_at: Instant::now(),
            // After an observable CPU error, establish another baseline before
            // reporting a percentage again. RAM failure does not reset CPU.
            has_cpu_baseline: counters.cpu_error().is_none(),
        });
        sample
    }

    fn minimum_interval(&self) -> Duration {
        MINIMUM_CPU_UPDATE_INTERVAL.max(Duration::from_secs(1))
    }
}

/// Values at the boundary of sysinfo. Kept separate for deterministic checks
/// that do not depend on installed Windows counters or load on the test host.
struct Counters {
    has_cpu: bool,
    cpu: f32,
    used_memory: u64,
    total_memory: u64,
    available_memory: u64,
}

impl Counters {
    fn cpu_error(&self) -> Option<&'static str> {
        if !self.has_cpu {
            Some("CPU counters are unavailable")
        } else if !self.cpu.is_finite() || !(0.0..=100.0).contains(&self.cpu) {
            Some("The operating system returned an invalid CPU value")
        } else {
            None
        }
    }

    fn memory_error(&self) -> Option<&'static str> {
        if self.total_memory == 0 {
            Some("Physical memory information is unavailable")
        } else if self.used_memory > self.total_memory || self.available_memory > self.total_memory
        {
            Some("The operating system returned inconsistent physical memory values")
        } else {
            None
        }
    }

    fn to_sample(&self, had_baseline: bool, measured_at: Instant) -> MetricsSample {
        let cpu_error = self.cpu_error();
        let memory_error = self.memory_error();
        let cpu_percent = (had_baseline && cpu_error.is_none()).then_some(self.cpu);
        let (ram_percent, used_memory_bytes, total_memory_bytes) = if memory_error.is_none() {
            (
                Some((100.0 * self.used_memory as f64 / self.total_memory as f64) as f32),
                self.used_memory,
                self.total_memory,
            )
        } else {
            // These zeroes are the legacy API's absence sentinels, not a
            // measured 0-byte system. Consumers must check ram_percent first.
            (None, 0, 0)
        };
        let status = match (cpu_error, memory_error) {
            (Some(cpu), Some(memory)) => MetricStatus::Unavailable(format!("{cpu}; {memory}")),
            (Some(error), None) | (None, Some(error)) => {
                MetricStatus::Unavailable(error.to_owned())
            }
            (None, None) if !had_baseline => MetricStatus::WarmingUp,
            (None, None) => MetricStatus::Ready,
        };
        MetricsSample {
            machine: Default::default(),
            cpu_percent,
            ram_percent,
            used_memory_bytes,
            total_memory_bytes,
            // Timestamp the beginning of collection, so a slow OS call cannot
            // return an older observation disguised as a newly measured one.
            measured_at,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_counters(cpu: f32) -> Counters {
        Counters {
            has_cpu: true,
            cpu,
            used_memory: 512,
            total_memory: 1024,
            available_memory: 512,
        }
    }

    #[test]
    fn first_refresh_warms_up_without_inventing_cpu() {
        let sample = valid_counters(84.0).to_sample(false, Instant::now());
        assert_eq!(sample.cpu_percent, None);
        assert_eq!(sample.ram_percent, Some(50.0));
        assert_eq!(sample.status, MetricStatus::WarmingUp);
    }

    #[test]
    fn legitimate_zero_cpu_is_ready_after_a_baseline() {
        let sample = valid_counters(0.0).to_sample(true, Instant::now());
        assert_eq!(sample.cpu_percent, Some(0.0));
        assert_eq!(sample.status, MetricStatus::Ready);
    }

    #[test]
    fn invalid_cpu_is_unavailable_and_preserves_valid_ram() {
        for cpu in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, 101.0] {
            let counters = valid_counters(cpu);
            let sample = counters.to_sample(true, Instant::now());
            assert_eq!(sample.cpu_percent, None, "invalid input: {cpu:?}");
            assert_eq!(sample.ram_percent, Some(50.0));
            assert_eq!(
                (sample.used_memory_bytes, sample.total_memory_bytes),
                (512, 1024)
            );
            assert!(matches!(sample.status, MetricStatus::Unavailable(_)));
            assert!(counters.cpu_error().is_some(), "must reset CPU baseline");
        }
    }

    #[test]
    fn missing_cpu_counters_cannot_be_reported_as_zero_usage() {
        let counters = Counters {
            has_cpu: false,
            ..valid_counters(0.0)
        };
        let sample = counters.to_sample(true, Instant::now());
        assert_eq!(sample.cpu_percent, None);
        assert!(matches!(sample.status, MetricStatus::Unavailable(_)));
        assert!(counters.cpu_error().is_some());
    }

    #[test]
    fn invalid_memory_uses_absence_sentinels_and_preserves_valid_cpu() {
        for (used_memory, total_memory, available_memory) in
            [(123, 0, 0), (101, 100, 0), (0, 100, 101)]
        {
            let counters = Counters {
                used_memory,
                total_memory,
                available_memory,
                ..valid_counters(75.0)
            };
            let sample = counters.to_sample(true, Instant::now());
            assert_eq!(sample.cpu_percent, Some(75.0));
            assert_eq!(sample.ram_percent, None);
            assert_eq!(
                (sample.used_memory_bytes, sample.total_memory_bytes),
                (0, 0)
            );
            assert!(matches!(sample.status, MetricStatus::Unavailable(_)));
            assert!(
                counters.cpu_error().is_none(),
                "valid CPU keeps its baseline"
            );
        }
    }

    #[test]
    fn both_counter_errors_are_explained() {
        let counters = Counters {
            has_cpu: false,
            total_memory: 0,
            ..valid_counters(0.0)
        };
        let sample = counters.to_sample(true, Instant::now());
        let MetricStatus::Unavailable(message) = sample.status else {
            panic!("both metrics should be unavailable");
        };
        assert!(message.contains("CPU counters"));
        assert!(message.contains("Physical memory"));
    }

    #[test]
    fn fast_repeated_call_keeps_value_status_and_original_timestamp() {
        let measured_at = Instant::now();
        for had_baseline in [false, true] {
            let previous = LastRefresh {
                sample: valid_counters(37.0).to_sample(had_baseline, measured_at),
                completed_at: measured_at + Duration::from_millis(4),
                has_cpu_baseline: true,
            };
            let repeat = previous
                .cached_if_too_early(previous.completed_at)
                .expect("sample should be reused until the CPU update interval");
            assert_eq!(repeat.measured_at, measured_at);
            assert_eq!(repeat.status, previous.sample.status);
            assert_eq!(repeat.cpu_percent, previous.sample.cpu_percent);
            assert_eq!(repeat.ram_percent, previous.sample.ram_percent);
            assert!(
                previous
                    .cached_if_too_early(
                        previous.completed_at
                            + MINIMUM_CPU_UPDATE_INTERVAL.max(Duration::from_secs(1))
                    )
                    .is_none()
            );
        }
    }
}
