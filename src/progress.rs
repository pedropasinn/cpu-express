//! Versioned local checkpoints. Never advances time while the app is closed.
use crate::simulation::{Journey, JourneySnapshot, World};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

/// v6 adds qualified services per line and the embarkation switch; the v5 progress
/// map remains unchanged. Legacy documents describe Aurora and get exact backups.
/// Optional fields decode older versions; current documents must supply them.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldCheckpoint {
    pub version: u32,
    /// Catalogue version of the writer. Absent before v4.
    #[serde(default)]
    pub catalog_version: u32,
    /// Catalogue line identifier. Absent before v4.
    #[serde(default)]
    pub line: String,
    pub journey: Checkpoint,
    pub service: crate::simulation::ServiceState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub economy: Option<crate::simulation::economy::EconomyState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<std::collections::BTreeMap<String, crate::simulation::world::LineProgress>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_services: Option<
        std::collections::BTreeMap<
            String,
            crate::simulation::ServiceState<crate::simulation::line_session::CatalogStationId>,
        >,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boarding_closed: Option<bool>,
}
fn decode_world(bytes: &[u8]) -> Result<World, String> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| format!("Jornada inválida, preservada: {e}"))?;
    if matches!(value.get("version").and_then(|v| v.as_u64()), Some(5..=6))
        && (value["economy"].get("pending_purchase").is_none()
            || value["journey"].get("station_wait").is_none())
    {
        return Err("Jornada v5 incompleta; arquivo preservado.".into());
    }
    // Defaults are only a migration convenience, never silent repair of a current save.
    if matches!(value.get("version").and_then(|v| v.as_u64()), Some(3..=6)) {
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
    if value.get("version").and_then(|v| v.as_u64()) == Some(6)
        && value["line_services"].as_object().is_some_and(|services| {
            services.values().any(|service| {
                service.get("automatic_contracts").is_none()
                    || service.get("branch").is_none()
                    || service["arrivals"].as_array().is_some_and(|arrivals| {
                        arrivals.iter().any(|a| a.get("cargo_returned").is_none())
                    })
            })
        })
    {
        return Err("Serviços v6 incompletos; arquivo preservado.".into());
    }
    let mut economy = None;
    let mut lines = None;
    let mut line_state = None;
    let (checkpoint, service) = match value.get("version").and_then(|v| v.as_u64()) {
        Some(1) => (
            serde_json::from_value::<Checkpoint>(value).map_err(|e| e.to_string())?,
            None,
        ),
        Some(version @ 2..=6) => {
            let saved: WorldCheckpoint =
                serde_json::from_value(value).map_err(|e| e.to_string())?;
            if version >= 4 {
                // A line or catalogue this edition cannot run is someone else's journey.
                if saved.catalog_version == 0
                    || saved.catalog_version > crate::simulation::catalog::CATALOG_VERSION
                {
                    return Err("Catálogo da jornada incompatível; arquivo preservado.".into());
                }
                if version < 6 && saved.line != crate::simulation::route::CIRCUIT_LINE_ID {
                    return Err(format!(
                        "A linha `{}` não pode ser percorrida por esta edição; arquivo preservado.",
                        saved.line
                    ));
                }
            } else if saved.catalog_version != 0 || !saved.line.is_empty() {
                return Err("Jornada com campos de outra versão; arquivo preservado.".into());
            }
            if version == 6 {
                line_state = Some((
                    saved.line,
                    saved
                        .line_services
                        .ok_or("Jornada v6 sem serviços por linha.")?,
                    saved
                        .boarding_closed
                        .ok_or("Jornada v6 sem controle de embarques.")?,
                ));
            } else if saved.line_services.is_some() || saved.boarding_closed.is_some() {
                return Err("Jornada antiga com campos v6; arquivo preservado.".into());
            }
            if version >= 5 {
                economy = Some(
                    saved
                        .economy
                        .ok_or("Jornada v5 sem economia; arquivo preservado.")?,
                );
                lines = Some(
                    saved
                        .lines
                        .ok_or("Jornada v5 sem progresso por linha; arquivo preservado.")?,
                );
            } else if saved.economy.is_some() || saved.lines.is_some() {
                return Err("Jornada antiga com campos v5; arquivo preservado.".into());
            }
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
        if economy.is_none()
            && service
                .composition
                .iter()
                .any(|w| !w.available(world.journey_snapshot().stations_visited))
        {
            return Err("Composição legada inválida; arquivo preservado.".into());
        }
        world.restore_service(service)?;
    }
    if let Some((active, services, boarding)) = line_state {
        world.restore_lines(active, lines.clone().unwrap(), services, boarding)?;
    } else if let Some(lines) = &lines
        && *lines != world.line_progress()
    {
        return Err("Progresso por linha divergente; arquivo preservado.".into());
    }
    if let Some(economy) = economy {
        if economy
            .cargo_by_kind
            .values()
            .try_fold(0_u64, |sum, value| sum.checked_add(*value))
            .is_none_or(|sum| sum > world.service_view().cargo_delivered)
        {
            return Err("Contadores de carga divergentes; arquivo preservado.".into());
        }
        // Do not silently repair mismatched duplicate livery ownership in a current save.
        if crate::simulation::objectives::Livery::ALL.iter().any(|l| {
            world.service_state().owns(*l)
                != economy.owns(&crate::simulation::economy::ItemId::livery(*l))
        }) {
            return Err("Posse de pinturas divergente; arquivo preservado.".into());
        }
        world.restore_economy(economy)?;
        let lines = lines.unwrap();
        if lines != world.line_progress() {
            return Err("Progresso por linha divergente; arquivo preservado.".into());
        }
        world.lines = lines;
    } else {
        world.migrate_economy();
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
pub const CURRENT_VERSION: u64 = 6;

/// The checkpoint that was valid immediately before the latest replacement.
pub fn previous_path(path: &Path) -> PathBuf {
    path.with_extension("previous.json")
}

/// Product start-up load. A damaged checkpoint is set aside byte for byte and the
/// journey resumes from the previous valid checkpoint, with a notice for the player.
/// Saves written by a newer version, unreadable files and oversized files are never
/// moved: they keep blocking normal saves exactly as [`load_world`] reports them.
/// The caller must hold the profile lock.
pub fn load_profile(path: &Path) -> Result<(Option<World>, Option<String>), String> {
    let error = match load_world(path) {
        Ok(world) => return settle_loaded_missions(path, world, None),
        Err(error) => error,
    };
    if fs::metadata(path).is_ok_and(|m| m.len() > 1_048_576) {
        return Err(error);
    }
    let Ok(damaged) = fs::read(path) else {
        return Err(error);
    };
    let newer = serde_json::from_slice::<serde_json::Value>(&damaged)
        .ok()
        .and_then(|value| value.get("version").and_then(|v| v.as_u64()))
        .is_some_and(|version| version > CURRENT_VERSION);
    if newer {
        return Err(error);
    }
    let previous = previous_path(path);
    let Ok(previous_bytes) = fs::read(&previous) else {
        return Err(error);
    };
    let Ok(world) = decode_world(&previous_bytes) else {
        return Err(error);
    };
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    // Renaming replaces an existing file on Windows, so only an unused name is accepted.
    let quarantine = (0..128)
        .map(|attempt| path.with_extension(format!("damaged-{nonce}-{attempt}.json")))
        .find(|candidate| !candidate.exists())
        .ok_or_else(|| format!("{error} Recuperação automática indisponível."))?;
    fs::rename(path, &quarantine)
        .map_err(|e| format!("{error} Recuperação automática indisponível: {e}"))?;
    let name = quarantine
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let mut notice = format!(
        "O arquivo da jornada estava danificado e foi preservado como {name}. A viagem foi retomada do checkpoint anterior."
    );
    // Restore the exact previous document so the profile is consistent before the next save.
    if let Err(e) = replace_file(path, &previous_bytes) {
        notice.push_str(&format!(
            " O arquivo principal ainda não foi regravado: {e}"
        ));
    }
    settle_loaded_missions(path, Some(world), Some(notice))
}

fn settle_loaded_missions(
    path: &Path,
    mut world: Option<World>,
    mut notice: Option<String>,
) -> Result<(Option<World>, Option<String>), String> {
    if let Some(w) = world
        .as_mut()
        .filter(|w| w.economy().legacy_mission_claim_pending)
    {
        let before = w.economy().balance;
        let result = economy_action(
            path,
            w,
            crate::simulation::economy::EconomyAction::SettleLegacyMissions,
        );
        let warning = match result {
            Ok(warning) => warning,
            Err(error) => {
                let message = format!(
                    "A viagem foi preservada; os marcos antigos ainda não foram pagos. Nova tentativa na próxima abertura. {error}"
                );
                notice = Some(
                    [notice, Some(message)]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .join(" "),
                );
                return Ok((world, notice));
            }
        };
        let message = format!(
            "Marcos antigos conferidos: +{} moedas, uma única vez.",
            w.economy().balance - before
        );
        notice = Some(
            [notice, Some(message), warning]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    Ok((world, notice))
}

/// Periodic profile save. Keeps the checkpoint being replaced as the recovery copy,
/// so one damaged write never leaves the profile without a valid journey.
/// `Ok(Some(_))` means the journey was saved but the recovery copy was not refreshed.
pub fn save_checkpoint(path: &Path, world: &World) -> Result<Option<String>, String> {
    // Validate the new document before touching the recovery copy.
    encode_world(world)?;
    let warning = match fs::read(path) {
        Ok(current) if current.len() <= 1_048_576 && decode_world(&current).is_ok() => {
            replace_file(&previous_path(path), &current).err().map(|e| {
                format!("Jornada salva, mas a cópia de recuperação não foi atualizada: {e}")
            })
        }
        _ => None,
    };
    save(path, world)?;
    Ok(warning)
}

/// Purchases/reveals/mission claims use this immediate writer, never the 30 s timer.
/// A recovery-copy warning is a successful commit; only Err rolls back memory.
pub fn transact_economy<T>(
    path: &Path,
    world: &mut World,
    change: impl FnOnce(&mut crate::simulation::economy::EconomyState) -> Result<T, String>,
) -> Result<(T, Option<String>), String> {
    let mut warning = None;
    let result = world.transact_economy(change, |candidate| {
        warning = save_checkpoint(path, candidate)?;
        Ok(())
    })?;
    Ok((result, warning))
}

pub fn economy_action(
    path: &Path,
    world: &mut World,
    action: crate::simulation::economy::EconomyAction,
) -> Result<Option<String>, String> {
    let mut warning = None;
    world.economy_action(action, |candidate| {
        warning = save_checkpoint(path, candidate)?;
        Ok(())
    })?;
    Ok(warning)
}

/// Persist an arrival before its events can be consumed. Failed writes restore
/// journey, services, RNG, wallet, train and notifications together.
pub fn advance_checkpoint(
    path: &Path,
    world: &mut World,
    train: &mut crate::simulation::Train,
    cpu: f32,
    seconds: f64,
    config: &crate::config::Config,
) -> Result<Option<String>, String> {
    if !world.may_arrive(seconds) {
        world.advance_expedition(train, cpu, seconds, config);
        return Ok(None);
    }
    let mut candidate = world.clone();
    let mut next_train = train.clone();
    candidate.advance_expedition(&mut next_train, cpu, seconds, config);
    let warning = if candidate.economy().next_transaction_id != world.economy().next_transaction_id
    {
        save_checkpoint(path, &candidate)?
    } else {
        None
    };
    *world = candidate;
    *train = next_train;
    Ok(warning)
}

/// Compatibility view for old tools; product callers must use load_world.
pub fn load(path: &Path) -> Result<Option<(Journey, f64)>, String> {
    load_world(path).map(|world| world.map(|w| (w.journey().clone(), w.station_wait)))
}

pub fn save(path: &Path, world: &World) -> Result<(), String> {
    // Refuse to replace a corrupt or future save, even if it changed after startup.
    load(path)?;
    let bytes = encode_world(world)?;
    // Keep the exact pre-migration document. Never overwrite an existing backup.
    if let Ok(original) = fs::read(path) {
        let value: serde_json::Value =
            serde_json::from_slice(&original).map_err(|e| e.to_string())?;
        if let Some(version @ 1..=5) = value.get("version").and_then(|v| v.as_u64()) {
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
    checkpoint.station_wait = world.station_wait;
    checkpoint.restore()?;
    world
        .service_snapshot()
        .validate(world.journey_snapshot())?;
    let saved = WorldCheckpoint {
        version: CURRENT_VERSION as u32,
        catalog_version: crate::simulation::catalog::CATALOG_VERSION,
        line: world.active_line_id().to_owned(),
        journey: checkpoint,
        service: world.service_snapshot().clone(),
        economy: Some(world.economy().clone()),
        lines: Some(world.line_progress()),
        line_services: Some(
            world
                .sessions
                .iter()
                .map(|(id, s)| (id.clone(), s.service().clone()))
                .collect(),
        ),
        boarding_closed: Some(world.boarding_closed()),
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
    replace_file(path, bytes).map_err(|e| format!("Jornada não salva: {e}"))
}

/// Complete temporary file, then one rename: a failure at any step leaves the
/// destination untouched and removes the temporary file.
fn replace_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
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
        })?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
