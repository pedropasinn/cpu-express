//! The generic itinerary reproduces the running circuit and handles end-to-end lines.
use cpu_express::simulation::{
    Journey, Route,
    catalog::{Catalog, Layout},
    itinerary::{DEFAULT_UNITS_PER_KM, Itinerary, playable},
};

fn catalog() -> &'static Catalog {
    Catalog::builtin().as_ref().unwrap()
}

/// Deterministic pseudo-random distances without a dependency.
fn samples(seed: u64, count: usize, limit: f64) -> Vec<f64> {
    let mut state = seed;
    (0..count)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (state >> 11) as f64 / (1_u64 << 53) as f64 * limit
        })
        .collect()
}

#[test]
fn circuit_itinerary_agrees_with_the_running_engine() {
    for seed in [0, 7, 0x4350_5520_4558_5052, u64::MAX] {
        let route = Route::from_seed(seed);
        let lengths: Vec<f64> = route.segments().iter().map(|s| s.length_units).collect();
        let itinerary = Itinerary::from_lengths("aurora-v1", Layout::Circuit, lengths).unwrap();
        assert_eq!(itinerary.station_count(), 4);
        assert_eq!(itinerary.run_length(), route.length_units());
        let mut distances = samples(seed ^ 0xA5A5, 400, 250_000.0);
        // Exact platforms, including the origin and many laps later.
        for lap in [0.0, 1.0, 13.0] {
            let mut position = lap * route.length_units();
            distances.push(position);
            for segment in route.segments() {
                position += segment.length_units;
                distances.push(position);
            }
        }
        for distance in distances {
            let journey = Journey::restore_checkpoint(seed, distance, 0.0).unwrap();
            let expected = journey.snapshot();
            let position = itinerary.locate(distance);
            assert_eq!(
                position.stations_visited, expected.stations_visited,
                "{distance}"
            );
            assert_eq!(position.runs_completed, expected.completed_laps);
            assert_eq!(position.segment, expected.route_segment.index);
            assert_eq!(position.from, expected.route_segment.from.index());
            assert_eq!(position.to, expected.route_segment.to.index());
            assert_eq!(
                position.at_station,
                expected.at_station.map(|station| station.index())
            );
            assert!(
                (position.distance_to_next - expected.distance_to_next_station_units).abs() < 1e-6,
                "{distance}"
            );
            assert!(position.forward);
        }
    }
}

#[test]
fn catalogue_circuit_uses_base_lengths_and_refuses_drafts() {
    let aurora = playable(catalog().line("aurora-v1").unwrap(), DEFAULT_UNITS_PER_KM).unwrap();
    assert_eq!(aurora.layout(), Layout::Circuit);
    assert_eq!(aurora.run_length(), 9000.0);
    let draft = catalog().line("curitiba-paranagua").unwrap();
    let error = playable(draft, DEFAULT_UNITS_PER_KM).unwrap_err();
    assert!(error.contains("curitiba-paranagua") && error.contains("rascunho"));
    assert!(Itinerary::from_line(draft, f64::NAN).is_err());
    assert!(Itinerary::from_line(draft, 0.0).is_err());
}

