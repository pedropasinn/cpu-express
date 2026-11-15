use super::*;
impl CpuExpressApp {
    pub(super) fn storage_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Exportar viagem…").clicked() {
                let result = (|| {
                    if let Some(path) = platform::choose_journey_file(true)? {
                        crate::progress::save(&path, &self.world)?;
                        return Ok(Some("Viagem exportada.".to_owned()));
                    }
                    Ok(None)
                })();
                match result {
                    Ok(Some(message)) => self.notification = Some(message),
                    Ok(None) => {}
                    Err(error) => self.progress_notice = Some(error),
                }
                self.clock = SimulationClock::new(self.opened.elapsed());
                self.sync_clock();
            }
            if ui
                .add_enabled(
                    self.progress_path.is_some(),
                    egui::Button::new("Importar viagem…"),
                )
                .on_hover_text(
                    "Substitui a viagem atual após validar o arquivo e guardar backup da anterior.",
                )
                .clicked()
            {
                let result = (|| {
                    let Some(source) = platform::choose_journey_file(false)? else {
                        return Ok(None);
                    };
                    let destination = self
                        .progress_path
                        .as_ref()
                        .ok_or("Perfil temporário não permite importar.".to_owned())?;
                    import_preserving_session(destination, &source, &self.world).map(Some)
                })();
                match result {
                    Ok(Some(world)) => {
                        self.world = world;
                        self.seen_rewards = (
                            self.world.service_snapshot().owned_rewards.len(),
                            self.world.service_snapshot().collection.len(),
                        );
                        self.world.set_paused(self.suspended());
                        self.config.visual.coaches =
                            self.world.service_snapshot().composition.map(coach_style);
                        if !self
                            .world
                            .service_snapshot()
                            .owns(self.config.visual.livery)
                        {
                            self.config.visual.livery = Default::default();
                        }
                        use crate::rendering::EngineStyle;
                        let requirement = self.config.visual.engine.required_reward();
                        if requirement
                            .is_some_and(|r| !self.world.service_snapshot().collection.contains(&r))
                        {
                            self.config.visual.engine = EngineStyle::Steam;
                        }
                        self.train = Train::new(&self.config);
                        self.travel_log.clear();
                        self.travel_notice = None;
                        self.progress_notice = None;
                        self.preferences_changed();
                        // Refresh the recovery copy so it no longer describes the replaced trip.
                        self.persist_progress(true);
                        self.notification =
                            Some("Viagem importada. A anterior foi preservada em backup.".into());
                    }
                    Ok(None) => {}
                    Err(error) => self.progress_notice = Some(error),
                }
                self.clock = SimulationClock::new(self.opened.elapsed());
                self.sync_clock();
            }
        });
        game::hint(
            ui,
            "Importar substitui a viagem atual e preserva um backup local. Arquivos incompatíveis são recusados.",
        );
    }
}

fn import_preserving_session(
    destination: &std::path::Path,
    source: &std::path::Path,
    current: &World,
) -> Result<World, String> {
    crate::progress::load_world(source)?.ok_or("Arquivo de origem não encontrado.".to_owned())?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let backup = destination.with_extension(format!("before-import-session-{nonce}.json"));
    crate::progress::save(&backup, current)?;
    crate::progress::import_world(destination, source)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn import_preserves_live_session_even_when_disk_save_is_corrupt() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("ce-live-import-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let source = dir.join("source.json");
        let destination = dir.join("current.json");
        crate::progress::save(&source, &World::default()).unwrap();
        std::fs::write(&destination, b"broken save").unwrap();
        let config = Config::default();
        let mut live = World::default();
        let mut train = Train::new(&config);
        for _ in 0..300 {
            live.advance_expedition(&mut train, 50.0, 1.0, &config);
        }
        let restored = import_preserving_session(&destination, &source, &live).unwrap();
        assert_eq!(restored.journey_snapshot().stations_visited, 0);
        let files: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        let backup = files
            .iter()
            .find(|p| p.to_string_lossy().contains("before-import-session"))
            .unwrap();
        let saved = crate::progress::load_world(backup).unwrap().unwrap();
        assert_eq!(saved.service_snapshot(), live.service_snapshot());
        assert_eq!(
            saved.journey_snapshot().total_distance_units,
            live.journey_snapshot().total_distance_units
        );
        assert!(
            files
                .iter()
                .any(|p| std::fs::read(p).unwrap() == b"broken save")
        );
        for file in files {
            std::fs::remove_file(file).unwrap();
        }
        std::fs::remove_dir(dir).unwrap();
    }
}
