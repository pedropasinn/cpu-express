use cpu_express::{
    local_weather::{LocalWeather, Snapshot, WeatherLocation, load_cache, save_cache},
    rendering::Weather,
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

fn path() -> PathBuf {
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "cpu-express-weather-{}-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ))
}

fn snapshot(age: u64) -> Snapshot {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    Snapshot {
        weather_code: 61,
        weather: Weather::Rain,
        temperature_c: 19.5,
        is_day: false,
        observed_unix: now - age,
        received_unix: now - age + 30,
        utc_offset_seconds: -10800,
        wind_speed_kmh: None,
        precipitation_mm: None,
        sunrise_unix: None,
        sunset_unix: None,
    }
}

#[test]
fn local_weather_location_validation_and_serde_agree() {
    let default = WeatherLocation::default();
    default.validate().unwrap();
    assert_eq!(default.city, "São Paulo, SP, Brasil");
    let json = serde_json::to_string(&default).unwrap();
    assert_eq!(
        serde_json::from_str::<WeatherLocation>(&json).unwrap(),
        default
    );
    for city in ["", "  ", " São Paulo", "A\nB", "A\0B", "A\u{202e}B"] {
        assert!(WeatherLocation::new(city, 0.0, 0.0).is_err());
    }
    assert!(WeatherLocation::new("a".repeat(121), 0.0, 0.0).is_err());
    for (lat, lon) in [
        (91.0, 0.0),
        (0.0, -181.0),
        (f64::NAN, 0.0),
        (0.0, f64::INFINITY),
    ] {
        assert!(WeatherLocation::new("City", lat, lon).is_err());
    }
    for json in [
        r#"{"city":"City","latitude":91,"longitude":0}"#,
        r#"{"city":"City","latitude":0,"longitude":0,"url":"evil"}"#,
        r#"{"city":"City","latitude":0}"#,
    ] {
        assert!(serde_json::from_str::<WeatherLocation>(json).is_err());
    }
    WeatherLocation::new("Polo Sul", -90.0, 180.0).unwrap();
}

#[test]
fn local_weather_restart_keeps_old_observation_and_replaces_valid_cache() {
    let path = path();
    let location = WeatherLocation::default();
    let old = snapshot(86_400);
    save_cache(&path, &location, &old).unwrap();
    let mut restarted = LocalWeather::new(true);
    restarted.set_cache_path(&path).unwrap();
    let restored = restarted.last_good.as_ref().unwrap();
    assert_eq!(restored.received_unix, old.received_unix);
    assert_eq!(restored.observed_unix, old.observed_unix);
    assert!(restored.age_seconds() >= 86_400);
    assert!(restored.stale());
    assert_eq!(restarted.status(), "stale");
    assert_eq!(restarted.atmosphere(), Some((Weather::Rain, false)));
    assert_eq!(restarted.diagnostics()["stale"], true);
    let fresh = snapshot(120);
    save_cache(&path, &location, &fresh).unwrap();
    assert_eq!(
        load_cache(&path, &location).unwrap().unwrap().observed_unix,
        fresh.observed_unix
    );
}

#[test]
fn local_weather_old_cache_defaults_and_new_solar_fields_round_trip() {
    let path = path();
    let location = WeatherLocation::default();
    let mut sample = snapshot(86_400);
    save_cache(&path, &location, &sample).unwrap();
    let mut old: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for field in [
        "wind_speed_kmh",
        "precipitation_mm",
        "sunrise_unix",
        "sunset_unix",
    ] {
        old["snapshot"].as_object_mut().unwrap().remove(field);
    }
    std::fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
    let legacy = load_cache(&path, &location).unwrap().unwrap();
    assert_eq!(legacy.observed_unix, sample.observed_unix);
    assert!(legacy.stale());
    assert!(legacy.sunrise_unix.is_none());
    assert!(legacy.wind_speed_kmh.is_none());
    assert!(legacy.lighting_phase_at(sample.received_unix).is_none());

    let midnight = ((sample.observed_unix as i64 + i64::from(sample.utc_offset_seconds))
        .div_euclid(86400)
        * 86400
        - i64::from(sample.utc_offset_seconds)) as u64;
    sample.sunrise_unix = Some(midnight + 6 * 3600);
    sample.sunset_unix = Some(midnight + 18 * 3600);
    sample.wind_speed_kmh = Some(12.0);
    sample.precipitation_mm = Some(0.4);
    save_cache(&path, &location, &sample).unwrap();
    let restored = load_cache(&path, &location).unwrap().unwrap();
    assert_eq!(restored.sunrise_unix, sample.sunrise_unix);
    assert_eq!(restored.sunset_unix, sample.sunset_unix);
    assert_eq!(restored.wind_speed_kmh, Some(12.0));
    assert_eq!(restored.precipitation_mm, Some(0.4));
    assert_eq!(restored.received_unix, sample.received_unix);
    assert_eq!(restored.lighting_phase_at(midnight + 6 * 3600), Some(0.0));
    let bytes = std::fs::read(&path).unwrap();
    sample.wind_speed_kmh = Some(f32::NAN);
    assert!(save_cache(&path, &location, &sample).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}

#[test]
fn local_weather_cache_is_bound_to_city_and_coordinates() {
    let path = path();
    let sp = WeatherLocation::default();
    let lisbon = WeatherLocation::new("Lisboa", 38.72, -9.14).unwrap();
    save_cache(&path, &sp, &snapshot(120)).unwrap();
    assert!(load_cache(&path, &lisbon).unwrap().is_none());
    let mut app = LocalWeather::new(false);
    app.set_cache_path(&path).unwrap();
    assert!(app.last_good.is_some());
    app.set_location(lisbon.clone()).unwrap();
    assert!(app.last_good.is_none());
    assert_eq!(app.location(), &lisbon);
    assert!(app.poll(&eframe::egui::Context::default()));
    assert_eq!(
        load_cache(&path, &sp).unwrap().unwrap().weather,
        Weather::Rain
    );
    let invalid = WeatherLocation {
        latitude: f64::NAN,
        ..sp.clone()
    };
    assert!(app.set_location(invalid).is_err());
    assert_eq!(app.location(), &lisbon);
    app.set_location(sp).unwrap();
    assert!(app.last_good.is_some());
}

#[test]
fn local_weather_corrupt_future_and_oversized_cache_are_never_overwritten() {
    let location = WeatherLocation::default();
    let sample = snapshot(120);
    let valid_path = path();
    save_cache(&valid_path, &location, &sample).unwrap();
    let valid: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&valid_path).unwrap()).unwrap();
    let mut cases = vec![b"broken".to_vec(), vec![b' '; 8193]];
    for (field, value) in [
        ("weather_code", serde_json::json!(999)),
        ("temperature_c", serde_json::json!(1000)),
        ("is_day", serde_json::json!(2)),
        ("observed_unix", serde_json::json!(0)),
        ("received_unix", serde_json::json!(u64::MAX)),
        ("utc_offset_seconds", serde_json::json!(999999)),
        ("wind_speed_kmh", serde_json::json!(-1)),
        ("precipitation_mm", serde_json::json!(-1)),
        ("sunrise_unix", serde_json::json!(0)),
        ("extra", serde_json::json!(true)),
    ] {
        let mut invalid = valid.clone();
        invalid["snapshot"][field] = value;
        cases.push(serde_json::to_vec(&invalid).unwrap());
    }
    let mut future = valid.clone();
    future["version"] = serde_json::json!(99);
    cases.push(serde_json::to_vec(&future).unwrap());
    let mut unknown = valid;
    unknown["extra"] = serde_json::json!(true);
    cases.push(serde_json::to_vec(&unknown).unwrap());
    for bytes in cases {
        let target = path();
        std::fs::write(&target, &bytes).unwrap();
        assert!(load_cache(&target, &location).is_err());
        assert!(save_cache(&target, &location, &sample).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), bytes);
        let mut app = LocalWeather::new(false);
        assert!(app.set_cache_path(&target).is_err());
        assert!(app.cache_error.is_some());
        app.last_good = Some(sample.clone());
        assert!(app.persist_cache().is_err());
        assert!(app.last_good.is_some());
        assert_eq!(std::fs::read(&target).unwrap(), bytes);
    }
}

