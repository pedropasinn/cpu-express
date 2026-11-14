//! Configured-location weather. All network work stays off the UI/render thread.
use crate::rendering::Weather;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    sync::mpsc::{self, Receiver, TryRecvError},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const REFRESH: Duration = Duration::from_secs(900);
const COOLDOWN: Duration = Duration::from_secs(60);
const MAX_BODY: usize = 65_536;
const MAX_CACHE: usize = 8192;

/// A user-selected location, never inferred from the device's IP or GPS.
/// Public fields allow config/UI integration; every I/O boundary revalidates them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "LocationFields")]
pub struct WeatherLocation {
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LocationFields {
    city: String,
    latitude: f64,
    longitude: f64,
}

impl TryFrom<LocationFields> for WeatherLocation {
    type Error = String;
    fn try_from(value: LocationFields) -> Result<Self, Self::Error> {
        Self::new(value.city, value.latitude, value.longitude)
    }
}

impl Default for WeatherLocation {
    fn default() -> Self {
        Self {
            city: "São Paulo, SP, Brasil".into(),
            latitude: -23.5505,
            longitude: -46.6333,
        }
    }
}

impl WeatherLocation {
    pub fn new(city: impl Into<String>, latitude: f64, longitude: f64) -> Result<Self, String> {
        let location = Self {
            city: city.into(),
            latitude,
            longitude,
        };
        location.validate()?;
        Ok(location)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.city.trim().is_empty()
            || self.city != self.city.trim()
            || self.city.chars().count() > 120
            || self.city.chars().any(|c| {
                c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
            })
            || !self.latitude.is_finite()
            || !self.longitude.is_finite()
            || !(-90.0..=90.0).contains(&self.latitude)
            || !(-180.0..=180.0).contains(&self.longitude)
        {
            return Err("Local de clima inválido: informe cidade e coordenadas válidas.".into());
        }
        Ok(())
    }

