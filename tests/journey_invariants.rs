//! Behavioural checks for travel, independent of the renderer and real CPU data.
//! Simulated time is advanced directly; these are not machine-load benchmarks.

use cpu_express::{
    config::Config,
    simulation::{
        Achievement, AdvanceOutcome, Journey, JourneySnapshot, LONG_JOURNEY_DISTANCE_UNITS,
        MAX_ACTIVE_DELTA_SECONDS, MAX_TRAVEL_EVENTS, MAX_TRAVEL_UNITS_PER_SECOND, StationId, Train,
        TravelEvent, World,
    },
};

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual={actual:.15}, expected={expected:.15}, tolerance={tolerance:e}"
    );
}

fn advance_distance(journey: &mut Journey, mut remaining: f64) {
    while remaining > 0.0 {
        let distance = remaining.min(100.0);
        assert!(matches!(
            journey.advance(1.0, distance),
            AdvanceOutcome::Advanced { .. }
        ));
        remaining -= distance;
    }
}

fn event_position(event: &TravelEvent) -> f64 {
    match *event {
        TravelEvent::StationArrived { distance_units, .. }
        | TravelEvent::StationDeparted { distance_units, .. }
        | TravelEvent::CircuitCompleted { distance_units, .. }
        | TravelEvent::AchievementUnlocked { distance_units, .. } => distance_units,
    }
}

fn assert_same_event(actual: &TravelEvent, expected: &TravelEvent) {
    match (actual, expected) {
        (
            TravelEvent::StationArrived {
                station: actual_station,
                visit: actual_visit,
                ..
            },
            TravelEvent::StationArrived {
                station: expected_station,
                visit: expected_visit,
                ..
            },
        ) => {
            assert_eq!(actual_station, expected_station);
            assert_eq!(actual_visit, expected_visit);
        }
        (
            TravelEvent::StationDeparted {
                station: actual, ..
            },
            TravelEvent::StationDeparted {
                station: expected, ..
            },
        ) => assert_eq!(actual, expected),
        (
            TravelEvent::CircuitCompleted {
                completed_laps: actual,
                ..
            },
            TravelEvent::CircuitCompleted {
                completed_laps: expected,
                ..
            },
        ) => assert_eq!(actual, expected),
        (
            TravelEvent::AchievementUnlocked {
                achievement: actual,
                ..
            },
            TravelEvent::AchievementUnlocked {
                achievement: expected,
                ..
            },
        ) => assert_eq!(actual, expected),
        _ => panic!("different event kinds: {actual:?} versus {expected:?}"),
    }
    assert_close(event_position(actual), event_position(expected), 1e-6);
}

#[test]
fn frame_rate_preserves_travel_station_sequence_and_train_motion() {
    fn simulate(fps: u32) -> (JourneySnapshot, f64, f64, Vec<TravelEvent>) {
        let config = Config::default();
        let mut train = Train::new(&config);
        let mut world = World::with_seed(42);
        let mut events = Vec::new();
        let mut previous_distance = 0.0;
        for second in 0..720 {
            let cpu = match (second / 45) % 4 {
                0 => 0.0,
                1 => 50.0,
                2 => 100.0,
                _ => 20.0,
            };
            for _ in 0..fps {
                assert!(matches!(
                    world.advance_train(&mut train, cpu, 1.0 / f64::from(fps), &config),
                    AdvanceOutcome::Advanced { .. }
                ));
                let snapshot = world.journey_snapshot();
                assert!(snapshot.total_distance_units >= previous_distance);
                assert!(
                    (0.0..snapshot.route_length_units).contains(&snapshot.route_position_units)
                );
                assert!((0.0..=1.0).contains(&snapshot.segment_progress));
                previous_distance = snapshot.total_distance_units;
                events.extend(world.drain_travel_events());
            }
        }
        (
            world.journey_snapshot(),
            train.speed_kmh(),
            train.wheel_phase(),
            events,
        )
    }

    let reference = simulate(30);
    assert!(reference.0.completed_laps >= 1);
    assert_eq!(reference.0.dropped_events, 0);
    for fps in [15, 60, 144] {
        let actual = simulate(fps);
        assert_close(
            actual.0.total_distance_units,
            reference.0.total_distance_units,
            1e-6,
        );
        assert_close(
            actual.0.route_position_units,
            reference.0.route_position_units,
            1e-6,
        );
        assert_close(actual.0.active_seconds, 720.0, 1e-8);
        assert_eq!(actual.0.route_segment, reference.0.route_segment);
        assert_eq!(actual.0.stations_visited, reference.0.stations_visited);
        assert_eq!(actual.0.completed_laps, reference.0.completed_laps);
        assert_eq!(actual.0.achievements, reference.0.achievements);
        assert_close(actual.1, reference.1, 1e-10);
        assert_close(actual.2, reference.2, 1e-8);
        assert_eq!(actual.3.len(), reference.3.len());
        for (actual_event, expected_event) in actual.3.iter().zip(&reference.3) {
            assert_same_event(actual_event, expected_event);
        }
    }
}

