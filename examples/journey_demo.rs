//! Run the real travel domain without a GPU, metrics collector or external crate.
//!
//! `cargo run --example journey_demo -- 42 1080`
//! `rustc --edition 2024 examples/journey_demo.rs -o journey-demo`
//! `./journey-demo 42 1080`

// The route reads its authored content from the catalogue.
#[path = "../src/simulation/catalog.rs"]
pub mod catalog;
#[path = "../src/simulation/journey.rs"]
pub mod journey;
#[path = "../src/simulation/route.rs"]
pub mod route;
#[path = "../src/simulation/timing.rs"]
pub mod timing;

use std::{env, process::ExitCode};

use journey::{Achievement, Journey, JourneySnapshot, TravelEvent};
use route::DEFAULT_JOURNEY_SEED;
use timing::AdvanceOutcome;

const STEP_SECONDS: f64 = 0.25;
const SIMULATED_UNITS_PER_SECOND: f64 = 55.0;
const DEFAULT_DURATION_SECONDS: u32 = 18 * 60;
const MAX_DURATION_SECONDS: u32 = 24 * 60 * 60;
const USAGE: &str = "Uso: journey_demo [seed_decimal] [segundos_simulados]\n\
    seed_decimal: inteiro de 0 a 18446744073709551615; padrão: 4850470395768754258.\n\
    segundos_simulados: inteiro de 1 a 86400; padrão: 1080 (18 minutos virtuais).\n\
    -h ou --help: mostra esta ajuda.\n\
    Exemplo Cargo: cargo run --example journey_demo -- 42 1080\n\
    Exemplo autônomo: rustc --edition 2024 examples/journey_demo.rs -o journey-demo\n\
                      ./journey-demo 42 1080\n";

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments.len() == 1 && matches!(arguments[0].as_str(), "-h" | "--help") {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let (seed, duration_seconds) = match parse_arguments(&arguments) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };

    run_demo(seed, duration_seconds);
    ExitCode::SUCCESS
}

fn parse_arguments(arguments: &[String]) -> Result<(u64, u32), &'static str> {
    if arguments.len() > 2 {
        return Err("Forneça no máximo uma seed e uma duração.");
    }
    let seed = match arguments.first() {
        Some(value) if decimal_digits(value) => value
            .parse::<u64>()
            .map_err(|_| "A seed deve caber em um inteiro u64.")?,
        Some(_) => return Err("A seed deve conter somente algarismos decimais."),
        None => DEFAULT_JOURNEY_SEED,
    };
    let duration = match arguments.get(1) {
        Some(value) if decimal_digits(value) => value
            .parse::<u32>()
            .map_err(|_| "A duração deve ser um inteiro de 1 a 86400 segundos simulados.")?,
        Some(_) => return Err("A duração deve conter somente algarismos decimais."),
        None => DEFAULT_DURATION_SECONDS,
    };
    if !(1..=MAX_DURATION_SECONDS).contains(&duration) {
        return Err("A duração deve estar entre 1 e 86400 segundos simulados.");
    }
    Ok((seed, duration))
}

