//! Phase-locked presentation deadlines; missed frames are skipped, never replayed.
use std::time::{Duration, Instant};

pub(crate) fn next_deadline(previous: Instant, now: Instant, interval: Duration) -> Instant {
    let interval = interval.max(Duration::from_micros(1));
    if previous > now {
        // Only a repaint inside the vsync tolerance consumes the deadline.
        // An explicit appearance/layout edit must not push future animation back.
        return if previous.duration_since(now) <= Duration::from_millis(1) {
            previous + interval
        } else {
            previous
        };
    }
    let remainder = now.duration_since(previous).as_nanos() % interval.as_nanos();
    now + interval - Duration::from_nanos(remainder as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn irregular_delivery_preserves_the_original_cadence() {
        let origin = Instant::now();
        let period = Duration::from_millis(20);
        let mut deadline = origin;
        for millis in [0, 23, 41, 65, 81] {
            deadline = next_deadline(deadline, origin + Duration::from_millis(millis), period);
        }
        assert_eq!(deadline, origin + Duration::from_millis(100));
    }
    #[test]
    fn long_stalls_skip_frames_without_scheduling_a_catch_up_burst() {
        let origin = Instant::now();
        let now = origin + Duration::from_millis(2007);
        assert_eq!(
            next_deadline(origin, now, Duration::from_millis(20)),
            origin + Duration::from_millis(2020)
        );
    }
    #[test]
    fn early_frame_consumes_one_deadline() {
        let now = Instant::now();
        let deadline = now + Duration::from_micros(500);
        assert_eq!(
            next_deadline(deadline, now, Duration::from_millis(20)),
            deadline + Duration::from_millis(20)
        );
    }
    #[test]
    fn repeated_visual_edits_do_not_postpone_the_scheduled_frame() {
        let origin = Instant::now();
        let interval = Duration::from_millis(20);
        let scheduled = origin + interval;
        let mut deadline = scheduled;
        for millis in [3, 5, 8, 12] {
            deadline = next_deadline(deadline, origin + Duration::from_millis(millis), interval);
            assert_eq!(deadline, scheduled);
        }
        assert_eq!(
            next_deadline(deadline, scheduled, interval),
            origin + interval * 2
        );
    }
}