    fn request_path(&self) -> Result<String, String> {
        self.validate()?;
        // f64 Display emits only URI-unreserved numeric characters here; neither
        // the city label nor any user-supplied URL fragment enters the request.
        Ok(format!(
            "/v1/forecast?latitude={}&longitude={}&current=temperature_2m,is_day,weather_code,wind_speed_10m,precipitation&daily=sunrise,sunset&wind_speed_unit=kmh&precipitation_unit=mm&timezone=auto&timeformat=unixtime&forecast_days=1",
            self.latitude, self.longitude
        ))
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Snapshot {
    pub weather_code: u16,
    pub weather: Weather,
    pub temperature_c: f32,
    pub is_day: bool,
    pub observed_unix: u64,
    pub received_unix: u64,
    pub utc_offset_seconds: i32,
    pub wind_speed_kmh: Option<f32>,
    /// Current model interval's precipitation amount, not a rate in mm/hour.
    pub precipitation_mm: Option<f32>,
    pub sunrise_unix: Option<u64>,
    pub sunset_unix: Option<u64>,
}
impl Snapshot {
    pub fn description(&self) -> &'static str {
        match self.weather {
            Weather::Clear => "Céu limpo",
            Weather::Cloudy => "Nublado",
            Weather::Fog => "Neblina",
            Weather::Rain => "Chuva",
            Weather::Thunderstorm => "Trovoadas",
            Weather::Snow => "Neve",
        }
    }
    pub fn local_time(&self) -> String {
        let local = ((self.observed_unix % 86400) as i64 + i64::from(self.utc_offset_seconds))
            .rem_euclid(86400);
        format!("{:02}:{:02}", local / 3600, (local % 3600) / 60)
    }
    pub fn stale(&self) -> bool {
        self.age_seconds() > 7200
    }
    pub fn age_seconds(&self) -> u64 {
        unix_now().saturating_sub(self.observed_unix)
    }

    /// Real-clock phase in the renderer's [0, 600) cycle: sunrise=0,
    /// solar midpoint=180, sunset=348, local midnight=480. Continuous modulo 600.
    /// Offline, repeat the last known local solar hours; age/stale stay unchanged.
    /// Missing/polar solar data returns None so the renderer can use is_day.
    pub fn lighting_phase_at(&self, now: u64) -> Option<f64> {
        validate_optional(
            self.wind_speed_kmh,
            self.precipitation_mm,
            self.sunrise_unix,
            self.sunset_unix,
            self.observed_unix,
            self.utc_offset_seconds,
        )
        .ok()?;
        let rise = local_second(self.sunrise_unix?, self.utc_offset_seconds) as f64;
        let set = local_second(self.sunset_unix?, self.utc_offset_seconds) as f64;
        let time = local_second(now, self.utc_offset_seconds) as f64;
        let noon = (rise + set) * 0.5;
        Some(if time < rise {
            480.0 + 120.0 * time / rise
        } else if time < noon {
            180.0 * (time - rise) / (noon - rise)
        } else if time < set {
            180.0 + 168.0 * (time - noon) / (set - noon)
        } else {
            348.0 + 132.0 * (time - set) / (86400.0 - set)
        })
    }
}

fn local_second(time: u64, offset: i32) -> u64 {
    (i128::from(time) + i128::from(offset)).rem_euclid(86400) as u64
}

fn local_day(time: u64, offset: i32) -> i128 {
    (i128::from(time) + i128::from(offset)).div_euclid(86400)
}

fn validate_optional(
    wind: Option<f32>,
    rain: Option<f32>,
    rise: Option<u64>,
    set: Option<u64>,
    observed: u64,
    offset: i32,
) -> Result<(), String> {
    if wind.is_some_and(|v| !v.is_finite() || !(0.0..=500.0).contains(&v))
        || rain.is_some_and(|v| !v.is_finite() || !(0.0..=1000.0).contains(&v))
        || !(-50400..=50400).contains(&offset)
    {
        return Err("Vento ou precipitação inválidos.".into());
    }
    match (rise, set) {
        (None, None) => Ok(()),
        (Some(rise), Some(set))
            if rise > 0 && rise < set
            && local_day(rise, offset) == local_day(set, offset)
            // Current can still be the previous day's last 15-minute interval.
            && (0..=1).contains(&(local_day(rise, offset) - local_day(observed, offset))) =>
        {
            Ok(())
        }
        _ => Err("Horários solares inválidos ou incompatíveis com a observação.".into()),
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cache {
    version: u32,
    location: WeatherLocation,
    snapshot: CachedSnapshot,
}

// Weather is reconstructed from WMO, so a cache cannot pair incompatible enums/codes.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CachedSnapshot {
    weather_code: u16,
    temperature_c: f32,
    is_day: bool,
    observed_unix: u64,
    received_unix: u64,
    utc_offset_seconds: i32,
    #[serde(default)]
    wind_speed_kmh: Option<f32>,
    #[serde(default)]
    precipitation_mm: Option<f32>,
    #[serde(default)]
    sunrise_unix: Option<u64>,
    #[serde(default)]
    sunset_unix: Option<u64>,
}

impl CachedSnapshot {
    fn from_snapshot(s: &Snapshot) -> Result<Self, String> {
        if map_code(s.weather_code)? != s.weather {
            return Err("Condição do cache incompatível com o código meteorológico.".into());
        }
        Ok(Self {
            weather_code: s.weather_code,
            temperature_c: s.temperature_c,
            is_day: s.is_day,
            observed_unix: s.observed_unix,
            received_unix: s.received_unix,
            utc_offset_seconds: s.utc_offset_seconds,
            wind_speed_kmh: s.wind_speed_kmh,
            precipitation_mm: s.precipitation_mm,
            sunrise_unix: s.sunrise_unix,
            sunset_unix: s.sunset_unix,
        })
    }

    fn restore(&self, now: u64) -> Result<Snapshot, String> {
        validate_optional(
            self.wind_speed_kmh,
            self.precipitation_mm,
            self.sunrise_unix,
            self.sunset_unix,
            self.observed_unix,
            self.utc_offset_seconds,
        )?;
        if !self.temperature_c.is_finite()
            || !(-90.0..=65.0).contains(&self.temperature_c)
            || !(-50400..=50400).contains(&self.utc_offset_seconds)
            || self.observed_unix == 0
            || self.received_unix == 0
            || self.received_unix > now.saturating_add(300)
            || self.observed_unix > now.saturating_add(300)
            || self.observed_unix > self.received_unix.saturating_add(300)
            || self.received_unix.saturating_sub(self.observed_unix) > 7200
        {
            return Err("Cache meteorológico inválido; arquivo preservado.".into());
        }
        Ok(Snapshot {
            weather_code: self.weather_code,
            weather: map_code(self.weather_code)?,
            temperature_c: self.temperature_c,
            is_day: self.is_day,
            observed_unix: self.observed_unix,
            received_unix: self.received_unix,
            utc_offset_seconds: self.utc_offset_seconds,
            wind_speed_kmh: self.wind_speed_kmh,
            precipitation_mm: self.precipitation_mm,
            sunrise_unix: self.sunrise_unix,
            sunset_unix: self.sunset_unix,
        })
    }
}

fn read_cache(path: &Path) -> Result<Option<Cache>, String> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("Não foi possível ler o cache de clima: {e}")),
    };
    let mut bytes = Vec::new();
    file.take(MAX_CACHE as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_CACHE {
        return Err("Cache de clima grande demais; preservado.".into());
    }
    let cache: Cache = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Cache de clima inválido; preservado: {e}"))?;
    if cache.version != 1 {
        return Err("Versão do cache de clima incompatível; preservado.".into());
    }
    cache.location.validate()?;
    cache.snapshot.restore(unix_now())?;
    Ok(Some(cache))
}

/// Loads only this exact location. Old observations are retained with their
/// original timestamps (never made fresh by restart). Invalid files are errors.
pub fn load_cache(path: &Path, location: &WeatherLocation) -> Result<Option<Snapshot>, String> {
    location.validate()?;
    read_cache(path)?
        .filter(|c| c.location == *location)
        .map(|c| c.snapshot.restore(unix_now()))
        .transpose()
}

/// Atomically replaces a valid cache, or creates an absent one. Corrupt/future
/// files are preserved. Callers must serialize writers for the selected path
/// (the application's profile lock provides that contract).
pub fn save_cache(
    path: &Path,
    location: &WeatherLocation,
    snapshot: &Snapshot,
) -> Result<(), String> {
    location.validate()?;
    read_cache(path)?;
    let cached = CachedSnapshot::from_snapshot(snapshot)?;
    cached.restore(unix_now())?;
    let bytes = serde_json::to_vec_pretty(&Cache {
        version: 1,
        location: location.clone(),
        snapshot: cached,
    })
    .map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    for _ in 0..16 {
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let mut name = path.as_os_str().to_os_string();
        name.push(format!(".{}.{}.tmp", std::process::id(), serial));
        let temporary = PathBuf::from(name);
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("Cache de clima não salvo: {e}")),
        };
        let result = (|| {
            file.write_all(&bytes).map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
            drop(file);
            read_cache(path)?;
            fs::rename(&temporary, path).map_err(|e| e.to_string())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        return result;
    }
    Err("Não foi possível reservar um temporário para o cache de clima.".into())
}
fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn map_code(code: u16) -> Result<Weather, String> {
    Ok(match code {
        0 | 1 => Weather::Clear,
        2 | 3 => Weather::Cloudy,
        45 | 48 => Weather::Fog,
        51 | 53 | 55 | 56 | 57 | 61 | 63 | 65 | 66 | 67 | 80 | 81 | 82 => Weather::Rain,
        71 | 73 | 75 | 77 | 85 | 86 => Weather::Snow,
        95 | 96 | 99 => Weather::Thunderstorm,
        _ => return Err("Condição meteorológica não reconhecida.".into()),
    })
}
#[derive(Deserialize)]
struct Response {
    current: Current,
    utc_offset_seconds: i32,
    #[serde(default)]
    daily: Option<Daily>,
}
#[derive(Deserialize)]
struct Daily {
    // forecast_days=1: fixed arrays reject missing/misaligned/multi-day rows.
    sunrise: [Option<u64>; 1],
    sunset: [Option<u64>; 1],
}
#[derive(Deserialize)]
struct Current {
    time: u64,
    temperature_2m: f32,
    is_day: u8,
    weather_code: u16,
    #[serde(default)]
    wind_speed_10m: Option<f32>,
    #[serde(default)]
    precipitation: Option<f32>,
}
fn parse(body: &[u8], now: u64) -> Result<Snapshot, String> {
    if body.len() > MAX_BODY {
        return Err("Resposta meteorológica grande demais.".into());
    }
    let data: Response =
        serde_json::from_slice(body).map_err(|_| "Resposta meteorológica inválida.".to_owned())?;
    let c = data.current;
    let (sunrise, sunset) = data
        .daily
        .map_or((None, None), |d| (d.sunrise[0], d.sunset[0]));
    validate_optional(
        c.wind_speed_10m,
        c.precipitation,
        sunrise,
        sunset,
        c.time,
        data.utc_offset_seconds,
    )?;
    if !c.temperature_2m.is_finite()
        || !(-90.0..=65.0).contains(&c.temperature_2m)
        || c.is_day > 1
        || !(-50400..=50400).contains(&data.utc_offset_seconds)
        || c.time > now.saturating_add(300)
        || now.saturating_sub(c.time) > 7200
    {
        return Err("Dados meteorológicos inválidos ou desatualizados.".into());
    }
    Ok(Snapshot {
        weather_code: c.weather_code,
        weather: map_code(c.weather_code)?,
        temperature_c: c.temperature_2m,
        is_day: c.is_day == 1,
        observed_unix: c.time,
        received_unix: now,
        utc_offset_seconds: data.utc_offset_seconds,
        wind_speed_kmh: c.wind_speed_10m,
        precipitation_mm: c.precipitation,
        sunrise_unix: sunrise,
        sunset_unix: sunset,
    })
}

pub struct LocalWeather {
    pub enabled: bool,
    pub last_good: Option<Snapshot>,
    pub error: Option<String>,
    pub cache_error: Option<String>,
    location: WeatherLocation,
    generation: u64,
    pending_generation: u64,
    cache_path: Option<PathBuf>,
    cache_dirty: bool,
    next_cache_attempt: Instant,
    scene_changed: bool,
    lighting_minute: Option<u64>,
    pending: Option<Receiver<Result<Snapshot, String>>>,
    next_attempt: Instant,
    last_attempt: Option<Instant>,
    failures: u32,
}
impl LocalWeather {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            last_good: None,
            error: None,
            cache_error: None,
            location: WeatherLocation::default(),
            generation: 0,
            pending_generation: 0,
            cache_path: None,
            cache_dirty: false,
            next_cache_attempt: Instant::now(),
            scene_changed: false,
            lighting_minute: None,
            pending: None,
            next_attempt: Instant::now(),
            last_attempt: None,
            failures: 0,
        }
    }
    pub fn location(&self) -> &WeatherLocation {
        &self.location
    }

    /// Invalid input leaves the current selection untouched. A valid change
    /// immediately clears the old city's snapshot. Cache errors are separate.
    /// An in-flight worker is drained before another can start, even for A→B→A.
    pub fn set_location(&mut self, location: WeatherLocation) -> Result<(), String> {
        location.validate()?;
        if self.location == location {
            return Ok(());
        }
        self.location = location;
        self.generation = self.generation.wrapping_add(1);
        self.last_good = None;
        self.scene_changed = true;
        self.error = None;
        self.cache_error = None;
        self.cache_dirty = false;
        self.failures = 0;
        self.next_attempt = Instant::now();
        if let Some(path) = self.cache_path.clone() {
            let _ = self.set_cache_path(path);
        }
        Ok(())
    }

    /// Opt-in disk cache. Call after set_location at startup; only a matching
    /// validated observation is loaded. Loading never changes its timestamps.
    /// The path remains selected on error, but corrupt files cannot be replaced.
    pub fn set_cache_path(&mut self, path: impl Into<PathBuf>) -> Result<(), String> {
        let path = path.into();
        self.cache_path = Some(path.clone());
        self.cache_dirty = self.last_good.is_some();
        self.next_cache_attempt = Instant::now();
        let result = load_cache(&path, &self.location);
        self.cache_error = result.as_ref().err().cloned();
        if let Some(snapshot) = result?
            && self
                .last_good
                .as_ref()
                .is_none_or(|s| s.observed_unix < snapshot.observed_unix)
        {
            self.last_good = Some(snapshot);
            self.scene_changed = true;
            self.cache_dirty = false;
        }
        Ok(())
    }

    /// Explicitly retry a pending cache write (also called with backoff by poll).
    pub fn persist_cache(&mut self) -> Result<(), String> {
        let result = match (&self.cache_path, &self.last_good) {
            (Some(path), Some(snapshot)) => save_cache(path, &self.location, snapshot),
            _ => return Ok(()),
        };
        self.cache_dirty = result.is_err();
        self.cache_error = result.as_ref().err().cloned();
        self.next_cache_attempt = Instant::now() + COOLDOWN;
        result
    }

    pub fn loading(&self) -> bool {
        self.pending.is_some()
    }
    pub fn can_refresh(&self) -> bool {
        self.enabled
            && self.pending.is_none()
            && self.last_attempt.is_none_or(|t| t.elapsed() >= COOLDOWN)
    }
    pub fn refresh(&mut self, ctx: &eframe::egui::Context) {
        if self.can_refresh() {
            self.start(ctx);
        }
    }
    fn start(&mut self, ctx: &eframe::egui::Context) {
        let (tx, rx) = mpsc::sync_channel(1);
        let ctx = ctx.clone();
        let location = self.location.clone();
        self.pending_generation = self.generation;
        self.last_attempt = Some(Instant::now());
        self.next_attempt = Instant::now() + REFRESH;
        match std::thread::Builder::new()
            .name("local-weather".into())
            .spawn(move || {
                let result = fetch(&location);
                let _ = tx.send(result);
                ctx.request_repaint();
            }) {
            Ok(_) => self.pending = Some(rx),
            Err(_) => self.fail("Não foi possível iniciar a consulta de clima.".into()),
        }
    }
    fn fail(&mut self, error: String) {
        self.error = Some(error);
        self.failures = self.failures.saturating_add(1);
        self.next_attempt =
            Instant::now() + Duration::from_secs((60_u64 << self.failures.min(4)).min(900));
    }
    /// Poll even while paused; true requests one scene redraw without simulation progress.
    pub fn poll(&mut self, ctx: &eframe::egui::Context) -> bool {
        let result = self.pending.as_ref().and_then(|rx| match rx.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err("Consulta de clima interrompida.".into())),
        });
        let mut changed = std::mem::take(&mut self.scene_changed)
            || (result.is_some() && self.pending_generation == self.generation);
        if let Some(result) = result {
            self.pending = None;
            if self.pending_generation == self.generation {
                match result {
                    Ok(snapshot) => {
                        self.last_good = Some(snapshot);
                        self.error = None;
                        self.failures = 0;
                        self.next_attempt = Instant::now() + REFRESH;
                        self.cache_dirty = true;
                        self.next_cache_attempt = Instant::now();
                    }
                    Err(error) => self.fail(error),
                }
            }
        }
        if self.cache_dirty
            && self.cache_path.is_some()
            && Instant::now() >= self.next_cache_attempt
        {
            let _ = self.persist_cache();
        }
        if self.can_refresh() && Instant::now() >= self.next_attempt {
            self.start(ctx);
        }
        changed |= self.poll_lighting(unix_now());
        if self.enabled && self.last_good.is_some() {
            ctx.request_repaint_after(Duration::from_secs(60));
        }
        changed
    }
    fn poll_lighting(&mut self, now: u64) -> bool {
        let minute = (self.enabled && self.last_good.is_some()).then_some(now / 60);
        let changed = self.lighting_minute != minute;
        self.lighting_minute = minute;
        changed
    }
    /// Real-time solar phase; never advances world/journey time. None when
    /// disabled or solar data is unavailable (including older cache schemas).
    pub fn lighting_phase(&self) -> Option<f64> {
        if !self.enabled {
            return None;
        }
        self.last_good.as_ref()?.lighting_phase_at(unix_now())
    }
    pub fn atmosphere(&self) -> Option<(Weather, bool)> {
        self.last_good.as_ref().map(|s| (s.weather, s.is_day))
    }
    pub fn diagnostics(&self) -> serde_json::Value {
        serde_json::json!({"enabled":self.enabled, "loading":self.loading(), "status":self.status(),
            "city":self.location.city, "location":self.location, "source":"Open-Meteo",
            "age_seconds": self.last_good.as_ref().map(Snapshot::age_seconds),
            "stale": self.last_good.as_ref().is_some_and(Snapshot::stale),
            "lighting_phase": self.lighting_phase(),
            "snapshot":self.last_good, "error":self.error, "cache_error":self.cache_error})
    }
    pub fn status(&self) -> &'static str {
        if !self.enabled {
            "illustrative"
        } else if self.error.is_some() {
            "offline"
        } else if self.last_good.as_ref().is_some_and(Snapshot::stale) {
            "stale"
        } else if self.loading() {
            "loading"
        } else if self.last_good.is_some() {
            "ready"
        } else {
            "waiting"
        }
    }
}