#[test]
fn local_weather_failed_write_recovers_and_invalid_snapshot_preserves_cache() {
    let cache_path = path();
    let sample = snapshot(120);
    let location = WeatherLocation::default();
    save_cache(&cache_path, &location, &sample).unwrap();
    let bytes = std::fs::read(&cache_path).unwrap();
    let mut bad = sample.clone();
    bad.weather = Weather::Clear;
    assert!(save_cache(&cache_path, &location, &bad).is_err());
    bad = sample.clone();
    bad.received_unix = bad.observed_unix + 7201;
    assert!(save_cache(&cache_path, &location, &bad).is_err());
    assert_eq!(std::fs::read(&cache_path).unwrap(), bytes);

    let mut app = LocalWeather::new(false);
    // A file as parent causes a real filesystem write failure, without network.
    // Windows may classify the initial read as NotFound rather than NotADirectory.
    let _ = app.set_cache_path(cache_path.join("weather.json"));
    app.last_good = Some(sample.clone());
    assert!(app.persist_cache().is_err());
    assert!(app.cache_error.is_some());
    assert_eq!(
        app.last_good.as_ref().unwrap().observed_unix,
        sample.observed_unix
    );
    let recovered = path();
    app.set_cache_path(&recovered).unwrap();
    app.persist_cache().unwrap();
    assert!(app.cache_error.is_none());
    assert!(load_cache(&recovered, &location).unwrap().is_some());
}