#[test]
fn world_adapter_uses_the_existing_analytical_train_integral() {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    let expected_speed = 100.0 - 88.0 * (-1.8_f64).exp();
    let expected_distance = (100.0 - 88.0 * (1.0 - (-1.8_f64).exp()) / 1.8) * 0.55;
    assert!(matches!(
        world.advance_train(&mut train, 100.0, 1.0, &config),
        AdvanceOutcome::Advanced { .. }
    ));
    assert_close(train.speed_kmh(), expected_speed, 1e-12);
    assert_close(world.scroll_px(), expected_distance, 1e-12);
    assert_close(
        world.journey_snapshot().total_distance_units,
        expected_distance,
        1e-12,
    );

    let mut idle_train = Train::new(&config);
    let mut idle_world = World::default();
    idle_world.advance_train(&mut idle_train, 0.0, 1.0, &config);
    assert_eq!(idle_train.speed_kmh(), 12.0);
    assert_close(
        idle_world.journey_snapshot().total_distance_units,
        6.6,
        1e-12,
    );
}

#[test]
fn exact_station_boundary_arrives_once_and_waits_for_motion_to_depart() {
    let mut journey = Journey::from_seed(7);
    let first_leg = journey.route().segments()[0].length_units;
    advance_distance(&mut journey, first_leg - 1.0);
    journey.drain_events().for_each(drop);

    journey.advance(0.5, 1.0);
    let events: Vec<_> = journey.drain_events().collect();
    let arrivals: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            TravelEvent::StationArrived { station, visit, .. } => Some((*station, *visit)),
            _ => None,
        })
        .collect();
    assert_eq!(arrivals, [(StationId::Pines, 1)]);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, TravelEvent::StationDeparted { .. }))
    );
    assert_eq!(journey.snapshot().at_station, Some(StationId::Pines));
    assert_eq!(journey.snapshot().next_station.id, StationId::Ridge);
    assert!(
        journey
            .snapshot()
            .achievements
            .contains(Achievement::FirstStation)
    );

    journey.advance(0.5, 0.0);
    assert_eq!(journey.snapshot().at_station, Some(StationId::Pines));
    assert_eq!(journey.pop_event(), None);
    journey.advance(0.5, 2.0);
    assert_eq!(
        journey.pop_event(),
        Some(TravelEvent::StationDeparted {
            station: StationId::Pines,
            distance_units: first_leg,
        })
    );
    assert_eq!(journey.pop_event(), None);
    assert_eq!(journey.snapshot().at_station, None);
    assert_eq!(journey.snapshot().stations_visited, 1);
}

#[test]
fn crossing_a_station_with_remaining_distance_does_not_force_a_stop() {
    let mut journey = Journey::from_seed(7);
    let first_leg = journey.route().segments()[0].length_units;
    advance_distance(&mut journey, first_leg - 10.0);
    journey.drain_events().for_each(drop);
    journey.advance(1.0, 20.0);
    let transitions: Vec<_> = journey
        .drain_events()
        .filter(|event| {
            matches!(
                event,
                TravelEvent::StationArrived { .. } | TravelEvent::StationDeparted { .. }
            )
        })
        .collect();
    assert_eq!(
        transitions,
        [
            TravelEvent::StationArrived {
                station: StationId::Pines,
                visit: 1,
                distance_units: first_leg,
            },
            TravelEvent::StationDeparted {
                station: StationId::Pines,
                distance_units: first_leg,
            },
        ]
    );
    assert_close(
        journey.snapshot().route_position_units,
        first_leg + 10.0,
        1e-12,
    );
    assert_eq!(journey.snapshot().at_station, None);
}

