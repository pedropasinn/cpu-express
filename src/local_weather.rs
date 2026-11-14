//! Fixed-city weather. All network work stays off the UI/render thread.
use crate::rendering::Weather;
use serde::{Deserialize, Serialize};
use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const REFRESH: Duration = Duration::from_secs(900);
const COOLDOWN: Duration = Duration::from_secs(60);
const MAX_BODY: usize = 65_536;
const PATH: &str = "/v1/forecast?latitude=-23.5505&longitude=-46.6333&current=temperature_2m,is_day,weather_code&timezone=America%2FSao_Paulo&timeformat=unixtime&forecast_days=1";

#[derive(Clone, Debug, Serialize)]
pub struct Snapshot {
    pub weather_code: u16,
    pub weather: Weather,
    pub temperature_c: f32,
    pub is_day: bool,
    pub observed_unix: u64,
    pub received_unix: u64,
    pub utc_offset_seconds: i32,
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
        let local =
            (self.observed_unix as i64 + i64::from(self.utc_offset_seconds)).rem_euclid(86400);
        format!("{:02}:{:02}", local / 3600, (local % 3600) / 60)
    }
    pub fn stale(&self) -> bool {
        unix_now().saturating_sub(self.observed_unix) > 7200
    }
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
}
#[derive(Deserialize)]
struct Current {
    time: u64,
    temperature_2m: f32,
    is_day: u8,
    weather_code: u16,
}
fn parse(body: &[u8], now: u64) -> Result<Snapshot, String> {
    if body.len() > MAX_BODY {
        return Err("Resposta meteorológica grande demais.".into());
    }
    let data: Response =
        serde_json::from_slice(body).map_err(|_| "Resposta meteorológica inválida.".to_owned())?;
    let c = data.current;
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
    })
}

pub struct LocalWeather {
    pub enabled: bool,
    pub last_good: Option<Snapshot>,
    pub error: Option<String>,
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
            pending: None,
            next_attempt: Instant::now(),
            last_attempt: None,
            failures: 0,
        }
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
        self.last_attempt = Some(Instant::now());
        self.next_attempt = Instant::now() + REFRESH;
        match std::thread::Builder::new()
            .name("local-weather".into())
            .spawn(move || {
                let result = fetch();
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
        let changed = result.is_some();
        if let Some(result) = result {
            self.pending = None;
            match result {
                Ok(snapshot) => {
                    self.last_good = Some(snapshot);
                    self.error = None;
                    self.failures = 0;
                    self.next_attempt = Instant::now() + REFRESH;
                }
                Err(error) => self.fail(error),
            }
        }
        if self.enabled && self.pending.is_none() && Instant::now() >= self.next_attempt {
            self.start(ctx);
        }
        changed
    }
    pub fn atmosphere(&self) -> Option<(Weather, bool)> {
        self.last_good.as_ref().map(|s| (s.weather, s.is_day))
    }
    pub fn diagnostics(&self) -> serde_json::Value {
        serde_json::json!({"enabled":self.enabled, "loading":self.loading(), "status":self.status(),
            "city":"São Paulo, SP, Brasil", "source":"Open-Meteo", "snapshot":self.last_good, "error":self.error})
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

fn fetch() -> Result<Snapshot, String> {
    parse(&download()?, unix_now())
}
#[cfg(not(windows))]
fn download() -> Result<Vec<u8>, String> {
    let _ = PATH;
    Err("Clima local disponível no Windows.".into())
}
#[cfg(windows)]
fn download() -> Result<Vec<u8>, String> {
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
    let path = wide(PATH);
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
}
