//! Independent tests compiled from the actual domain modules, with std only.
//!
//! `rustc --test --edition 2024 examples/journey_core_tests.rs -o journey-core-tests`
//! `./journey-core-tests`

// The route reads its authored content from the catalogue.
#[path = "../src/simulation/catalog.rs"]
pub mod catalog;
#[path = "../src/simulation/journey.rs"]
pub mod journey;
#[path = "../src/simulation/route.rs"]
pub mod route;
#[path = "../src/simulation/timing.rs"]
pub mod timing;

fn main() {
    println!(
        "Compile os testes autônomos: rustc --test --edition 2024 examples/journey_core_tests.rs -o journey-core-tests"
    );
}

#[cfg(test)]
mod tests {
    use super::{
        journey::{Achievement, Journey, MAX_TRAVEL_EVENTS, TravelEvent},
        route::{Route, StationId},
        timing::AdvanceOutcome,
    };

    fn move_units(journey: &mut Journey, mut remaining: f64) {
        while remaining > 0.0 {
            let distance = remaining.min(100.0);
            assert!(matches!(
                journey.advance(1.0, distance),
                AdvanceOutcome::Advanced { .. }
            ));
            remaining -= distance;
        }
    }

    fn event_distance(event: TravelEvent) -> f64 {
        match event {
            TravelEvent::StationArrived { distance_units, .. }
            | TravelEvent::StationDeparted { distance_units, .. }
            | TravelEvent::CircuitCompleted { distance_units, .. }
            | TravelEvent::AchievementUnlocked { distance_units, .. } => distance_units,
        }
    }

    #[test]
    fn same_seed_repeats_route_progress_and_events_without_external_dependencies() {
        let mut first = Journey::from_seed(42);
        let mut repeated = Journey::from_seed(42);
        assert_eq!(first.route(), repeated.route());
        assert_ne!(
            Route::from_seed(0).segments(),
            Route::from_seed(1).segments()
        );
        for step in 0..400 {
            let distance = if step % 7 == 0 { 0.0 } else { 37.5 };
            first.advance(0.25, distance);
            repeated.advance(0.25, distance);
        }
        assert_eq!(first.snapshot(), repeated.snapshot());
        assert_eq!(
            first.drain_events().collect::<Vec<_>>(),
            repeated.drain_events().collect::<Vec<_>>()
        );
        assert!(first.snapshot().achievements.first_circuit);
    }

    #[test]
    fn exact_arrival_and_next_movement_emit_separate_visual_cues() {
        let mut journey = Journey::default();
        let first_leg = journey.route().segments()[0].length_units;
        move_units(&mut journey, first_leg);
        assert_eq!(journey.snapshot().at_station, Some(StationId::Pines));
        assert_eq!(journey.snapshot().total_distance_units, first_leg);
        assert_eq!(
            journey.drain_events().collect::<Vec<_>>(),
            vec![
                TravelEvent::StationDeparted {
                    station: StationId::Aurora,
                    distance_units: 0.0,
                },
                TravelEvent::StationArrived {
                    station: StationId::Pines,
                    visit: 1,
                    distance_units: first_leg,
                },
                TravelEvent::AchievementUnlocked {
                    achievement: Achievement::FirstStation,
                    distance_units: first_leg,
                },
            ]
        );

        journey.advance(0.25, 0.0);
        assert_eq!(journey.snapshot().at_station, Some(StationId::Pines));
        assert_eq!(journey.pending_event_count(), 0);
        journey.advance(0.25, 1.0);
        assert_eq!(journey.snapshot().at_station, None);
        assert_eq!(
            journey.pop_event(),
            Some(TravelEvent::StationDeparted {
                station: StationId::Pines,
                distance_units: first_leg,
            })
        );
        assert_eq!(journey.pop_event(), None);
    }