#[test]
fn completing_a_circuit_discovers_every_station_without_resetting_distance() {
    let mut journey = Journey::default();
    let length = journey.route().length_units();
    advance_distance(&mut journey, length);
    let snapshot = journey.snapshot();
    assert_eq!(snapshot.total_distance_units, length);
    assert_eq!(snapshot.route_position_units, 0.0);
    assert_eq!(snapshot.route_progress, 0.0);
    assert_eq!(snapshot.segment_progress, 0.0);
    assert_eq!(snapshot.completed_laps, 1);
    assert_eq!(snapshot.stations_visited, 4);
    assert_eq!(snapshot.discovered_stations, [true; 4]);
    assert_eq!(snapshot.at_station, Some(StationId::Aurora));
    assert_eq!(snapshot.next_station.id, StationId::Pines);
    assert!(snapshot.achievements.contains(Achievement::FirstCircuit));
    let arrivals: Vec<_> = journey
        .drain_events()
        .filter_map(|event| match event {
            TravelEvent::StationArrived { station, .. } => Some(station),
            _ => None,
        })
        .collect();
    assert_eq!(
        arrivals,
        [
            StationId::Pines,
            StationId::Ridge,
            StationId::Harbor,
            StationId::Aurora
        ]
    );
    journey.advance(1.0, 10.0);
    assert_eq!(journey.snapshot().total_distance_units, length + 10.0);
    assert_eq!(journey.snapshot().route_position_units, 10.0);
}

#[test]
fn active_time_and_distance_are_distinct_and_zero_motion_has_no_transition() {
    let mut journey = Journey::default();
    assert_eq!(
        journey.advance(1.0, 0.0),
        AdvanceOutcome::Advanced {
            seconds: 1.0,
            distance_units: 0.0
        }
    );
    assert_eq!(journey.snapshot().active_seconds, 1.0);
    assert_eq!(journey.snapshot().total_distance_units, 0.0);
    assert_eq!(journey.snapshot().at_station, Some(StationId::Aurora));
    assert_eq!(journey.pop_event(), None);
    journey.advance(0.5, 100.0);
    assert_eq!(journey.snapshot().active_seconds, 1.5);
    assert_eq!(journey.snapshot().total_distance_units, 100.0);
}

#[test]
fn rejected_elapsed_time_never_becomes_offline_travel() {
    let mut journey = Journey::default();
    journey.advance(0.5, 50.0);
    let before = journey.snapshot();
    for (dt, expected) in [
        (-1.0, AdvanceOutcome::InvalidDelta),
        (f64::NAN, AdvanceOutcome::InvalidDelta),
        (f64::INFINITY, AdvanceOutcome::InvalidDelta),
        (f64::NEG_INFINITY, AdvanceOutcome::InvalidDelta),
        (0.0, AdvanceOutcome::NoElapsedTime),
        (MAX_ACTIVE_DELTA_SECONDS + 1e-8, AdvanceOutcome::Suspended),
        (86_400.0, AdvanceOutcome::Suspended),
        (f64::MAX, AdvanceOutcome::Suspended),
    ] {
        assert_eq!(journey.advance(dt, 100.0), expected);
        assert_eq!(journey.snapshot(), before);
    }
    journey.advance(0.5, 50.0);
    assert_eq!(journey.snapshot().total_distance_units, 100.0);
    assert_eq!(journey.snapshot().active_seconds, 1.0);
}

#[test]
fn invalid_distance_discards_the_whole_step_including_active_time() {
    let mut journey = Journey::default();
    journey.advance(1.0, 25.0);
    let before = journey.snapshot();
    for distance in [
        -1.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX,
        MAX_TRAVEL_UNITS_PER_SECOND + 0.01,
    ] {
        assert_eq!(
            journey.advance(1.0, distance),
            AdvanceOutcome::InvalidDistance
        );
        assert_eq!(journey.snapshot(), before);
    }
}

