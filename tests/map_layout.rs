use cpu_express::simulation::{
    Route,
    catalog::{Catalog, Layout, LineDef},
    itinerary::{DEFAULT_UNITS_PER_KM, Itinerary},
    map_layout::MapLayout,
};

fn line() -> LineDef {
    Catalog::builtin()
        .as_ref()
        .unwrap()
        .line("curitiba-paranagua")
        .unwrap()
        .clone()
}

#[test]
fn fourteen_stations_and_train_follow_both_directions_and_endpoint_dwell() {
    let line = line();
    let itinerary = Itinerary::from_line(&line, DEFAULT_UNITS_PER_KM).unwrap();
    let map = MapLayout::new(&line, &itinerary).unwrap();
    assert_eq!(map.stations().len(), 14);
    for i in 0..14 {
        let offset = itinerary.station_offset(i).unwrap();
        let station = &map.stations()[i];
        for distance in [offset, 2.0 * itinerary.run_length() - offset] {
            let train = map.train(distance);
            assert_eq!(train.point, station.point);
            assert_eq!(train.profile, station.profile);
            assert_eq!(train.at_station, Some(i));
            assert!(map.discovered(i, distance));
        }
        assert_eq!(map.discovered(i, 0.0), i == 0);
    }
    let half = itinerary.segment_length(4).unwrap() / 2.0;
    let outbound = itinerary.station_offset(4).unwrap() + half;
    let a = map.train(outbound);
    let b = map.train(2.0 * itinerary.run_length() - outbound);
    assert!((a.point.x - b.point.x).abs() < 1e-9 && (a.point.y - b.point.y).abs() < 1e-9);
    assert!(a.forward && !b.forward);
    assert_eq!((a.from, a.to), (b.to, b.from));
    for bad in [f64::NAN, f64::INFINITY, -1.0] {
        assert_eq!(map.train(bad), map.train(0.0));
    }
}

#[test]
fn twenty_nine_stations_fit_five_rows_without_overlapping_markers() {
    let mut line = line();
    let station = line.stations[0].clone();
    let segment = line.segments[0].clone();
    line.stations = (0..29)
        .map(|i| {
            let mut s = station.clone();
            s.id = format!("fixture-{i}");
            s.km = Some(i as f64);
            s.elevation_m = Some(100.0);
            s
        })
        .collect();
    line.segments = (0..28)
        .map(|i| {
            let mut s = segment.clone();
            s.from = line.stations[i].id.clone();
            s.to = line.stations[i + 1].id.clone();
            s.distance_km = Some(1.0);
            s
        })
        .collect();
    let itinerary = Itinerary::from_line(&line, 480.0).unwrap();
    let map = MapLayout::new(&line, &itinerary).unwrap();
    assert_eq!(map.schematic_height(), 128.0);
    for (i, s) in map.stations().iter().enumerate() {
        assert!((0.0..=1.0).contains(&s.point.x) && (0.0..=1.0).contains(&s.point.y));
        assert!(
            !map.stations()[..i]
                .iter()
                .any(|other| other.point == s.point)
        );
        assert_eq!(s.profile.unwrap().y, 0.5);
    }
    assert_eq!(map.train(itinerary.run_length()).at_station, Some(28));
    assert!(!map.train(itinerary.run_length()).forward);
}

#[test]
fn missing_altitudes_are_gaps_and_retired_stations_keep_catalog_indices() {
    let mut line = line();
    line.stations[1].elevation_m = None;
    let mut retired = line.stations[0].clone();
    retired.id = "retired-fixture".into();
    retired.retired = true;
    line.stations.insert(1, retired);
    let itinerary = Itinerary::from_line(&line, 480.0).unwrap();
    let map = MapLayout::new(&line, &itinerary).unwrap();
    assert_eq!(map.stations()[1].catalog_index, 2);
    assert_eq!(map.stations()[1].profile, None);
    assert_eq!(map.train(100.0).profile, None);
    assert!(map.train(0.0).profile.is_some());
    for station in &mut line.stations {
        station.elevation_m = None;
    }
    assert_eq!(
        MapLayout::new(&line, &itinerary).unwrap().elevation_range(),
        None
    );
}

#[test]
fn seeded_circuit_closes_and_mismatched_lines_are_refused() {
    let mut line = Catalog::builtin()
        .as_ref()
        .unwrap()
        .line("aurora-v1")
        .unwrap()
        .clone();
    let route = Route::from_seed(9);
    let itinerary = Itinerary::from_lengths(
        "aurora-v1",
        Layout::Circuit,
        route.segments().iter().map(|s| s.length_units).collect(),
    )
    .unwrap();
    for (i, station) in line.stations.iter_mut().enumerate() {
        station.elevation_m = Some(i as f64 * 10.0);
    }
    let map = MapLayout::new(&line, &itinerary).unwrap();
    let last = itinerary.station_offset(3).unwrap();
    let mid = last + itinerary.segment_length(3).unwrap() / 2.0;
    assert!(map.train(mid).profile.unwrap().x > map.stations()[3].profile.unwrap().x);
    assert_eq!(
        map.train(itinerary.run_length()).point,
        map.stations()[0].point
    );
    line.id = "wrong".into();
    assert!(MapLayout::new(&line, &itinerary).is_err());
    line.id = "aurora-v1".into();
    line.stations.swap(1, 2);
    assert!(MapLayout::new(&line, &itinerary).is_err());
}