fn decimal_digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn run_demo(seed: u64, duration_seconds: u32) {
    let mut journey = Journey::from_seed(seed);
    println!("CPU EXPRESS | MODO SIMULADO");
    println!(
        "Entrada fictícia: {SIMULATED_UNITS_PER_SECOND:.0} unidades/s, passos de {STEP_SECONDS:.2} s."
    );
    println!("Unidades de viagem são ficcionais; não representam quilômetros reais.");
    println!("Esta demonstração usa tempo virtual e não coleta métricas do computador.");
    println!("Seed: {seed} | Duração ativa: {duration_seconds} segundos simulados");
    println!(
        "Rota: {} | {:.0} unidades por circuito",
        journey.route().name(),
        journey.route().length_units()
    );
    for segment in journey.route().segments() {
        println!(
            "  {} -> {} | {:.0} unidades | bioma {:?}",
            segment.from.name(),
            segment.to.name(),
            segment.length_units,
            segment.biome_hint
        );
    }
    println!();
    print_progress(journey.snapshot());

    let total_steps = duration_seconds * 4;
    for step in 1..=total_steps {
        if step == total_steps / 2 + 1 {
            demonstrate_pause_and_resume(&mut journey);
        }
        let outcome = journey.advance(STEP_SECONDS, SIMULATED_UNITS_PER_SECOND * STEP_SECONDS);
        assert!(matches!(outcome, AdvanceOutcome::Advanced { .. }));
        print_events(&mut journey);
        if step % (120 * 4) == 0 || step == total_steps {
            print_progress(journey.snapshot());
        }
    }

    let final_state = journey.snapshot();
    let discoveries = final_state
        .discovered_stations
        .iter()
        .filter(|discovered| **discovered)
        .count();
    println!("\nRESUMO SIMULADO");
    println!(
        "Tempo ativo: {:.2} s | Distância acumulada: {:.2} unidades | Voltas: {}",
        final_state.active_seconds, final_state.total_distance_units, final_state.completed_laps
    );
    println!(
        "Descobertas: {discoveries}/{} | Chegadas: {} | Fila: {} | Eventos descartados: {}",
        route::STATION_COUNT,
        final_state.stations_visited,
        final_state.pending_events,
        final_state.dropped_events
    );
    println!(
        "Conquistas: primeira estação={} | primeiro circuito={} | longa viagem={}",
        final_state.achievements.first_station,
        final_state.achievements.first_circuit,
        final_state.achievements.long_journey
    );
}

fn demonstrate_pause_and_resume(journey: &mut Journey) {
    let before = journey.snapshot();
    journey.set_paused(true);
    let pause = journey.advance(STEP_SECONDS, SIMULATED_UNITS_PER_SECOND * STEP_SECONDS);
    assert_eq!(pause, AdvanceOutcome::Paused);
    journey.set_paused(false);
    assert_eq!(journey.snapshot(), before);
    println!("[PAUSA] {pause:?}: distância, tempo ativo e fila preservados.");

    let suspension = journey.advance(3_600.0, SIMULATED_UNITS_PER_SECOND * 3_600.0);
    assert_eq!(suspension, AdvanceOutcome::Suspended);
    assert_eq!(journey.snapshot(), before);
    println!(
        "[SLEEP/RESUME] 3600 s ignorados ({suspension:?}): snapshot preservado; próximos passos ativos retomam a viagem."
    );
}

fn print_progress(state: JourneySnapshot) {
    println!(
        "[PROGRESSO] ativo={:.0}s | total={:.1}u | circuito={:.1}% | próxima={} em {:.1}u | bioma={:?}",
        state.active_seconds,
        state.total_distance_units,
        state.route_progress * 100.0,
        state.next_station.name,
        state.distance_to_next_station_units,
        state.route_segment.biome_hint
    );
}

fn print_events(journey: &mut Journey) {
    while let Some(event) = journey.pop_event() {
        match event {
            TravelEvent::StationArrived {
                station,
                visit,
                distance_units,
            } => println!(
                "[CHEGADA] {} | visita {visit} | {distance_units:.1}u acumuladas",
                station.name()
            ),
            TravelEvent::StationDeparted {
                station,
                distance_units,
            } => println!(
                "[PARTIDA] {} | {distance_units:.1}u acumuladas",
                station.name()
            ),
            TravelEvent::CircuitCompleted {
                completed_laps,
                distance_units,
            } => println!(
                "[CIRCUITO] volta {completed_laps} concluída | {distance_units:.1}u acumuladas"
            ),
            TravelEvent::AchievementUnlocked {
                achievement,
                distance_units,
            } => println!(
                "[CONQUISTA] {} | {distance_units:.1}u acumuladas",
                achievement_name(achievement)
            ),
        }
    }
}

fn achievement_name(achievement: Achievement) -> &'static str {
    match achievement {
        Achievement::FirstStation => "Primeira estação",
        Achievement::FirstCircuit => "Primeiro circuito",
        Achievement::LongJourney => "Longa viagem",
    }
}