fn fetch(location: &WeatherLocation) -> Result<Snapshot, String> {
    parse(&download(&location.request_path()?)?, unix_now())
}
#[cfg(not(windows))]
fn download(_path: &str) -> Result<Vec<u8>, String> {
    Err("Clima local disponível no Windows.".into())
}
#[cfg(windows)]
fn download(request_path: &str) -> Result<Vec<u8>, String> {
    use std::{
        ffi::c_void,
        ptr::{null, null_mut},
    };
    use windows_sys::Win32::Networking::WinHttp::*;
    struct Handle(*mut c_void);
    impl Handle {
        fn new(raw: *mut c_void) -> Result<Self, String> {
            if raw.is_null() {
                Err("Não foi possível conectar ao serviço de clima.".into())
            } else {
                Ok(Self(raw))
            }
        }
    }
    impl Drop for Handle {
        fn drop(&mut self) {
            // SAFETY: each non-null WinHTTP handle is owned here and closed exactly once.
            unsafe {
                WinHttpCloseHandle(self.0);
            }
        }
    }
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }
    fn check(ok: i32) -> Result<(), String> {
        if ok == 0 {
            Err(format!(
                "Consulta indisponível: {}",
                std::io::Error::last_os_error()
            ))
        } else {
            Ok(())
        }
    }
    let agent = wide("CPU-Express/0.3 weather");
    let host = wide("api.open-meteo.com");
    let path = wide(request_path);
    let verb = wide("GET");
    // SAFETY: NUL-terminated UTF-16 arguments stay alive; handles are checked and RAII-owned.
    // All calls are synchronous on this worker; buffers are sized correctly and never shared.
    unsafe {
        let session = Handle::new(WinHttpOpen(
            agent.as_ptr(),
            WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
            null(),
            null(),
            0,
        ))?;
        check(WinHttpSetTimeouts(session.0, 5000, 5000, 5000, 5000))?;
        let retries: u32 = 0;
        check(WinHttpSetOption(
            session.0,
            WINHTTP_OPTION_CONNECT_RETRIES,
            (&retries as *const u32).cast(),
            4,
        ))?;
        let connection = Handle::new(WinHttpConnect(session.0, host.as_ptr(), 443, 0))?;
        let request = Handle::new(WinHttpOpenRequest(
            connection.0,
            verb.as_ptr(),
            path.as_ptr(),
            null(),
            null(),
            null(),
            WINHTTP_FLAG_SECURE,
        ))?;
        let redirects = WINHTTP_OPTION_REDIRECT_POLICY_NEVER;
        check(WinHttpSetOption(
            request.0,
            WINHTTP_OPTION_REDIRECT_POLICY,
            (&redirects as *const u32).cast(),
            4,
        ))?;
        check(WinHttpSendRequest(request.0, null(), 0, null(), 0, 0, 0))?;
        check(WinHttpReceiveResponse(request.0, null_mut()))?;
        let mut status: u32 = 0;
        let mut size: u32 = 4;
        check(WinHttpQueryHeaders(
            request.0,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            null(),
            (&mut status as *mut u32).cast(),
            &mut size,
            null_mut(),
        ))?;
        if status != 200 {
            return Err(format!("Serviço de clima indisponível (HTTP {status})."));
        }
        let mut body = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if Instant::now() >= deadline {
                return Err("Tempo limite da consulta de clima.".into());
            }
            let mut buffer = [0_u8; 4096];
            let mut read = 0;
            check(WinHttpReadData(
                request.0,
                buffer.as_mut_ptr().cast(),
                buffer.len() as u32,
                &mut read,
            ))?;
            if read == 0 {
                break;
            }
            if body.len() + read as usize > MAX_BODY {
                return Err("Resposta meteorológica grande demais.".into());
            }
            body.extend_from_slice(&buffer[..read as usize]);
        }
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn solar_fixture() -> (serde_json::Value, u64) {
        // UTC-3, local midnight of day 10. Deliberately nonstandard 05:00/19:00.
        let midnight = 10 * 86400 + 10800;
        let mut body: serde_json::Value =
            serde_json::from_slice(&fixture(61, midnight + 43200)).unwrap();
        body["current"]["wind_speed_10m"] = serde_json::json!(27.5);
        body["current"]["precipitation"] = serde_json::json!(2.25);
        body["daily"] = serde_json::json!({
            "time": [midnight], "sunrise": [midnight + 5 * 3600], "sunset": [midnight + 19 * 3600]
        });
        (body, midnight)
    }

    #[test]
    fn solar_and_intensity_fields_parse_with_explicit_units_and_legacy_fallback() {
        let (body, midnight) = solar_fixture();
        let s = parse(&serde_json::to_vec(&body).unwrap(), midnight + 43230).unwrap();
        assert_eq!(s.wind_speed_kmh, Some(27.5));
        assert_eq!(s.precipitation_mm, Some(2.25));
        assert_eq!(s.sunrise_unix, Some(midnight + 5 * 3600));
        assert_eq!(s.sunset_unix, Some(midnight + 19 * 3600));
        let legacy = parse(&fixture(0, 86400), 86410).unwrap();
        assert!(legacy.wind_speed_kmh.is_none());
        assert!(legacy.precipitation_mm.is_none());
        assert!(legacy.lighting_phase_at(86410).is_none());
        let path = WeatherLocation::default().request_path().unwrap();
        for parameter in [
            "wind_speed_10m,precipitation",
            "daily=sunrise,sunset",
            "wind_speed_unit=kmh",
            "precipitation_unit=mm",
            "timeformat=unixtime",
        ] {
            assert!(path.contains(parameter));
        }
    }

    #[test]
    fn solar_phase_matches_ampere_anchors_and_is_continuous_across_dawn_and_midnight() {
        let (body, midnight) = solar_fixture();
        let s = parse(&serde_json::to_vec(&body).unwrap(), midnight + 43230).unwrap();
        for (seconds, expected) in [
            (0, 480.0),
            (5 * 3600, 0.0),
            (12 * 3600, 180.0),
            (19 * 3600, 348.0),
        ] {
            assert!((s.lighting_phase_at(midnight + seconds).unwrap() - expected).abs() < 1e-9);
            // Old data repeats last known local solar hours, without resetting age.
            assert!(
                (s.lighting_phase_at(midnight + seconds + 86400).unwrap() - expected).abs() < 1e-9
            );
        }
        let before = s.lighting_phase_at(midnight + 5 * 3600 - 1).unwrap();
        let after = s.lighting_phase_at(midnight + 5 * 3600 + 1).unwrap();
        assert!(before > 599.9 && after < 0.1);
        let before_midnight = s.lighting_phase_at(midnight + 86400 - 1).unwrap();
        assert!((before_midnight - 480.0).abs() < 0.1);
        for second in (0..86400).step_by(60) {
            let p = s.lighting_phase_at(midnight + second).unwrap();
            assert!(p.is_finite() && (0.0..600.0).contains(&p));
        }
        assert_eq!(s.observed_unix, midnight + 43200);
    }

    #[test]
    fn optional_fields_reject_invalid_values_but_allow_polar_missing_events() {
        let (body, midnight) = solar_fixture();
        for (field, invalid) in [
            ("wind_speed_10m", -1.0),
            ("wind_speed_10m", 501.0),
            ("precipitation", -0.1),
            ("precipitation", 1001.0),
        ] {
            let mut bad = body.clone();
            bad["current"][field] = serde_json::json!(invalid);
            assert!(parse(&serde_json::to_vec(&bad).unwrap(), midnight + 43230).is_err());
        }
        for daily in [
            serde_json::json!({"sunrise": [], "sunset": []}),
            serde_json::json!({"sunrise": [1,2], "sunset": [3,4]}),
            serde_json::json!({"sunrise": [0], "sunset": [1]}),
            serde_json::json!({"sunrise": [midnight + 100], "sunset": [midnight + 99]}),
            serde_json::json!({"sunrise": [midnight + 100], "sunset": [null]}),
            serde_json::json!({"sunrise": [midnight - 86400 + 100], "sunset": [midnight - 86400 + 200]}),
            serde_json::json!({"sunrise": [midnight + 100], "sunset": [midnight + 86400 + 200]}),
        ] {
            let mut bad = body.clone();
            bad["daily"] = daily;
            assert!(parse(&serde_json::to_vec(&bad).unwrap(), midnight + 43230).is_err());
        }
        let mut polar = body;
        polar["daily"] = serde_json::json!({"sunrise": [null], "sunset": [null]});
        let s = parse(&serde_json::to_vec(&polar).unwrap(), midnight + 43230).unwrap();
        assert!(s.lighting_phase_at(midnight).is_none());
    }

    #[test]
    fn lighting_redraw_uses_wall_clock_minute_without_mutating_observation() {
        let (body, midnight) = solar_fixture();
        let mut worker = LocalWeather::new(true);
        worker.last_good =
            Some(parse(&serde_json::to_vec(&body).unwrap(), midnight + 43230).unwrap());
        let before = serde_json::to_value(&worker.last_good).unwrap();
        assert!(worker.poll_lighting(midnight + 43230));
        assert!(!worker.poll_lighting(midnight + 43259));
        assert!(worker.poll_lighting(midnight + 43260));
        // Clock corrections also repaint once; no accumulated time/catch-up loop.
        assert!(worker.poll_lighting(midnight));
        assert_eq!(before, serde_json::to_value(&worker.last_good).unwrap());
        worker.enabled = false;
        assert!(worker.lighting_phase().is_none());
        worker.poll_lighting(midnight);
        assert!(!worker.poll_lighting(midnight + 60));
    }

    fn fixture(code: u16, time: u64) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "utc_offset_seconds": -10800,
            "current": {"time": time, "temperature_2m": 21.5, "is_day": 0, "weather_code": code}
        }))
        .unwrap()
    }
    #[test]
    fn parses_local_time_and_night() {
        let s = parse(&fixture(95, 86400), 86410).unwrap();
        assert_eq!(s.weather, Weather::Thunderstorm);
        assert!(!s.is_day);
        assert_eq!(s.local_time(), "21:00");
        assert_eq!(s.temperature_c, 21.5);
    }
    #[test]
    fn maps_all_supported_wmo_groups() {
        for (code, expected) in [
            (0, Weather::Clear),
            (1, Weather::Clear),
            (3, Weather::Cloudy),
            (48, Weather::Fog),
            (57, Weather::Rain),
            (82, Weather::Rain),
            (77, Weather::Snow),
            (86, Weather::Snow),
            (99, Weather::Thunderstorm),
        ] {
            assert_eq!(map_code(code).unwrap(), expected);
        }
        assert!(map_code(255).is_err());
    }
    #[test]
    fn rejects_missing_corrupt_stale_future_and_oversized() {
        for body in [
            b"{}".to_vec(),
            b"not-json".to_vec(),
            fixture(999, 86400),
            fixture(0, 1),
            fixture(0, 90000),
            vec![b' '; MAX_BODY + 1],
        ] {
            assert!(parse(&body, 86410).is_err());
        }
        let invalid = String::from_utf8(fixture(0, 86400))
            .unwrap()
            .replace("21.5", "null");
        assert!(parse(invalid.as_bytes(), 86410).is_err());
    }
    #[test]
    fn offline_keeps_last_good_and_throttles_retry() {
        let mut worker = LocalWeather::new(true);
        worker.last_good = Some(parse(&fixture(61, 86400), 86410).unwrap());
        worker.fail("offline".into());
        assert_eq!(worker.atmosphere(), Some((Weather::Rain, false)));
        assert!(worker.next_attempt > Instant::now() + COOLDOWN);
        worker.last_attempt = Some(Instant::now());
        assert!(!worker.can_refresh());
    }
    #[test]
    fn completed_worker_requests_redraw_and_recovers_from_error() {
        let mut worker = LocalWeather::new(true);
        worker.error = Some("offline".into());
        let (tx, rx) = mpsc::sync_channel(1);
        worker.pending = Some(rx);
        tx.send(Ok(parse(&fixture(3, 86400), 86410).unwrap()))
            .unwrap();
        assert!(worker.poll(&eframe::egui::Context::default()));
        assert_eq!(worker.atmosphere(), Some((Weather::Cloudy, false)));
        assert!(worker.error.is_none());
        assert!(!worker.loading());
        assert!(worker.next_attempt > Instant::now() + Duration::from_secs(890));
    }
    #[test]
    fn invalid_day_and_temperature_are_rejected() {
        let valid = String::from_utf8(fixture(0, 86400)).unwrap();
        for invalid in [
            valid.replace("21.5", "1000"),
            valid.replace("\"is_day\":0", "\"is_day\":2"),
            valid.replace("-10800", "-2147483648"),
        ] {
            assert!(parse(invalid.as_bytes(), 86410).is_err());
        }
    }
    #[test]
    fn disabled_never_starts_network() {
        let mut worker = LocalWeather::new(false);
        assert!(!worker.poll(&eframe::egui::Context::default()));
        assert!(!worker.loading());
        assert!(!worker.can_refresh());
        assert_eq!(worker.status(), "illustrative");
    }

    #[test]
    fn location_change_drains_old_worker_without_accepting_its_result() {
        let ctx = eframe::egui::Context::default();
        for old_result in [
            Ok(parse(&fixture(61, 86400), 86410).unwrap()),
            Err("old failure".into()),
        ] {
            let mut worker = LocalWeather::new(false);
            worker.last_good = Some(parse(&fixture(61, 86400), 86410).unwrap());
            let (tx, rx) = mpsc::sync_channel(1);
            worker.pending = Some(rx);
            worker
                .set_location(WeatherLocation::new("Lisboa", 38.72, -9.14).unwrap())
                .unwrap();
            assert!(worker.last_good.is_none());
            assert!(worker.loading());
            assert!(!worker.can_refresh());
            // Even returning to A does not make the original A request current.
            worker.set_location(WeatherLocation::default()).unwrap();
            tx.send(old_result).unwrap();
            assert!(worker.poll(&ctx));
            assert!(!worker.poll(&ctx));
            assert!(worker.last_good.is_none());
            assert!(worker.error.is_none());
            assert!(!worker.loading());
        }
    }

    #[test]
    fn location_switch_preserves_global_request_cooldown() {
        let mut worker = LocalWeather::new(true);
        worker.last_attempt = Some(Instant::now());
        worker
            .set_location(WeatherLocation::new("Lisboa", 38.72, -9.14).unwrap())
            .unwrap();
        assert!(!worker.can_refresh());
        assert!(worker.poll(&eframe::egui::Context::default()));
        assert!(!worker.loading());
    }

    #[test]
    fn coordinates_only_enter_fixed_endpoint_as_numeric_query_values() {
        let location = WeatherLocation::new("A&B?timezone=evil", 90.0, -180.0).unwrap();
        let path = location.request_path().unwrap();
        assert!(path.contains("latitude=90&longitude=-180&"));
        assert!(path.contains("timezone=auto"));
        assert!(!path.contains("evil"));
        let invalid = WeatherLocation {
            latitude: f64::NAN,
            ..location
        };
        assert!(invalid.request_path().is_err());
    }

    #[test]
    fn accepted_result_persists_and_cache_failure_retries_without_network() {
        let root = std::env::temp_dir().join(format!(
            "ce-weather-retry-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let blocked = root.join("blocked");
        fs::write(&blocked, b"obstruction").unwrap();
        let mut worker = LocalWeather::new(false);
        let _ = worker.set_cache_path(blocked.join("weather.json"));
        let sample = parse(&fixture(61, unix_now() - 60), unix_now()).unwrap();
        let (tx, rx) = mpsc::sync_channel(1);
        worker.pending = Some(rx);
        tx.send(Ok(sample.clone())).unwrap();
        let ctx = eframe::egui::Context::default();
        assert!(worker.poll(&ctx));
        assert!(worker.cache_error.is_some());
        assert!(worker.cache_dirty);
        assert!(worker.next_cache_attempt > Instant::now());
        assert!(worker.error.is_none());
        assert!(!worker.poll(&ctx));
        let restored = root.join("restored.json");
        worker.set_cache_path(&restored).unwrap();
        assert!(!worker.poll(&ctx));
        assert!(!worker.cache_dirty);
        assert!(worker.cache_error.is_none());
        assert_eq!(
            load_cache(&restored, worker.location())
                .unwrap()
                .unwrap()
                .received_unix,
            sample.received_unix
        );
        assert!(!worker.loading());
    }

    #[test]
    fn old_city_completion_cannot_replace_new_city_cache() {
        let path = std::env::temp_dir().join(format!(
            "ce-weather-generation-{}-{}.json",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let lisbon = WeatherLocation::new("Lisboa", 38.72, -9.14).unwrap();
        let sample = parse(&fixture(0, unix_now() - 60), unix_now()).unwrap();
        save_cache(&path, &lisbon, &sample).unwrap();
        let bytes = fs::read(&path).unwrap();
        let mut worker = LocalWeather::new(false);
        let (tx, rx) = mpsc::sync_channel(1);
        worker.pending = Some(rx);
        worker.set_cache_path(&path).unwrap();
        worker.set_location(lisbon).unwrap();
        tx.send(Ok(parse(&fixture(61, unix_now()), unix_now()).unwrap()))
            .unwrap();
        assert!(worker.poll(&eframe::egui::Context::default()));
        assert_eq!(worker.atmosphere(), Some((Weather::Clear, false)));
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
}
