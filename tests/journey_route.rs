use cpu_express::simulation::route::{
    Biome, DEFAULT_JOURNEY_SEED, Route, STATION_COUNT, StationId,
};

#[test]
fn every_seed_builds_one_connected_four_station_circuit() {
    for seed in [0, 1, 42, DEFAULT_JOURNEY_SEED, u64::MAX] {
        let route = Route::from_seed(seed);
        assert_eq!(route.seed(), seed);
        assert_eq!(route.stations().len(), STATION_COUNT);
        assert_eq!(route.segments().len(), STATION_COUNT);
        assert_eq!(route.stations()[0].distance_from_origin_units, 0.0);
        assert_eq!(route.stations()[0].id, StationId::Aurora);

        let mut total_length = 0.0;
        for (index, segment) in route.segments().iter().enumerate() {
            let station = &route.stations()[index];
            let next = &route.stations()[(index + 1) % STATION_COUNT];
            assert_eq!(segment.index, index);
            assert_eq!(segment.from, station.id);
            assert_eq!(segment.to, next.id);
            assert_eq!(station.id.index(), index);
            assert_eq!(station.name, station.id.name());
            assert_eq!(route.station(station.id), station);
            assert_eq!(route.segment(index), Some(segment));
            assert_eq!(station.distance_from_origin_units, total_length);
            assert!(segment.length_units.is_finite() && segment.length_units > 0.0);
            total_length += segment.length_units;
        }
        assert_eq!(total_length, route.length_units());
        assert_eq!(route.segments()[3].to, StationId::Aurora);
        assert!(route.segment(STATION_COUNT).is_none());
        assert!(route.segment(usize::MAX).is_none());
    }
}

#[test]
fn seed_is_repeatable_and_changes_lengths_without_changing_station_identity() {
    assert_eq!(Route::default(), Route::from_seed(DEFAULT_JOURNEY_SEED));
    let zero = Route::from_seed(0);
    let one = Route::from_seed(1);
    let zero_again = Route::from_seed(0);
    assert_eq!(zero, zero_again);
    assert_ne!(zero.segments(), one.segments());
    assert_eq!(zero.name(), "Circuito Aurora");
    for (original, other) in zero.stations().iter().zip(one.stations()) {
        assert_eq!(original.id, other.id);
        assert_eq!(original.name, other.name);
    }
    for (original, other) in zero.segments().iter().zip(one.segments()) {
        assert_eq!(original.from, other.from);
        assert_eq!(original.to, other.to);
        assert_eq!(original.biome_hint, other.biome_hint);
    }
}

#[test]
fn published_seed_fixtures_keep_the_route_reproducible_across_releases() {
    // Deliberate compatibility fixtures, not a second RNG implementation.
    // If route generation intentionally changes, persistence needs a route
    // version instead of silently reinterpreting a saved position.
    for (seed, expected) in [
        (0, [1_875.0, 2_300.0, 2_025.0, 2_775.0]),
        (DEFAULT_JOURNEY_SEED, [1_900.0, 2_375.0, 2_025.0, 2_725.0]),
    ] {
        let route = Route::from_seed(seed);
        assert_eq!(route.segments().map(|leg| leg.length_units), expected);
    }
}

#[test]
fn legs_are_distinct_and_expose_four_theme_hints_without_visuals() {
    let expected_biomes = [Biome::Meadow, Biome::Forest, Biome::Highland, Biome::Coast];
    for seed in 0..256 {
        let route = Route::from_seed(seed);
        assert!((8_600.0..=9_400.0).contains(&route.length_units()));
        for (index, segment) in route.segments().iter().enumerate() {
            assert_eq!(segment.biome_hint, expected_biomes[index]);
            assert!((1_700.0..=2_800.0).contains(&segment.length_units));
            assert_eq!(segment.length_units % 25.0, 0.0);
            for other in &route.segments()[index + 1..] {
                assert_ne!(segment.length_units, other.length_units);
            }
        }
    }
}

#[test]
fn station_names_are_stable_display_labels() {
    let route = Route::default();
    assert_eq!(route.station(StationId::Aurora).name, "Vila Aurora");
    assert_eq!(route.station(StationId::Pines).name, "Bosque dos Pinheiros");
    assert_eq!(route.station(StationId::Ridge).name, "Serra Clara");
    assert_eq!(route.station(StationId::Harbor).name, "Porto do Sol");
}
