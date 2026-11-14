//! Versioned local checkpoints. Never advances time while the app is closed.
use crate::simulation::{Journey, JourneySnapshot, World};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub version: u32,
    pub route: String,
    pub seed: u64,
    pub distance: f64,
    pub active_seconds: f64,
    #[serde(default)]
    pub station_wait: f64,
}

impl Checkpoint {
    pub fn capture(snapshot: JourneySnapshot) -> Self {
        Self {
            version: 1,
            route: "aurora-v1".into(),
            seed: snapshot.seed,
            distance: snapshot.total_distance_units,
            active_seconds: snapshot.active_seconds,
            station_wait: 0.0,
        }
    }

    pub fn restore(&self) -> Result<Journey, String> {
        if self.version != 1
            || self.route != "aurora-v1"
            || !self.station_wait.is_finite()
            || !(0.0..=8.0).contains(&self.station_wait)
        {
            return Err("Versão da jornada incompatível; arquivo preservado.".into());
        }
        Journey::restore_checkpoint(self.seed, self.distance, self.active_seconds)
    }
}

pub fn path_for(config: &Path) -> PathBuf {
    config.with_file_name(format!(
        "{}.journey.json",
        config.file_stem().unwrap_or_default().to_string_lossy()
    ))
}

/// Windows file sharing gives a crash-safe exclusive writer: the OS releases
/// the handle on exit, so an old empty lock file never blocks the next launch.
#[cfg(windows)]
pub fn lock_profile(config: &Path) -> Result<fs::File, String> {
    use std::os::windows::fs::OpenOptionsExt;
    if let Some(parent) = config.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .share_mode(0)
        .open(config.with_extension("profile.lock"))
        .map_err(|e| format!("Este perfil já está em uso ou não pode ser aberto: {e}"))
}

/// v3 adds dispatch choice and safe returns; v1/v2 migrate with exact backups.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldCheckpoint {
    pub version: u32,
    pub journey: Checkpoint,
    pub service: crate::simulation::ServiceState,
}
fn decode_world(bytes: &[u8]) -> Result<World, String> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| format!("Jornada inválida, preservada: {e}"))?;
    // Defaults are only a migration convenience, never silent repair of a current save.
    if value.get("version").and_then(|v| v.as_u64()) == Some(3) {
        let service = &value["service"];
        if service.get("automatic_contracts").is_none()
            || service.get("branch").is_none()
            || service
                .get("arrivals")
                .and_then(|a| a.as_array())
                .is_some_and(|arrivals| arrivals.iter().any(|a| a.get("cargo_returned").is_none()))
        {
            return Err("Jornada v3 incompleta; arquivo preservado.".into());
        }
    }
    let (checkpoint, service) = match value.get("version").and_then(|v| v.as_u64()) {
        Some(1) => (
            serde_json::from_value::<Checkpoint>(value).map_err(|e| e.to_string())?,
            None,
        ),
        Some(2 | 3) => {
            let saved: WorldCheckpoint =
                serde_json::from_value(value).map_err(|e| e.to_string())?;
            (saved.journey, Some(saved.service))
        }
        _ => return Err("Versão da jornada incompatível; arquivo preservado.".into()),
    };
    let journey = checkpoint.restore()?;
    if checkpoint.station_wait > 0.0 && journey.snapshot().at_station.is_none() {
        return Err("Parada fora da estação; arquivo preservado.".into());
    }
    let mut world = World::with_seed(checkpoint.seed);
    world.restore_journey(journey);
    world.set_station_wait(checkpoint.station_wait);
    if let Some(service) = service {
        world.restore_service(service)?;
    }
    Ok(world)
}
/// Use this in the app: restoring just Journey intentionally cannot restore services.
pub fn load_world(path: &Path) -> Result<Option<World>, String> {
    if fs::metadata(path).is_ok_and(|m| m.len() > 1_048_576) {
        return Err("Jornada grande demais; arquivo preservado.".into());
    }
    match fs::read(path) {
        Ok(bytes) => {
            if bytes.len() > 1_048_576 {
                return Err("Jornada grande demais; arquivo preservado.".into());
            }
            decode_world(&bytes).map(Some)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Não foi possível ler a jornada: {e}")),
    }
}
/// Compatibility view for old tools; product callers must use load_world.
pub fn load(path: &Path) -> Result<Option<(Journey, f64)>, String> {
    load_world(path).map(|world| world.map(|w| (w.journey().clone(), w.station_wait())))
}

pub fn save(path: &Path, world: &World) -> Result<(), String> {
    // Refuse to replace a corrupt or future save, even if it changed after startup.
    load(path)?;
    let bytes = encode_world(world)?;
    // Keep the exact pre-migration document. Never overwrite an existing backup.
    if let Ok(original) = fs::read(path) {
        let value: serde_json::Value =
            serde_json::from_slice(&original).map_err(|e| e.to_string())?;
        if let Some(version @ (1 | 2)) = value.get("version").and_then(|v| v.as_u64()) {
            let backup = path.with_extension(format!("v{version}.backup.json"));
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&backup)
            {
                Ok(mut file) => {
                    file.write_all(&original)
                        .and_then(|()| file.sync_all())
                        .map_err(|e| e.to_string())?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    if fs::read(&backup).map_err(|e| e.to_string())? != original {
                        return Err(format!(
                            "Backup v{version} existente difere do original; migração cancelada."
                        ));
                    }
                }
                Err(e) => return Err(e.to_string()),
            }
        }
    }
    write_atomic(path, &bytes)
}

