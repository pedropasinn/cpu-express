//! Rules for active simulation time. No wall clock, I/O, or catch-up loop.

/// A larger gap is treated as a suspension and discarded in its entirety.
/// Simulated long sessions must be divided into active steps of at most 1 s.
pub const MAX_ACTIVE_DELTA_SECONDS: f64 = 1.0;

/// Fictional journey units per scenery pixel, not a geographical conversion.
pub const TRAVEL_UNITS_PER_SCENE_PIXEL: f64 = 1.0;

/// Safety envelope: the existing normalized config permits at most 500
/// fictional km/h, and the train maps each of those to 0.55 pixels/second.
/// This is an input validation bound, never a target or a CPU workload.
pub const MAX_TRAVEL_UNITS_PER_SECOND: f64 = 275.0;

/// An ignored step changes neither distance, active time, nor event history.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AdvanceOutcome {
    Advanced { seconds: f64, distance_units: f64 },
    Paused,
    NoElapsedTime,
    Suspended,
    InvalidDelta,
    InvalidDistance,
}

pub(super) fn active_delta(seconds: f64) -> Result<f64, AdvanceOutcome> {
    if !seconds.is_finite() || seconds < 0.0 {
        Err(AdvanceOutcome::InvalidDelta)
    } else if seconds == 0.0 {
        Err(AdvanceOutcome::NoElapsedTime)
    } else if seconds > MAX_ACTIVE_DELTA_SECONDS {
        Err(AdvanceOutcome::Suspended)
    } else {
        Ok(seconds)
    }
}

pub(super) fn active_distance(seconds: f64, distance: f64) -> Result<f64, AdvanceOutcome> {
    let maximum = MAX_TRAVEL_UNITS_PER_SECOND * seconds;
    if !distance.is_finite() || distance < 0.0 {
        return Err(AdvanceOutcome::InvalidDistance);
    }
    if distance <= maximum {
        return Ok(distance);
    }
    // Absorb only round-off at the legitimate train's analytical upper bound.
    // Relative tolerance also works for very small positive time steps.
    if distance - maximum <= maximum * (32.0 * f64::EPSILON) {
        Ok(maximum)
    } else {
        Err(AdvanceOutcome::InvalidDistance)
    }
}