#[test]
fn shuttle_runs_end_to_end_and_returns_over_the_same_segments() {
    let line = catalog().line("curitiba-paranagua").unwrap();
    let itinerary = Itinerary::from_line(line, DEFAULT_UNITS_PER_KM).unwrap();
    let stations = line.stations.len();
    assert_eq!(itinerary.station_count(), stations);
    assert_eq!(itinerary.layout(), Layout::Shuttle);
    let run = itinerary.run_length();
    let last_km = line.stations.last().unwrap().km.unwrap();
    assert!((run - last_km * DEFAULT_UNITS_PER_KM).abs() < 1e-6);
    for (index, station) in line.stations.iter().enumerate() {
        let offset = itinerary.station_offset(index).unwrap();
        assert!((offset - station.km.unwrap() * DEFAULT_UNITS_PER_KM).abs() < 1e-6);
    }

    // Order of arrivals: up to Curitiba, back to Paranaguá, and again.
    let name = |index: usize| line.stations[index].id.as_str();
    let visits: Vec<&str> = (0..=28)
        .map(|v| name(itinerary.station_at_visit(v)))
        .collect();
    assert_eq!(
        &visits[..4],
        ["paranagua", "alexandra", "saquarema", "morretes"]
    );
    assert_eq!(visits[13], "curitiba");
    assert_eq!(visits[14], "pinhais");
    assert_eq!(visits[23], "morretes");
    assert_eq!(visits[26], "paranagua");
    assert_eq!(visits[27], "alexandra");

    // Origin, first run, the far end, the return and the second departure.
    let origin = itinerary.locate(0.0);
    assert_eq!((origin.at_station, origin.from, origin.to), (Some(0), 0, 1));
    assert!(origin.forward);
    let morretes = itinerary.station_offset(3).unwrap();
    let climbing = itinerary.locate(morretes + 100.0);
    assert_eq!((climbing.from, climbing.to, climbing.segment), (3, 4, 3));
    assert_eq!(climbing.stations_visited, 3);
    assert!(climbing.forward && climbing.at_station.is_none());

    let far_end = itinerary.locate(run);
    assert_eq!(far_end.at_station, Some(stations - 1));
    assert_eq!((far_end.from, far_end.to), (stations - 1, stations - 2));
    assert_eq!(far_end.segment, stations - 2);
    assert!(!far_end.forward);
    assert_eq!(far_end.runs_completed, 1);

    // Coming down the same distance below Curitiba as `climbing` was above Morretes.
    let descending = itinerary.locate(2.0 * run - morretes - 100.0);
    assert_eq!(
        (descending.from, descending.to, descending.segment),
        (4, 3, 3)
    );
    assert!(!descending.forward);
    assert!((descending.distance_to_next - 100.0).abs() < 1e-6);

    let home = itinerary.locate(2.0 * run);
    assert_eq!(home.at_station, Some(0));
    assert_eq!((home.from, home.to, home.runs_completed), (0, 1, 2));
    assert!(home.forward);
    assert_eq!(home.stations_visited, 26);
}

#[test]
fn position_is_consistent_for_any_distance_on_both_layouts() {
    let shuttle =
        Itinerary::from_line(catalog().line("curitiba-paranagua").unwrap(), 480.0).unwrap();
    let circuit = playable(catalog().line("aurora-v1").unwrap(), 480.0).unwrap();
    for itinerary in [&shuttle, &circuit] {
        let mut previous_visits = 0;
        let mut distances = samples(99, 2_000, itinerary.run_length() * 9.0);
        distances.sort_by(f64::total_cmp);
        for distance in distances {
            let position = itinerary.locate(distance);
            let length = itinerary.segment_length(position.segment).unwrap();
            assert!(position.distance_into_segment >= 0.0);
            assert!(
                (position.distance_into_segment + position.distance_to_next - length).abs() < 1e-6
            );
            // Arrivals only grow, one platform at a time along the line.
            assert!(position.stations_visited >= previous_visits);
            previous_visits = position.stations_visited;
            assert_eq!(
                position.from,
                itinerary.station_at_visit(position.stations_visited)
            );
            assert_eq!(
                position.to,
                itinerary.station_at_visit(position.stations_visited + 1)
            );
            let step = position.from.abs_diff(position.to);
            assert!(step == 1 || step == itinerary.station_count() - 1);
            // The segment travelled always joins the two stations.
            if itinerary.layout() == Layout::Shuttle {
                assert_eq!(position.segment, position.from.min(position.to));
            }
        }
        // Invalid input never panics and means the origin.
        for invalid in [f64::NAN, f64::INFINITY, -5.0] {
            assert_eq!(itinerary.locate(invalid), itinerary.locate(0.0));
        }
    }
    assert!(Itinerary::from_lengths("x", Layout::Circuit, vec![10.0]).is_err());
    assert!(Itinerary::from_lengths("x", Layout::Shuttle, vec![]).is_err());
    assert!(Itinerary::from_lengths("x", Layout::Shuttle, vec![10.0, -1.0]).is_err());
}