fn encode_world(world: &World) -> Result<Vec<u8>, String> {
    let mut checkpoint = Checkpoint::capture(world.journey_snapshot());
    checkpoint.station_wait = world.station_wait();
    checkpoint.restore()?;
    world
        .service_snapshot()
        .validate(world.journey_snapshot())?;
    let saved = WorldCheckpoint {
        version: 3,
        journey: checkpoint,
        service: world.service_snapshot().clone(),
    };
    let bytes = serde_json::to_vec_pretty(&saved).map_err(|e| e.to_string())?;
    decode_world(&bytes)?;
    Ok(bytes)
}

/// Explicit import is the only path allowed to replace an incompatible destination.
/// The caller must hold the destination profile lock, as for normal saves.
pub fn import_world(destination: &Path, source: &Path) -> Result<World, String> {
    let world =
        load_world(source)?.ok_or_else(|| "Arquivo de origem não encontrado.".to_owned())?;
    let bytes = encode_world(&world)?;
    if let (Ok(a), Ok(b)) = (fs::canonicalize(destination), fs::canonicalize(source))
        && a == b
    {
        return Ok(world);
    }
    match fs::read(destination) {
        Ok(original) => {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let mut preserved = false;
            for attempt in 0..128 {
                let backup = destination.with_extension(format!(
                    "before-import-{nonce}-{}-{attempt}.backup.json",
                    std::process::id()
                ));
                match fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&backup)
                {
                    Ok(mut file) => {
                        file.write_all(&original)
                            .and_then(|()| file.sync_all())
                            .map_err(|e| {
                                format!("Backup não concluído; importação cancelada: {e}")
                            })?;
                        preserved = true;
                        break;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(e) => {
                        return Err(format!("Backup indisponível; importação cancelada: {e}"));
                    }
                }
            }
            if !preserved {
                return Err("Nomes de backup ocupados; importação cancelada.".into());
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("Não foi possível preservar o destino: {e}")),
    }
    write_atomic(destination, &bytes)?;
    Ok(world)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let (temporary, mut file) = (0..128)
        .find_map(|attempt| {
            let temporary = path.with_extension(format!("{}.{attempt}.tmp", std::process::id()));
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
            {
                Ok(file) => Some(Ok((temporary, file))),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => None,
                Err(error) => Some(Err(error)),
            }
        })
        .unwrap_or_else(|| {
            Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "Temporary filenames occupied",
            ))
        })
        .map_err(|error| error.to_string())?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|e| format!("Jornada não salva: {e}"))
}