    #[test]
    fn unconsumed_notifications_stay_bounded_and_do_not_erase_achievements() {
        let mut journey = Journey::default();
        for _ in 0..4_096 {
            journey.advance(1.0, 275.0);
            assert!(journey.pending_event_count() <= MAX_TRAVEL_EVENTS);
        }
        let final_state = journey.snapshot();
        assert_eq!(final_state.pending_events, MAX_TRAVEL_EVENTS);
        assert!(final_state.dropped_events > 0);
        assert!(final_state.discovered_stations.iter().all(|value| *value));
        assert!(final_state.achievements.long_journey);
        let events: Vec<_> = journey.drain_events().collect();
        assert_eq!(events.len(), MAX_TRAVEL_EVENTS);
        assert!(event_distance(events[0]) > 0.0);
        assert!(
            events
                .windows(2)
                .all(|pair| { event_distance(pair[0]) <= event_distance(pair[1]) })
        );
        assert!(event_distance(*events.last().unwrap()) <= final_state.total_distance_units);
        let drained = journey.snapshot();
        assert_eq!(drained.pending_events, 0);
        assert_eq!(drained.achievements, final_state.achievements);
        assert_eq!(drained.discovered_stations, final_state.discovered_stations);
        assert_eq!(drained.dropped_events, final_state.dropped_events);
    }

    #[test]
    fn pauses_sleep_and_invalid_steps_leave_progress_and_event_history_unchanged() {
        let mut journey = Journey::default();
        journey.advance(0.25, 10.0);
        let active = journey.snapshot();
        journey.set_paused(true);
        let paused = journey.snapshot();
        for (seconds, distance) in [(0.25, 10.0), (3_600.0, 200_000.0), (f64::NAN, f64::NAN)] {
            assert_eq!(journey.advance(seconds, distance), AdvanceOutcome::Paused);
            assert_eq!(journey.snapshot(), paused);
        }
        journey.set_paused(false);
        assert_eq!(journey.snapshot(), active);
        for (seconds, distance, expected) in [
            (3_600.0, 200_000.0, AdvanceOutcome::Suspended),
            (f64::NAN, 0.0, AdvanceOutcome::InvalidDelta),
            (f64::INFINITY, 0.0, AdvanceOutcome::InvalidDelta),
            (-1.0, 0.0, AdvanceOutcome::InvalidDelta),
            (0.0, f64::MAX, AdvanceOutcome::NoElapsedTime),
            (1.0, 276.0, AdvanceOutcome::InvalidDistance),
            (0.25, f64::NAN, AdvanceOutcome::InvalidDistance),
            (0.25, -1.0, AdvanceOutcome::InvalidDistance),
        ] {
            assert_eq!(journey.advance(seconds, distance), expected);
            assert_eq!(journey.snapshot(), active);
        }
        journey.advance(0.25, 10.0);
        assert_eq!(journey.snapshot().total_distance_units, 20.0);
        assert_eq!(journey.snapshot().active_seconds, 0.5);
    }

    #[test]
    fn equal_active_travel_has_equal_progress_at_different_frame_rates() {
        fn simulate(fps: u32) -> Journey {
            let mut journey = Journey::from_seed(0);
            for _ in 0..fps * 600 {
                journey.advance(1.0 / f64::from(fps), 55.0 / f64::from(fps));
            }
            journey
        }
        let reference = simulate(1).snapshot();
        for fps in [15, 30, 60, 144] {
            let state = simulate(fps).snapshot();
            assert!((state.total_distance_units - 33_000.0).abs() < 1e-7);
            assert!((state.active_seconds - 600.0).abs() < 1e-7);
            assert!((state.route_position_units - reference.route_position_units).abs() < 1e-7);
            assert_eq!(state.route_segment, reference.route_segment);
            assert_eq!(state.completed_laps, reference.completed_laps);
            assert_eq!(state.stations_visited, reference.stations_visited);
            assert_eq!(state.discovered_stations, reference.discovered_stations);
            assert_eq!(state.achievements, reference.achievements);
        }
    }
}
