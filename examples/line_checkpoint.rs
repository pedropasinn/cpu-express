//! Export a deterministic v6 fixture to a new, explicit path. No real profile access.
use cpu_express::{
    config::Config,
    progress,
    simulation::{Train, World},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("Informe um caminho novo para a fixture v6.")?,
    );
    if path.exists() {
        return Err("Destino existente; nada foi alterado.".into());
    }
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::with_seed(42);
    world.switch_line("curitiba-paranagua", |_| Ok(()))?;
    for _ in 0..1300 {
        world.advance_expedition(&mut train, 40.0, 1.0, &config);
    }
    progress::save(&path, &world)?;
    println!("Fixture v6 gravada em {}", path.display());
    Ok(())
}