#[test]
fn displacement_bound_accepts_legitimate_roundoff_but_rejects_excess() {
    for dt in [1.0, 0.5, 1.0 / 60.0, 1e-6] {
        let maximum = MAX_TRAVEL_UNITS_PER_SECOND * dt;
        for distance in [maximum, f64::from_bits(maximum.to_bits() + 4)] {
            let mut journey = Journey::default();
            assert_eq!(
                journey.advance(dt, distance),
                AdvanceOutcome::Advanced {
                    seconds: dt,
                    distance_units: maximum,
                }
            );
            assert_eq!(journey.snapshot().total_distance_units, maximum);
        }
        let mut journey = Journey::default();
        assert_eq!(
            journey.advance(dt, maximum + 1e-6),
            AdvanceOutcome::InvalidDistance
        );
        assert_eq!(journey.snapshot().total_distance_units, 0.0);
    }
}

#[test]
fn journey_pause_is_reversible_and_does_not_create_events() {
    let mut journey = Journey::default();
    journey.advance(1.0, 25.0);
    journey.set_paused(true);
    let paused = journey.snapshot();
    assert!(journey.is_paused());
    journey.set_paused(true);
    assert_eq!(journey.advance(1.0, 100.0), AdvanceOutcome::Paused);
    assert_eq!(journey.snapshot(), paused);
    journey.set_paused(false);
    journey.set_paused(false);
    assert!(!journey.is_paused());
    journey.advance(1.0, 25.0);
    assert_eq!(journey.snapshot().total_distance_units, 50.0);
    assert_eq!(journey.snapshot().active_seconds, 2.0);
    assert_eq!(journey.pending_event_count(), paused.pending_events);
}

#[test]
fn paused_adapter_freezes_train_journey_and_scenery_together() {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    world.advance_train(&mut train, 100.0, 1.0, &config);
    world.set_paused(true);
    let before = world.journey_snapshot();
    let motion = (
        train.speed_kmh(),
        train.target_speed_kmh(),
        train.wheel_phase(),
        world.elapsed_seconds(),
        world.scroll_px(),
    );
    let smoke: Vec<_> = world
        .smoke()
        .iter()
        .map(|p| (p.x, p.y, p.age, p.size))
        .collect();
    assert_eq!(
        world.advance_train(&mut train, 0.0, 1.0, &config),
        AdvanceOutcome::Paused
    );
    assert_eq!(world.journey_snapshot(), before);
    assert_eq!(
        (
            train.speed_kmh(),
            train.target_speed_kmh(),
            train.wheel_phase(),
            world.elapsed_seconds(),
            world.scroll_px()
        ),
        motion
    );
    assert_eq!(
        world
            .smoke()
            .iter()
            .map(|p| (p.x, p.y, p.age, p.size))
            .collect::<Vec<_>>(),
        smoke
    );
    world.set_paused(false);
    assert!(matches!(
        world.advance_train(&mut train, 0.0, 0.5, &config),
        AdvanceOutcome::Advanced { .. }
    ));
    assert!(world.journey_snapshot().total_distance_units > before.total_distance_units);
    assert_eq!(
        world.journey_snapshot().active_seconds,
        before.active_seconds + 0.5
    );
}

#[test]
fn invalid_or_suspended_adapter_step_leaves_train_state_untouched() {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    world.advance_train(&mut train, 100.0, 0.5, &config);
    let before = world.journey_snapshot();
    let motion = (
        train.speed_kmh(),
        train.target_speed_kmh(),
        train.wheel_phase(),
        world.elapsed_seconds(),
        world.scroll_px(),
    );
    for (dt, expected) in [
        (f64::NAN, AdvanceOutcome::InvalidDelta),
        (f64::INFINITY, AdvanceOutcome::InvalidDelta),
        (-1.0, AdvanceOutcome::InvalidDelta),
        (0.0, AdvanceOutcome::NoElapsedTime),
        (60.0, AdvanceOutcome::Suspended),
        (f64::MAX, AdvanceOutcome::Suspended),
    ] {
        assert_eq!(world.advance_train(&mut train, 0.0, dt, &config), expected);
        assert_eq!(world.journey_snapshot(), before);
        assert_eq!(
            (
                train.speed_kmh(),
                train.target_speed_kmh(),
                train.wheel_phase(),
                world.elapsed_seconds(),
                world.scroll_px()
            ),
            motion
        );
    }
}

