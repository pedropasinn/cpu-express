use std::f64::consts::TAU;

use crate::config::Config;

/// Visual scale only: the displayed km/h are fictional game units.
pub const PIXELS_PER_SECOND_PER_KMH: f64 = 0.55;
/// Matches the large driving wheels in the original 640 × 140 scene.
pub const WHEEL_RADIUS_PX: f64 = 6.0;

/// The UI owns a train and lends it immutably to the renderer after each step.
#[derive(Clone, Debug)]
pub struct Train {
    speed_kmh: f64,
    target_speed_kmh: f64,
    wheel_phase: f64,
}

impl Train {
    pub fn stop_at_station(&mut self) {
        self.speed_kmh = 0.0;
        self.target_speed_kmh = 0.0;
    }
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let config = config.clone().normalized();
        Self {
            speed_kmh: config.min_speed_kmh,
            target_speed_kmh: config.min_speed_kmh,
            wheel_phase: 0.0,
        }
    }

    /// Advance using the exact solution of `dv/dt = k * (target - v)`.
    ///
    /// The target is held constant over this step. Both the final speed *and*
    /// its time integral are analytical, so splitting a second into 30 or 60
    /// frames gives the same distance and wheel rotation within float precision.
    /// Invalid CPU input falls back to the minimum target. Invalid or negative
    /// time causes no movement. Gaps above 1 s are discarded as suspension;
    /// split intentional long simulations into steps of at most 1 s. The
    /// returned distance is in scene pixels. As before, even a zero-time call
    /// updates the target and normalizes the speed against the current config.
    pub fn update(&mut self, cpu_percent: f32, dt_seconds: f64, config: &Config) -> f64 {
        let config = config.clone().normalized();
        let cpu = if cpu_percent.is_finite() {
            f64::from(cpu_percent.clamp(0.0, 100.0)) / 100.0
        } else {
            0.0
        };
        self.target_speed_kmh =
            config.min_speed_kmh + (config.max_speed_kmh - config.min_speed_kmh) * cpu;
        self.speed_kmh = self
            .speed_kmh
            .clamp(config.min_speed_kmh, config.max_speed_kmh);

        let dt = super::valid_delta(dt_seconds);
        if dt == 0.0 {
            return 0.0;
        }

        let previous_speed = self.speed_kmh;
        // exp_m1 avoids loss of precision for very short frames.
        let alpha = -(-config.smoothing_k * dt).exp_m1();
        self.speed_kmh = (previous_speed + (self.target_speed_kmh - previous_speed) * alpha)
            .clamp(config.min_speed_kmh, config.max_speed_kmh);

        let integrated_speed = self.target_speed_kmh * dt
            + (previous_speed - self.target_speed_kmh) * alpha / config.smoothing_k;
        let distance = integrated_speed.max(0.0) * PIXELS_PER_SECOND_PER_KMH;
        // Defensive finite result even if the configuration bounds later change.
        let distance = if distance.is_finite() {
            distance
        } else {
            f64::MAX
        };

        let circumference = TAU * WHEEL_RADIUS_PX;
        // Reduce before multiplication, avoiding overflow for exceptional deltas.
        let rotation = distance.rem_euclid(circumference) / WHEEL_RADIUS_PX;
        self.wheel_phase = (self.wheel_phase + rotation).rem_euclid(TAU);
        distance
    }

    #[must_use]
    pub fn speed_kmh(&self) -> f64 {
        self.speed_kmh
    }

    #[must_use]
    pub fn target_speed_kmh(&self) -> f64 {
        self.target_speed_kmh
    }

    /// Driving-wheel angle in radians, always in `[0, 2π)`.
    #[must_use]
    pub fn wheel_phase(&self) -> f64 {
        self.wheel_phase
    }
}
