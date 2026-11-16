//! Bounded, ephemeral machine signals. No OS access, persistence or service mutations.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MachineSignals {
    pub cpu_percent: Option<f32>,
    pub memory_percent: Option<f32>,
    pub disk_bytes_per_second: Option<f32>,
    pub network_bytes_per_second: Option<f32>,
    pub input_active: Option<bool>,
    pub input_idle_seconds: Option<u32>,
    pub on_ac_power: Option<bool>,
    pub battery_percent: Option<f32>,
}

impl MachineSignals {
    pub fn validated(mut self) -> Self {
        for value in [
            &mut self.cpu_percent,
            &mut self.memory_percent,
            &mut self.battery_percent,
        ] {
            *value = value.filter(|v| v.is_finite() && (0.0..=100.0).contains(v));
        }
        for value in [
            &mut self.disk_bytes_per_second,
            &mut self.network_bytes_per_second,
        ] {
            *value = value.filter(|v| v.is_finite() && *v >= 0.0);
        }
        self
    }

    pub fn without_input(mut self) -> Self {
        self.input_active = None;
        self.input_idle_seconds = None;
        self
    }

    pub fn demo(cpu: f32) -> Self {
        Self {
            cpu_percent: Some(cpu),
            memory_percent: Some(54.0),
            disk_bytes_per_second: Some(3_500_000.0),
            network_bytes_per_second: Some(80_000.0),
            input_active: Some(true),
            input_idle_seconds: Some(0),
            on_ac_power: Some(true),
            battery_percent: None,
        }
        .validated()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MachineEffects {
    /// Normalized gauges: boiler, stoker, platform, crane, telegraph, tender.
    pub gauges: [Option<f32>; 6],
    pub coin_parts: [f32; 3],
    pub quiet_march: bool,
    pub low_power: bool,
    pub on_ac_power: Option<bool>,
}

impl MachineEffects {
    pub fn multiplier(self) -> f32 {
        1.0 + self.coin_parts.iter().sum::<f32>().clamp(0.0, 0.30)
    }
}

#[derive(Clone, Debug)]
pub struct MachineState {
    samples: [(f64, MachineSignals); 60],
    next: usize,
    latest: Option<f64>,
    next_telegram: Option<f64>,
}

impl Default for MachineState {
    fn default() -> Self {
        Self {
            samples: [(-1.0, MachineSignals::default()); 60],
            next: 0,
            latest: None,
            next_telegram: None,
        }
    }
}

impl MachineState {
    /// At most one observation per second; a replay cannot fill the minute.
    pub fn observe(&mut self, seconds: f64, signals: MachineSignals) {
        if !seconds.is_finite() || seconds < 0.0 || self.latest.is_some_and(|t| seconds - t < 1.0) {
            return;
        }
        self.samples[self.next] = (seconds, signals.validated());
        self.next = (self.next + 1) % self.samples.len();
        self.latest = Some(seconds);
    }

    pub fn clear_input(&mut self) {
        for (_, sample) in &mut self.samples {
            *sample = sample.without_input();
        }
    }

    pub fn effects(&self, seconds: f64, max_age: f64, input_enabled: bool) -> MachineEffects {
        let Some(latest) = self.latest else {
            return MachineEffects::default();
        };
        if !seconds.is_finite() || seconds < latest || seconds - latest > max_age {
            return MachineEffects::default();
        }
        let current = self.samples[(self.next + 59) % 60].1;
        let mean = |get: fn(MachineSignals) -> Option<f32>| -> Option<f32> {
            get(current)?;
            let mut sum = 0.0;
            let mut count = 0;
            for (at, sample) in self.samples {
                if at >= 0.0
                    && seconds - at < 60.0
                    && seconds >= at
                    && let Some(value) = get(sample)
                {
                    sum += value;
                    count += 1;
                }
            }
            (count > 0).then(|| sum / count as f32)
        };
        // Saturate each observation before smoothing: overload cannot bank a bonus.
        let cpu = mean(|s| s.cpu_percent.map(|v| v.min(15.0))).map(|v| v / 15.0);
        let memory = mean(|s| s.memory_percent.map(|v| v.min(60.0))).map(|v| v / 60.0);
        let disk =
            mean(|s| s.disk_bytes_per_second.map(|v| v.min(5_000_000.0))).map(|v| v / 5_000_000.0);
        let network = mean(|s| s.network_bytes_per_second.map(|v| v.min(200_000.0)));
        let input = if input_enabled && current.input_active.is_some() {
            Some(
                (self
                    .samples
                    .iter()
                    .filter(|(at, s)| {
                        *at >= 0.0
                            && seconds >= *at
                            && seconds - *at < 60.0
                            && s.input_active == Some(true)
                    })
                    .count() as f32
                    / 20.0)
                    .min(1.0),
            )
        } else {
            None
        };
        MachineEffects {
            gauges: [
                cpu,
                input,
                memory,
                disk,
                network.map(|v| (v / 200_000.0).min(1.0)),
                if current.on_ac_power == Some(true) {
                    Some(1.0)
                } else {
                    current.battery_percent.map(|v| v / 100.0)
                },
            ],
            coin_parts: [
                cpu.unwrap_or(0.0) * 0.10,
                input.unwrap_or(0.0) * 0.15,
                if network.is_some_and(|v| v > 0.0) {
                    0.05
                } else {
                    0.0
                },
            ],
            quiet_march: input_enabled && current.input_idle_seconds.is_some_and(|v| v >= 300),
            low_power: current.on_ac_power == Some(false)
                && current.battery_percent.is_some_and(|v| v < 20.0),
            on_ac_power: current.on_ac_power,
        }
    }

    /// Occasional request by active journey time, never wall-clock/uptime.
    /// The World remains responsible for arrival, shared cooldown and fourth slot.
    pub fn take_telegram(&mut self, active_seconds: f64, effects: MachineEffects) -> bool {
        if !active_seconds.is_finite() || active_seconds < 0.0 {
            return false;
        }
        let next = *self.next_telegram.get_or_insert(active_seconds + 1800.0);
        if active_seconds >= next && effects.coin_parts[2] > 0.0 {
            self.next_telegram = Some(active_seconds + 1800.0);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saturation_replay_expiry_and_opt_out() {
        let mut state = MachineState::default();
        for second in 0..60 {
            state.observe(second as f64, MachineSignals::demo(100.0));
        }
        let full = state.effects(59.0, 3.0, true);
        assert!((full.multiplier() - 1.30).abs() < 0.00001);
        assert_eq!(full.gauges[1], Some(1.0));
        assert_eq!(state.effects(63.0, 3.0, true).multiplier(), 1.0);
        assert_eq!(state.effects(58.0, 3.0, true).multiplier(), 1.0);
        state.clear_input();
        assert!(state.effects(59.0, 3.0, true).gauges[1].is_none());
        let mut one = MachineState::default();
        for _ in 0..60 {
            one.observe(0.0, MachineSignals::demo(15.0));
        }
        assert_eq!(one.effects(0.0, 3.0, true).gauges[1], Some(0.05));
    }
    #[test]
    fn partial_invalid_and_power_never_reduce_base() {
        let mut state = MachineState::default();
        state.observe(
            0.0,
            MachineSignals {
                cpu_percent: Some(f32::NAN),
                memory_percent: Some(30.0),
                disk_bytes_per_second: Some(-1.0),
                on_ac_power: Some(false),
                battery_percent: Some(19.0),
                ..Default::default()
            },
        );
        let effects = state.effects(0.0, 3.0, true);
        assert!(!state.take_telegram(0.0, effects));
        assert_eq!(effects.multiplier(), 1.0);
        assert_eq!(effects.gauges[2], Some(0.5));
        assert_eq!(effects.gauges[3], None);
        assert!(effects.low_power);
        state.observe(1.0, MachineSignals::default());
        assert_eq!(state.effects(1.0, 3.0, true), MachineEffects::default());
    }
    #[test]
    fn telegram_uses_active_time_and_no_backlog() {
        let mut state = MachineState::default();
        state.observe(0.0, MachineSignals::demo(15.0));
        let effects = state.effects(0.0, 3.0, true);
        assert!(!state.take_telegram(0.0, effects));
        assert!(!state.take_telegram(1799.0, effects));
        assert!(state.take_telegram(1800.0, effects));
        assert!(!state.take_telegram(1800.0, effects));
        assert!(!state.take_telegram(4000.0, MachineEffects::default()));
        assert!(state.take_telegram(4000.0, effects));
        assert!(!state.take_telegram(4000.0, effects));
    }

    #[test]
    fn overload_cannot_store_extra_bonus_in_the_average() {
        let mut normal = MachineState::default();
        let mut overloaded = MachineState::default();
        for second in 0..60 {
            let working = second < 10;
            normal.observe(
                second as f64,
                MachineSignals::demo(if working { 15.0 } else { 0.0 }),
            );
            overloaded.observe(
                second as f64,
                MachineSignals::demo(if working { 100.0 } else { 0.0 }),
            );
        }
        assert_eq!(
            normal.effects(59.0, 3.0, true),
            overloaded.effects(59.0, 3.0, true)
        );
        let mut reopened = MachineState::default();
        assert!(!reopened.take_telegram(9000.0, normal.effects(59.0, 3.0, true)));
    }
}
