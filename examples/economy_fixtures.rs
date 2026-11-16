//! Local fixture exporter. Refuses replacement; never opens the real profile.
use cpu_express::{
    progress::{self, Checkpoint, WorldCheckpoint},
    simulation::{
        Journey, ServiceReward, World,
        economy::{CrateKind, EconomyAction},
    },
};
use std::{fs::OpenOptions, io::Write, path::Path};

fn create(path: &Path, value: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    file.sync_all()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let folder = std::env::args()
        .nth(1)
        .ok_or("Informe uma pasta nova para fixtures.")?;
    let folder = Path::new(&folder);
    std::fs::create_dir_all(folder)?;
    let mut world = World::with_seed(42);
    world.restore_journey(Journey::restore_checkpoint(42, 40_000.0, 2000.0)?);
    let mut service = world.service_state().clone();
    service.passengers_boarded = 20;
    service.passengers_delivered = 20;
    service.cargo_delivered = 20;
    service.contracts_completed = 2;
    service.credits = 240;
    service.collection = vec![
        ServiceReward::PassengerPostcard,
        ServiceReward::CargoPostcard,
    ];
    world.restore_service(service)?;
    let journey = Checkpoint::capture(world.journey_snapshot());
    create(&folder.join("v1.json"), &serde_json::to_value(&journey)?)?;
    let checkpoint = WorldCheckpoint {
        version: 4,
        catalog_version: 1,
        line: "aurora-v1".into(),
        journey,
        service: world.service_state().clone(),
        economy: None,
        lines: None,
        line_services: None,
        boarding_closed: None,
    };
    let v4 = serde_json::to_value(checkpoint)?;
    create(&folder.join("v4.json"), &v4)?;
    let mut v3 = v4.clone();
    v3["version"] = 3.into();
    v3.as_object_mut().unwrap().remove("line");
    v3.as_object_mut().unwrap().remove("catalog_version");
    create(&folder.join("v3.json"), &v3)?;
    let mut v2 = v3;
    v2["version"] = 2.into();
    v2["service"]
        .as_object_mut()
        .unwrap()
        .remove("automatic_contracts");
    v2["service"].as_object_mut().unwrap().remove("branch");
    create(&folder.join("v2.json"), &v2)?;
    let mut migrated = progress::load_world(&folder.join("v4.json"))?.ok_or("Fixture ausente")?;
    migrated.economy_action(
        EconomyAction::BuyCrate {
            kind: CrateKind::GeneralYard,
            free: true,
        },
        |_| Ok(()),
    )?;
    // Capture the product schema into new fixture files without replacing a profile.
    let capture = |world: &World| -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut journey = Checkpoint::capture(world.journey_snapshot());
        journey.station_wait = world.station_wait();
        Ok(serde_json::to_value(WorldCheckpoint {
            version: 5,
            catalog_version: 1,
            line: "aurora-v1".into(),
            journey,
            service: world.service_state().clone(),
            economy: Some(world.economy().clone()),
            lines: Some(world.line_progress()),
            line_services: None,
            boarding_closed: None,
        })?)
    };
    create(&folder.join("v5-pending.json"), &capture(&migrated)?)?;
    create(
        &folder.join("v5-new.json"),
        &capture(&World::with_seed(42))?,
    )?;
    println!("Fixtures gravadas em {}", folder.display());
    Ok(())
}
