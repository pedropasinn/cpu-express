use cpu_express::{
    config::Config,
    rendering::SceneRenderer,
    simulation::{
        Train, World,
        machine::{MachineSignals, MachineState},
    },
};

#[test]
fn machine_preserves_services_timetable_and_adds_only_one_fourth_order() {
    let config = Config::default();
    let mut baseline = World::default();
    let mut boosted = World::default();
    let mut trains = [Train::new(&config), Train::new(&config)];
    let mut machine = MachineState::default();
    for second in 0..3600 {
        machine.observe(second as f64, MachineSignals::demo(15.0));
        let effects = machine.effects(second as f64, 3.0, true);
        boosted.set_machine_bonus(effects.multiplier());
        if machine.take_telegram(boosted.journey_snapshot().active_seconds, effects) {
            boosted.request_telegram();
        }
        baseline.advance_expedition(&mut trains[0], 0.0, 1.0, &config);
        boosted.advance_expedition(&mut trains[1], 100.0, 1.0, &config);
        assert_eq!(baseline.service_state(), boosted.service_state());
        assert_eq!(baseline.station_wait(), boosted.station_wait());
        assert_eq!(
            baseline.journey_snapshot().total_distance_units,
            boosted.journey_snapshot().total_distance_units
        );
    }
    assert_eq!(baseline.economy().daily_orders.len(), 3);
    assert_eq!(boosted.economy().daily_orders.len(), 4);
    assert_eq!(
        boosted
            .economy()
            .daily_orders
            .iter()
            .filter(|o| o.telegram)
            .count(),
        1
    );
    for pair in boosted.economy().daily_orders.windows(2) {
        assert!(pair[1].entered_active_seconds - pair[0].entered_active_seconds >= 600.0);
    }
    assert!(boosted.economy().balance > baseline.economy().balance);
}

#[test]
fn yard_effects_are_optional_and_do_not_mutate_the_world() {
    let config = Config::default();
    let mut world = World::default();
    let mut train = Train::new(&config);
    for _ in 0..1000 {
        world.advance_expedition(&mut train, 15.0, 1.0, &config);
        if world.station_wait() > 0.0 {
            break;
        }
    }
    assert!(world.station_wait() > 0.0);
    let economy = world.economy().clone();
    let service = world.service_state().clone();
    let mut machine = MachineState::default();
    machine.observe(0.0, MachineSignals::demo(15.0));
    let mut renderer = SceneRenderer::new();
    let old = renderer.render(&train, &world).to_vec();
    renderer.set_machine_effects(machine.effects(0.0, 3.0, true), true);
    let decorated = renderer.render(&train, &world).to_vec();
    assert_ne!(old, decorated);
    assert_eq!(decorated, renderer.render(&train, &world));
    renderer.set_machine_effects(Default::default(), false);
    assert_eq!(old, renderer.render(&train, &world));
    assert_eq!(&economy, world.economy());
    assert_eq!(&service, world.service_state());
}
