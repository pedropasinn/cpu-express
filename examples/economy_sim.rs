//! Local Monte Carlo, replaying real arrival traces through the production economy.
// Including these two modules exposes crate-private reducers only to this executable.
pub use cpu_express::simulation::{
    ArrivalSummary, BranchStop, CargoContract, CargoKind, ContractStatus, JourneySnapshot,
    ServiceReward, ServiceState, ServiceWagon, StationId, Train, World, catalog, objectives,
};
#[allow(dead_code)]
#[path = "../src/simulation/economy.rs"]
mod economy;
#[allow(dead_code)]
#[path = "../src/simulation/economy_goals.rs"]
mod economy_goals;
use cpu_express::config::Config;
use economy::{CrateKind, EconomyAction, EconomyState, Rarity};

#[derive(Clone)]
struct Arrival {
    snapshot: JourneySnapshot,
    service: ServiceState,
}
#[derive(Clone, Default)]
struct ResultRow {
    balance: f64,
    crates: f64,
    first_buy: Option<f64>,
    first_rare: Option<f64>,
    complete: Option<f64>,
    hourly: f64,
    stamps: f64,
}
fn arrival_snapshot(world: &World) -> JourneySnapshot {
    let mut snapshot = world.journey_snapshot();
    // The remainder of this <=1 s frame already spent some of the 8 s stop.
    // Orders in World capture their baseline at arrival, before that remainder.
    snapshot.active_seconds -= (8.0 - world.station_wait()).clamp(0.0, 1.0);
    snapshot
}
fn trace(services: bool) -> Vec<Arrival> {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    if !services {
        world.set_automatic_contracts(false);
        world
            .set_service_composition([ServiceWagon::Tank; 3])
            .unwrap();
    }
    let mut result = Vec::new();
    for _ in 0..(240 * 3600) {
        let before = world.journey_snapshot().stations_visited;
        world.advance_expedition(&mut train, 0.0, 1.0, &config);
        if world.journey_snapshot().stations_visited > before {
            result.push(Arrival {
                snapshot: arrival_snapshot(&world),
                service: world.service_state().clone(),
            });
        }
        world.drain_travel_events().for_each(drop);
    }
    result
}
fn reveal(e: &mut EconomyState, a: &Arrival, rare: &mut Option<f64>) {
    let p = e.pending_purchase.as_ref().unwrap();
    if p.rarity >= Rarity::Rare && rare.is_none() {
        *rare = Some(a.snapshot.active_seconds / 60.0);
    }
    let transaction_id = p.transaction_id;
    e.apply(
        EconomyAction::Reveal { transaction_id },
        a.snapshot,
        &a.service,
    )
    .unwrap();
}
fn run(seed: u64, trace: &[Arrival], bonus: u32) -> [ResultRow; 3] {
    let mut e = EconomyState::new(seed);
    let pools: Vec<_> = CrateKind::ALL
        .into_iter()
        .map(|k| k.definition().unwrap())
        .collect();
    let items = EconomyState::collection_items();
    let mut first_buy = None;
    let mut first_rare = None;
    let mut complete = None;
    let mut crates = 1;
    let mut recurring = 0;
    let mut results: [ResultRow; 3] = Default::default();
    let mut checkpoint = 0;
    let mut start = trace[0].clone();
    start.snapshot.active_seconds = 0.0;
    e.apply(
        EconomyAction::BuyCrate {
            kind: CrateKind::GeneralYard,
            free: true,
        },
        start.snapshot,
        &start.service,
    )
    .unwrap();
    reveal(&mut e, &start, &mut first_rare);
    for a in trace {
        let hours = a.snapshot.active_seconds / 3600.0;
        while checkpoint < 3 && hours > [30.0, 90.0, 240.0][checkpoint] {
            results[checkpoint] = ResultRow {
                balance: e.balance as f64,
                crates: crates as f64,
                first_buy,
                first_rare,
                complete,
                hourly: recurring as f64 / [30.0, 90.0, 240.0][checkpoint],
                stamps: e.museum_stamps as f64,
            };
            checkpoint += 1;
        }
        let before = e.total_earned;
        e.arrive(a.snapshot, &a.service, bonus);
        e.offer_order_on_arrival(a.snapshot, &a.service, false);
        let claims: Vec<_> = e
            .daily_orders
            .iter()
            .filter(|o| o.claimable(&e, a.snapshot, &a.service))
            .map(|o| o.id)
            .collect();
        for order_id in claims {
            e.apply(
                EconomyAction::ClaimOrder { order_id },
                a.snapshot,
                &a.service,
            )
            .unwrap();
        }
        recurring += e.total_earned - before;
        // Swap only orders the fixed equipped composition cannot deliver; never buy equipment for an order.
        let blocked = e
            .daily_orders
            .iter()
            .find(|o| match o.template().unwrap().counter {
                economy_goals::OrderCounter::Cargo(k) => {
                    a.service.cargo_capacity(k) < 12 || !a.service.automatic_contracts
                }
                economy_goals::OrderCounter::Passengers => a.service.capacity().0 == 0,
                economy_goals::OrderCounter::Contracts => !a.service.automatic_contracts,
                _ => false,
            })
            .map(|o| o.id);
        if let Some(order_id) = blocked {
            let _ = e.apply(
                EconomyAction::SwapOrder { order_id },
                a.snapshot,
                &a.service,
            );
        }
        for m in e
            .missions(a.snapshot, &a.service)
            .into_iter()
            .filter(|m| m.claimable())
        {
            e.apply(
                EconomyAction::ClaimMission(m.id.into()),
                a.snapshot,
                &a.service,
            )
            .unwrap();
        }
        loop {
            let pool = pools
                .iter()
                .filter(|d| d.items.iter().any(|i| !e.owns(i)))
                .min_by_key(|d| d.price);
            let action = if let Some(pool) = pool {
                if e.balance < pool.price {
                    break;
                }
                EconomyAction::BuyCrate {
                    kind: pool.kind,
                    free: false,
                }
            } else if let Some(item) = items
                .iter()
                .filter(|i| !e.owns(i))
                .min_by_key(|i| i.rarity().unwrap().direct_price())
            {
                if e.balance < item.rarity().unwrap().direct_price() {
                    break;
                }
                EconomyAction::BuyDirect(item.clone())
            } else {
                if complete.is_none() {
                    complete = Some(hours);
                }
                if e.balance < 120 {
                    break;
                }
                EconomyAction::BuyMuseumStamp
            };
            let is_crate = matches!(action, EconomyAction::BuyCrate { .. });
            e.apply(action, a.snapshot, &a.service).unwrap();
            if is_crate {
                first_buy.get_or_insert(a.snapshot.active_seconds / 60.0);
                crates += 1;
                reveal(&mut e, a, &mut first_rare);
            }
            if first_rare.is_none()
                && e.inventory
                    .iter()
                    .any(|i| i.rarity().is_some_and(|r| r >= Rarity::Rare))
            {
                first_rare = Some(a.snapshot.active_seconds / 60.0);
            }
        }
        if first_rare.is_none()
            && e.inventory
                .iter()
                .any(|i| i.rarity().is_some_and(|r| r >= Rarity::Rare))
        {
            first_rare = Some(a.snapshot.active_seconds / 60.0);
        }
    }
    results[2] = ResultRow {
        balance: e.balance as f64,
        crates: crates as f64,
        first_buy,
        first_rare,
        complete,
        hourly: recurring as f64 / 240.0,
        stamps: e.museum_stamps as f64,
    };
    results
}
fn quantiles(mut values: Vec<f64>) -> String {
    values.sort_by(f64::total_cmp);
    let p = |q: f64| values[((values.len() - 1) as f64 * q).round() as usize];
    let show = |v: f64| {
        if v.is_infinite() {
            "não atingido".into()
        } else {
            format!("{v:.2}")
        }
    };
    format!("{} / {} / {}", show(p(0.1)), show(p(0.5)), show(p(0.9)))
}
fn main() {
    let runs = std::env::args()
        .nth(1)
        .map(|s| s.parse::<u64>().expect("Número de execuções"))
        .unwrap_or(1000);
    assert!(runs > 0);
    println!(
        "CPU Express — simulação E1b; {runs} sementes por cenário; percentis P10 / P50 / P90."
    );
    println!(
        "30 dias, horas ativas; sem progresso entre sessões. Renda recorrente inclui ordens e bônus, exclui boas-vindas, missões únicas e reembolsos."
    );
    println!(
        "Estratégia: caixa mais barata com novidade possível; depois compra direta mais barata faltante; depois selos. Empates: ordem CrateKind::ALL/ID. Grátis aberta em t=0."
    );
    println!(
        "Traço Aurora padrão real em passos <=1 s; composição fixa Passageiros/Passageiros/Carga ou Tanque/Tanque/Tanque sem despacho. Resgates em cada chegada; troca de ordem incompatível quando permitida. Sem excursões/telegramas. Sementes econômicas 0..N; rota fixa."
    );
    for services in [false, true] {
        let trace = trace(services);
        for bonus in [1000, 1300] {
            let mut rows: [Vec<ResultRow>; 3] = Default::default();
            for seed in 0..runs {
                for (i, r) in run(seed, &trace, bonus).into_iter().enumerate() {
                    rows[i].push(r);
                }
            }
            for (i, hours) in [1, 3, 8].into_iter().enumerate() {
                let r = &rows[i];
                println!(
                    "\n{hours} h/dia; serviços={services}; bônus=×{:.2}; incompletas={}/{}",
                    bonus as f64 / 1000.0,
                    r.iter().filter(|r| r.complete.is_none()).count(),
                    runs
                );
                for (label, values) in [
                    (
                        "Renda recorrente moedas/h",
                        r.iter().map(|r| r.hourly).collect(),
                    ),
                    ("Saldo final", r.iter().map(|r| r.balance).collect()),
                    (
                        "Caixas (inclui grátis)",
                        r.iter().map(|r| r.crates).collect(),
                    ),
                    (
                        "Primeira compra (min ativos)",
                        r.iter()
                            .map(|r| r.first_buy.unwrap_or(f64::INFINITY))
                            .collect(),
                    ),
                    (
                        "Primeira rara+ possuída (min ativos)",
                        r.iter()
                            .map(|r| r.first_rare.unwrap_or(f64::INFINITY))
                            .collect(),
                    ),
                    (
                        "Coleção completa (h ativas; censura em +infinito)",
                        r.iter()
                            .map(|r| r.complete.unwrap_or(f64::INFINITY))
                            .collect(),
                    ),
                    ("Selos", r.iter().map(|r| r.stamps).collect()),
                ] {
                    println!("{label}: {}", quantiles(values));
                }
            }
        }
    }
}

#[cfg(test)]
mod trace_tests {
    use super::*;

    #[test]
    fn replayed_arrivals_match_product_wallet_inventory_orders_and_rng() {
        for services in [false, true] {
            for bonus in [1000, 1300] {
                let config = Config::default();
                let mut world = World::default();
                let mut train = Train::new(&config);
                world.set_machine_bonus(bonus as f32 / 1000.0);
                if !services {
                    world.set_automatic_contracts(false);
                    world
                        .set_service_composition([ServiceWagon::Tank; 3])
                        .unwrap();
                }
                let mut replay = EconomyState::new(world.journey_snapshot().seed);
                for _ in 0..2400 {
                    let previous = world.journey_snapshot().stations_visited;
                    world.advance_expedition(&mut train, 0.0, 1.0, &config);
                    if world.journey_snapshot().stations_visited != previous {
                        let snapshot = arrival_snapshot(&world);
                        replay.arrive(snapshot, world.service_state(), bonus);
                        replay.offer_order_on_arrival(snapshot, world.service_state(), false);
                        assert_eq!(
                            serde_json::to_value(&replay).unwrap(),
                            serde_json::to_value(world.economy()).unwrap()
                        );
                    }
                }
            }
        }
    }
}