#[test]
fn event_queue_keeps_only_the_newest_events_without_losing_achievements() {
    let mut buffered = Journey::from_seed(17);
    let mut consumed = Journey::from_seed(17);
    let mut all_events = Vec::new();
    let target = (buffered.route().length_units() * 20.0)
        .max(LONG_JOURNEY_DISTANCE_UNITS + buffered.route().length_units());
    let mut remaining = target;
    while remaining > 0.0 {
        let distance = remaining.min(100.0);
        buffered.advance(1.0, distance);
        consumed.advance(1.0, distance);
        all_events.extend(consumed.drain_events());
        assert!(buffered.pending_event_count() <= MAX_TRAVEL_EVENTS);
        remaining -= distance;
    }
    assert!(all_events.len() > MAX_TRAVEL_EVENTS);
    assert_eq!(buffered.pending_event_count(), MAX_TRAVEL_EVENTS);
    assert_eq!(
        buffered.dropped_event_count(),
        (all_events.len() - MAX_TRAVEL_EVENTS) as u64
    );
    let tail: Vec<_> = buffered.drain_events().collect();
    assert_eq!(tail, all_events[all_events.len() - MAX_TRAVEL_EVENTS..]);
    assert_eq!(buffered.pending_event_count(), 0);
    for achievement in [
        Achievement::FirstStation,
        Achievement::FirstCircuit,
        Achievement::LongJourney,
    ] {
        assert!(buffered.snapshot().achievements.contains(achievement));
    }
    assert_eq!(
        buffered.snapshot().achievements,
        consumed.snapshot().achievements
    );
}

#[test]
fn consuming_events_cannot_replay_or_erase_an_achievement() {
    let mut journey = Journey::default();
    advance_distance(&mut journey, LONG_JOURNEY_DISTANCE_UNITS);
    assert!(journey.snapshot().achievements.long_journey);
    assert!(journey.pop_event().is_some());
    journey.drain_events().for_each(drop);
    assert_eq!(journey.pop_event(), None);
    assert_eq!(journey.pending_event_count(), 0);
    let achievements = journey.snapshot().achievements;
    let length = journey.route().length_units();
    advance_distance(&mut journey, length);
    assert_eq!(journey.snapshot().achievements, achievements);
    assert!(
        !journey
            .drain_events()
            .any(|event| matches!(event, TravelEvent::AchievementUnlocked { .. }))
    );
}

#[test]
fn identical_seed_and_inputs_reproduce_snapshots_and_events() {
    for seed in [0, 1, 42, u64::MAX] {
        let mut first = Journey::from_seed(seed);
        let mut second = Journey::from_seed(seed);
        for index in 0..1_000 {
            let distance = f64::from(index % 250);
            assert_eq!(first.advance(1.0, distance), second.advance(1.0, distance));
            assert_eq!(first.snapshot(), second.snapshot());
            assert_eq!(
                first.drain_events().collect::<Vec<_>>(),
                second.drain_events().collect::<Vec<_>>()
            );
        }
        assert_eq!(first.snapshot().seed, seed);
    }
}

#[test]
fn boundary_roundoff_cannot_place_departure_before_the_preceding_arrival() {
    let mut journey = Journey::default();
    let first_leg = journey.route().segments()[0].length_units;
    advance_distance(&mut journey, first_leg - 5e-10);
    let arrival = journey
        .drain_events()
        .find_map(|event| match event {
            TravelEvent::StationArrived {
                station: StationId::Pines,
                distance_units,
                ..
            } => Some(distance_units),
            _ => None,
        })
        .expect("boundary round-off should produce the arrival cue");
    journey.advance(1.0, 1.0);
    let departure = journey
        .drain_events()
        .find_map(|event| match event {
            TravelEvent::StationDeparted {
                station: StationId::Pines,
                distance_units,
            } => Some(distance_units),
            _ => None,
        })
        .expect("motion after arrival should produce departure");
    assert!(
        departure >= arrival,
        "departure={departure:.15} precedes arrival={arrival:.15}"
    );
}
