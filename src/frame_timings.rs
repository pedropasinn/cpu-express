//! Bounded measurements of actual simulation/render/upload preparation work.
use std::time::Duration;
#[derive(Clone)]
pub struct FrameTimings {
    samples: [f64; 256],
    cursor: usize,
    len: usize,
}
impl Default for FrameTimings {
    fn default() -> Self {
        Self {
            samples: [0.0; 256],
            cursor: 0,
            len: 0,
        }
    }
}
impl FrameTimings {
    pub fn record(&mut self, duration: Duration) {
        self.samples[self.cursor] = duration.as_secs_f64() * 1000.0;
        self.cursor = (self.cursor + 1) % 256;
        self.len = (self.len + 1).min(256);
    }
    pub fn diagnostics(&self) -> serde_json::Value {
        if self.len == 0 {
            return serde_json::json!({"samples":0});
        }
        let mut samples = self.samples;
        samples[..self.len].sort_by(f64::total_cmp);
        serde_json::json!({"samples":self.len,"mean_ms":samples[..self.len].iter().sum::<f64>()/self.len as f64,"p95_ms":samples[((self.len*95).div_ceil(100)).saturating_sub(1)],"max_ms":samples[self.len-1],"scope":"simulation_render_texture_submission_not_compositor"})
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_window_evicts_ancient_spike() {
        let mut timings = FrameTimings::default();
        timings.record(Duration::from_secs(1));
        for _ in 0..256 {
            timings.record(Duration::from_millis(2));
        }
        assert_eq!(timings.diagnostics()["samples"], 256);
        assert_eq!(timings.diagnostics()["max_ms"], 2.0);
    }
}
